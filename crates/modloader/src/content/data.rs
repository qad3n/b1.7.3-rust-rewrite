use crate::registry::{BlockEntry, ItemEntry, RecipeEntry, Registry};
use game::resources::Source;
use mod_protocol::Identifier;
use mod_protocol::abi::RegistryKind;
use mod_protocol::data::{self, DATA, DataFile, Parsed};
use mod_protocol::spec::{BlockSpec, ItemSpec, RecipeSpec, WorldgenSpec};
use std::cell::RefCell;

thread_local!
{
    // worldgen applies after freeze, when block ids exist
    static PENDING_WORLDGEN: RefCell<Vec<(String, Identifier, WorldgenSpec)>> = const { RefCell::new(Vec::new()) };
}

pub fn read(source: &mut Source, name: &str) -> Result<String, String>
{
    let bytes: Vec<u8> = source.read(name).ok_or_else(|| format!("{name} could not be read"))?;
    String::from_utf8(bytes).map_err(|_| format!("{name} is not UTF-8"))
}

// the mod a data file came from, the override folder has no mod and may not add content that runs mod code
pub struct Owner
{
    pub m_id: String,
    pub m_index: Option<usize>,
}

impl Owner
{
    fn index(&self, kind: &str) -> Result<usize, String>
    {
        self.m_index.ok_or_else(|| format!("the override folder cannot add {kind}, they belong to a mod"))
    }
}

// files that name content other mods add, or that set conditions, wait until every mod registered
fn waits(file: &DataFile) -> bool
{
    matches!(
        file,
        DataFile::MachineJob { .. }
            | DataFile::MachineFuel { .. }
            | DataFile::MachineRecipe { .. }
            | DataFile::ModifyMachine(_)
            | DataFile::ModifyRecipes(_)
    )
}

// one parsed data file goes to the registry it names
pub fn apply(owner: &Owner, file: DataFile) -> Result<(), String>
{
    let modId: &str = &owner.m_id;

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

        DataFile::ModifyItem(spec) =>
        {
            crate::modify::queueItem(modId, spec);
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

        DataFile::ModifyRecipes(spec) => spec.remove.iter().try_for_each(crate::lifecycle::queueRecipeRemoval),
        DataFile::ModifyMachine(spec) => super::machines::removeOutputs(&spec.machine, &spec.remove_outputs),
        DataFile::Tag { registry, tag, entries, replace } =>
        {
            super::tags::addEntries(registry, tag, entries, replace);
            Ok(())
        }

        DataFile::RegistryTag { registry, tag, entries, replace } =>
        {
            super::tags::addEntries(super::tags::Scope::Custom(registry), tag, entries, replace);
            Ok(())
        }

        DataFile::RegistryEntry { registry, id, value } => super::custom::add(registry, id, value),
        DataFile::DataMap { map, target, values, remove, replace } =>
        {
            super::custom::setValues(map, Some(target), values, remove, replace);
            Ok(())
        }

        DataFile::Entity(id, spec) => crate::entities::register(id, owner.index("entities")?, *spec),
        DataFile::TileEntity(id, spec) => crate::tiles::register(id, modId, owner.index("tile entities")?, spec),
        DataFile::Machine(id, spec) => super::machines::register(id, spec),
        DataFile::MachineJob { machine, job } => super::machines::addJob(&machine, *job, Vec::new()),
        DataFile::MachineFuel { machine, fuel } => super::machines::addFuel(&machine, fuel),
        DataFile::MachineRecipe { machine, recipe } => super::machines::addRecipe(&machine, recipe),
        DataFile::AccessorySlot(id, spec) => super::accessories::register(id, spec),
        DataFile::Energy(id, spec) => crate::energy::register(modId, id, spec),
        DataFile::FluidContainer(spec) => super::fluids::addContainer(spec),
        DataFile::LootPool { target, id, pool } => super::loot::addPool(target, id, pool),
        DataFile::ChestLoot(table, spec) => super::loot::addChest(table, spec),
        DataFile::Structure(id, spec) => crate::structures::register(owner.index("structures")?, id, spec, None),
        DataFile::Biome(id, spec) => crate::dimensions::registerBiome(owner.index("biomes")?, id, spec),
        DataFile::Template(id, spec) => super::structure_templates::register(id, spec),
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
pub fn load(owner: &Owner, source: &mut Source) -> Vec<String>
{
    let mut problems: Vec<String> = Vec::new();

    for name in source.list(DATA).into_iter().filter(|name| data::isData(name))
    {
        let parsed: Result<Parsed, String> =
            read(source, &name).and_then(|text| data::parse(Some(&owner.m_id), &name, &text));
        let applied: Result<(), String> =
            parsed.and_then(|parsed| match parsed.when.is_some() || waits(&parsed.file)
            {
                true =>
                {
                    super::deferred::push(owner, name.clone(), parsed);
                    Ok(())
                }
                false => apply(owner, parsed.file).map_err(|reason| format!("{name}: {reason}")),
            });
        problems.extend(applied.err());
    }

    for name in source.list("assets/").into_iter().filter(|name| name.ends_with(super::lang::LANG_FILE))
    {
        match source.read(&name).map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
        {
            Some(text) => super::lang::load(&owner.m_id, &text),
            None => problems.push(format!("{name} could not be read")),
        }
    }

    problems
}
