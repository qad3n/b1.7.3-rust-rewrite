use crate::discovery::Source;
use crate::registry::{BlockEntry, ItemEntry, RecipeEntry};
use mod_protocol::Identifier;
use mod_protocol::spec::{BlockSpec, ItemSpec, RecipeSpec, TagSpec, WorldgenSpec};
use serde::de::DeserializeOwned;
use std::cell::RefCell;

const DATA: &str = "data/";
const TOML: &str = ".toml";

thread_local! {
    // worldgen applies after freeze, when block ids exist
    static PENDING_WORLDGEN: RefCell<Vec<(String, WorldgenSpec)>> = const { RefCell::new(Vec::new()) };
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
        super::tags::add(mod_protocol::abi::RegistryKind::Block, tag.clone(), vec![id.clone()], false);
    }
    crate::registry::with(|registries| -> Result<(), String> {
        registries
            .m_blocks
            .register(id.clone(), owner, BlockEntry { m_spec: Some(spec) })
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
        super::tags::add(mod_protocol::abi::RegistryKind::Item, tag.clone(), vec![id.clone()], false);
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

pub fn registerWorldgen(owner: &str, spec: WorldgenSpec)
{
    PENDING_WORLDGEN.with(|list| list.borrow_mut().push((owner.to_owned(), spec)));
}

pub fn takeWorldgen() -> Vec<(String, WorldgenSpec)>
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
            "worldgen" => parse::<WorldgenSpec>(source, &name).map(|spec| registerWorldgen(modId, spec)),
            "tags" =>
            {
                let (folder, tagPath) =
                    path.split_once('/').ok_or_else(|| format!("{name} needs tags/<registry>/<tag>.toml"))?;
                let registry = super::tags::registryKind(folder)
                    .ok_or_else(|| format!("{name}: unknown tag registry {folder}"))?;
                let tag: Identifier = Identifier::new(namespace, tagPath).map_err(|error| error.to_string())?;
                parse::<TagSpec>(source, &name).map(|spec| super::tags::add(registry, tag, spec.values, spec.replace))
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
            Some(text) => super::lang::load(&text),
            None => problems.push(format!("{name} could not be read")),
        }
    }
    problems
}
