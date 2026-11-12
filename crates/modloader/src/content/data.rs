use crate::registry::{BlockEntry, ItemEntry, RecipeEntry, Registry};
use game::resources::Source;
use mod_protocol::Identifier;
use mod_protocol::abi::RegistryKind;
use mod_protocol::data::{self, DATA, DataFile};
use mod_protocol::spec::{BlockSpec, ItemSpec, RecipeSpec, WorldgenSpec};
use std::cell::RefCell;

thread_local!
{
    // worldgen applies after freeze, when block ids exist
    static PENDING_WORLDGEN: RefCell<Vec<(String, Identifier, WorldgenSpec)>> = const { RefCell::new(Vec::new()) };
}

fn read(source: &mut Source, name: &str) -> Result<String, String>
{
    let bytes: Vec<u8> = source.read(name).ok_or_else(|| format!("{name} could not be read"))?;
    String::from_utf8(bytes).map_err(|_| format!("{name} is not UTF-8"))
}

// one parsed data file goes to the registry it names
fn apply(modId: &str, file: DataFile) -> Result<(), String>
{
    match file
    {
        DataFile::Block(id, spec) => registerBlock(id, modId, *spec),
        DataFile::Item(id, spec) => registerItem(id, modId, *spec),
        DataFile::Recipe(id, spec) => registerRecipe(id, modId, spec),
        DataFile::States(id, spec) => crate::states::register(id, &spec),
        DataFile::Achievement(id, spec) => super::progress::register(id, spec),
        DataFile::Particle(id, spec) => super::particles::register(id, spec),
        DataFile::Effect(id, spec) => super::effects::register(id, spec),
        DataFile::Attribute(id, spec) => crate::attributes::register(id, spec),
        DataFile::GameRule(id, spec) => crate::gamerules::register(id, spec),
        DataFile::Fluid(id, spec) => super::fluids::register(id, spec),
        DataFile::DamageType(id, spec) => crate::damage::register(id, spec),
        DataFile::Loot { registry, target, spec } => super::loot::add(registry, target, spec),
        DataFile::Feature(id, spec) => super::features::register(modId, None, id, *spec, None),
        DataFile::Worldgen(id, spec) =>
        {
            registerWorldgen(modId, id, spec);
            Ok(())
        }

        DataFile::ModifyBlock(spec) =>
        {
            crate::modify::queueBlock(modId, *spec);
            Ok(())
        }

        DataFile::ModifySpawn(spec) =>
        {
            crate::modify::queueSpawn(modId, spec);
            Ok(())
        }

        DataFile::ModifyFeature(spec) =>
        {
            super::features::disable(spec);
            Ok(())
        }

        DataFile::Tag { registry, tag, entries, replace } =>
        {
            super::tags::addEntries(registry, tag, entries, replace);
            Ok(())
        }
    }
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

    crate::registry::with(|registries| -> Result<(), String>
    {
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

    crate::registry::with(|registries|
    {
        registries
            .m_items
            .register(id, owner, ItemEntry { m_spec: Some(spec), m_block: None })
            .map_err(|error| error.to_string())
    })
}

pub fn registerRecipe(id: Identifier, owner: &str, spec: RecipeSpec) -> Result<(), String>
{
    super::recipes::validate(&spec)?;

    crate::registry::with(|registries|
    {
        let registry: &mut Registry<RecipeEntry> = match spec
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

    for name in source.list(DATA).into_iter().filter(|name| data::isData(name))
    {
        let parsed: Result<DataFile, String> = read(source, &name).and_then(|text| data::parse(modId, &name, &text));
        if let Err(reason) = parsed.and_then(|file| apply(modId, file).map_err(|reason| format!("{name}: {reason}")))
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
