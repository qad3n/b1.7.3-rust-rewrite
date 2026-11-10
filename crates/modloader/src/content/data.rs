use crate::discovery::Source;
use crate::registry::{BlockEntry, ItemEntry, RecipeEntry};
use mod_protocol::Identifier;
use mod_protocol::abi::RegistryKind;
use mod_protocol::spec::{
    BlockModifySpec, BlockSpec, BlockStatesSpec, EffectSpec, FluidSpec, ItemSpec, LootSpec, ParticleSpec, RecipeSpec,
    SpawnModifySpec, TagEntry, TagSpec, TagValue, WorldgenSpec,
};
use serde::de::DeserializeOwned;
use std::cell::RefCell;

const DATA: &str = "data/";
const TOML: &str = ".toml";

thread_local! {
    // worldgen applies after freeze, when block ids exist
    static PENDING_WORLDGEN: RefCell<Vec<(String, Identifier, WorldgenSpec)>> = const { RefCell::new(Vec::new()) };
}

fn parse<T: DeserializeOwned>(source: &mut Source, name: &str) -> Result<T, String>
{
    let bytes: Vec<u8> = source.read(name).ok_or_else(|| format!("{name} could not be read"))?;
    let text: &str = std::str::from_utf8(&bytes).map_err(|_| format!("{name} is not UTF-8"))?;
    toml::from_str(text).map_err(|error| format!("{name}: {error}"))
}

// data/<namespace>/<kind>/<path>.toml gives namespace:path, only tags may use a namespace other than the mod id
fn split(name: &str) -> Option<(&str, &str, &str)>
{
    let rest: &str = name.strip_prefix(DATA)?.strip_suffix(TOML)?;
    let (namespace, rest) = rest.split_once('/')?;
    let (kind, path) = rest.split_once('/')?;
    Some((namespace, kind, path))
}

pub fn registerBlock(id: Identifier, owner: &str, spec: BlockSpec) -> Result<(), String>
{
    super::blocks::validate(&spec)?;
    if let Some(item) = &spec.item
    {
        super::items::requestTextures(item);
    }
    super::blocks::requestTextures(&spec);
    for tag in &spec.tags
    {
        super::tags::add(RegistryKind::Block, tag.clone(), vec![id.clone()], false);
    }
    crate::registry::with(|registries| -> Result<(), String> {
        registries
            .m_blocks
            .register(id.clone(), owner, BlockEntry { m_spec: Some(spec), m_page: None })
            .map_err(|error| error.to_string())?;
        registries
            .m_items
            .register(id.clone(), owner, ItemEntry { m_spec: None, m_block: Some(id) })
            .map_err(|error| error.to_string())
    })
}

pub fn registerItem(id: Identifier, owner: &str, spec: ItemSpec) -> Result<(), String>
{
    super::items::validate(&spec)?;
    super::items::requestTextures(&spec);
    for tag in &spec.tags
    {
        super::tags::add(RegistryKind::Item, tag.clone(), vec![id.clone()], false);
    }
    crate::registry::with(|registries| {
        registries
            .m_items
            .register(id, owner, ItemEntry { m_spec: Some(spec), m_block: None })
            .map_err(|error| error.to_string())
    })
}

pub fn registerRecipe(id: Identifier, owner: &str, spec: RecipeSpec) -> Result<(), String>
{
    super::recipes::validate(&spec)?;
    crate::registry::with(|registries| {
        let registry = match spec
        {
            RecipeSpec::Smelting(_) => &mut registries.m_smelting,
            _ => &mut registries.m_recipes,
        };
        registry.register(id, owner, RecipeEntry { m_spec: Some(spec) }).map_err(|error| error.to_string())
    })
}

// the feature's identifier seeds its random, so adding or removing other mods never moves its ores
pub fn registerWorldgen(owner: &str, id: Identifier, spec: WorldgenSpec)
{
    PENDING_WORLDGEN.with(|list| list.borrow_mut().push((owner.to_owned(), id, spec)));
}

pub fn takeWorldgen() -> Vec<(String, Identifier, WorldgenSpec)>
{
    PENDING_WORLDGEN.with(|list| list.borrow_mut().drain(..).collect())
}

// data content registers before the mod code runs, so code can look it up
pub fn load(modId: &str, source: &mut Source) -> Vec<String>
{
    let mut problems: Vec<String> = Vec::new();
    for name in source.list(DATA).into_iter().filter(|name| name.ends_with(TOML))
    {
        let Some((namespace, kind, path)) = split(&name)
        else
        {
            problems.push(format!("{name} is not data/<namespace>/<kind>/<path>.toml"));
            continue;
        };
        if kind != "tags" && namespace != modId
        {
            problems.push(format!("{name} registers into namespace {namespace}, a mod only owns {modId}"));
            continue;
        }
        let id: Result<Identifier, String> = Identifier::new(namespace, path).map_err(|error| error.to_string());
        let result: Result<(), String> = id.and_then(|id| match kind
        {
            "blocks" => parse::<BlockSpec>(source, &name).and_then(|spec| registerBlock(id, modId, spec)),
            "items" => parse::<ItemSpec>(source, &name).and_then(|spec| registerItem(id, modId, spec)),
            "recipes" => parse::<RecipeSpec>(source, &name).and_then(|spec| registerRecipe(id, modId, spec)),
            "states" => parse::<BlockStatesSpec>(source, &name).and_then(|spec| crate::states::register(id, &spec)),
            "particles" => parse::<ParticleSpec>(source, &name).map(|spec| super::particles::register(id, spec)),
            "effects" => parse::<EffectSpec>(source, &name).and_then(|spec| super::effects::register(id, spec)),
            "fluids" => parse::<FluidSpec>(source, &name).and_then(|spec| super::fluids::register(id, spec)),
            "loot" =>
            {
                let (registry, target) = lootTarget(path)
                    .ok_or_else(|| format!("{name} needs loot/<blocks|entities>/<namespace>/<path>.toml"))?;
                parse::<LootSpec>(source, &name).and_then(|spec| super::loot::add(registry, target, spec))
            }
            "worldgen" => parse::<WorldgenSpec>(source, &name).map(|spec| registerWorldgen(modId, id, spec)),
            "modify" => match path.split_once('/')
            {
                Some(("blocks", _)) =>
                {
                    parse::<BlockModifySpec>(source, &name).map(|spec| crate::modify::queueBlock(modId, spec))
                }
                Some(("spawns", _)) =>
                {
                    parse::<SpawnModifySpec>(source, &name).map(|spec| crate::modify::queueSpawn(modId, spec))
                }
                _ => Err(format!("{name} needs modify/blocks/<name>.toml or modify/spawns/<name>.toml")),
            },
            "tags" =>
            {
                let (folder, tagPath) =
                    path.split_once('/').ok_or_else(|| format!("{name} needs tags/<registry>/<tag>.toml"))?;
                let registry = super::tags::registryKind(folder)
                    .ok_or_else(|| format!("{name}: unknown tag registry {folder}"))?;
                let tag: Identifier = Identifier::new(namespace, tagPath).map_err(|error| error.to_string())?;
                parse::<TagSpec>(source, &name).and_then(|spec| {
                    let entries: Vec<TagEntry> = spec
                        .values
                        .iter()
                        .map(TagValue::entry)
                        .collect::<Result<_, _>>()
                        .map_err(|error| format!("{name}: {error}"))?;
                    super::tags::addEntries(registry, tag, entries, spec.replace);
                    Ok(())
                })
            }
            other => Err(format!("{name}: unknown data kind {other}")),
        });
        if let Err(reason) = result
        {
            problems.push(reason);
        }
    }
    for name in source.list("assets/").into_iter().filter(|name| name.ends_with(super::lang::LANG_FILE))
    {
        match source.read(&name).map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
        {
            Some(text) => super::lang::load(modId, &text),
            None => problems.push(format!("{name} could not be read")),
        }
    }
    problems
}

// loot/blocks/minecraft/gravel names the block minecraft:gravel
fn lootTarget(path: &str) -> Option<(RegistryKind, Identifier)>
{
    let (folder, rest) = path.split_once('/')?;
    let registry: RegistryKind = match folder
    {
        "blocks" => RegistryKind::Block,
        "entities" => RegistryKind::Entity,
        _ => return None,
    };
    let (namespace, targetPath) = rest.split_once('/')?;
    Some((registry, Identifier::new(namespace, targetPath).ok()?))
}
