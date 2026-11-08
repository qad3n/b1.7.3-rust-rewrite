use mod_protocol::Identifier;
use mod_protocol::abi::{BiomeSpec, DimensionSpec, Handle};
use std::cell::RefCell;
use std::collections::BTreeMap;

#[derive(Clone)]
pub struct ModDimension
{
    pub m_id: Identifier,
    pub m_mod: usize,
    pub m_spec: DimensionSpec,
    pub m_numeric: i32,
}

#[derive(Clone)]
pub struct ModBiome
{
    pub m_id: Identifier,
    pub m_mod: usize,
    pub m_spec: BiomeSpec,
}

#[derive(Default)]
struct Tables
{
    m_dimensions: BTreeMap<Identifier, ModDimension>,
    m_biomes: BTreeMap<Identifier, ModBiome>,
    // dimension id to the guest generator that fills its chunks, a vanilla dimension may also get one
    m_generators: BTreeMap<Identifier, (usize, Handle)>,
    // dimension id to the guest that fills each new chunk, absent means flat terrain from the spec
    m_terrains: BTreeMap<Identifier, (usize, Handle)>,
}

thread_local! {
    static TABLES: RefCell<Tables> = RefCell::new(Tables::default());
}

pub fn registerDimension(modIndex: usize, id: Identifier, spec: DimensionSpec) -> Result<(), String>
{
    let owner: String = crate::state::modId(modIndex);
    crate::registry::with(|registries| {
        registries
            .m_dimensions
            .register(id.clone(), &owner, crate::registry::DimensionEntry)
            .map_err(|error| error.to_string())
    })?;
    TABLES.with(|tables| {
        tables
            .borrow_mut()
            .m_dimensions
            .insert(id.clone(), ModDimension { m_id: id, m_mod: modIndex, m_spec: spec, m_numeric: 0 })
    });
    Ok(())
}

pub fn registerBiome(modIndex: usize, id: Identifier, spec: BiomeSpec) -> Result<(), String>
{
    let owner: String = crate::state::modId(modIndex);
    crate::registry::with(|registries| {
        registries.m_biomes.register(id.clone(), &owner, crate::registry::BiomeEntry).map_err(|error| error.to_string())
    })?;
    TABLES.with(|tables| {
        tables.borrow_mut().m_biomes.insert(id.clone(), ModBiome { m_id: id, m_mod: modIndex, m_spec: spec })
    });
    Ok(())
}

pub fn registerGenerator(modIndex: usize, dimension: Identifier, handler: Handle)
{
    TABLES.with(|tables| tables.borrow_mut().m_generators.insert(dimension, (modIndex, handler)));
}

pub fn registerTerrain(modIndex: usize, dimension: Identifier, handler: Handle)
{
    TABLES.with(|tables| tables.borrow_mut().m_terrains.insert(dimension, (modIndex, handler)));
}

pub fn terrain(dimension: &Identifier) -> Option<(usize, Handle)>
{
    TABLES.with(|tables| tables.borrow().m_terrains.get(dimension).copied())
}

// numeric dimension ids come from the frozen registry
pub fn finish()
{
    let ids: Vec<Identifier> = TABLES.with(|tables| tables.borrow().m_dimensions.keys().cloned().collect());
    for id in ids
    {
        let numeric: Option<i32> = crate::registry::with(|registries| registries.m_dimensions.numeric(&id));
        TABLES.with(|tables| {
            if let (Some(numeric), Some(dimension)) = (numeric, tables.borrow_mut().m_dimensions.get_mut(&id))
            {
                dimension.m_numeric = numeric;
            }
        });
    }
}

pub fn dimensions() -> Vec<ModDimension>
{
    TABLES.with(|tables| tables.borrow().m_dimensions.values().cloned().collect())
}

pub fn dimension(numeric: i32) -> Option<ModDimension>
{
    TABLES
        .with(|tables| tables.borrow().m_dimensions.values().find(|dimension| dimension.m_numeric == numeric).cloned())
}

pub fn biomes() -> Vec<ModBiome> { TABLES.with(|tables| tables.borrow().m_biomes.values().cloned().collect()) }

pub fn generator(dimension: &Identifier) -> Option<(usize, Handle)>
{
    TABLES.with(|tables| tables.borrow().m_generators.get(dimension).copied())
}

pub fn isModded(numeric: i32) -> bool { dimension(numeric).is_some() }

// a generator registered for a dimension runs once per populated chunk with that chunk's seed
pub fn populate(world: &std::rc::Rc<game::world::World>, chunkX: i32, chunkZ: i32)
{
    let numeric: i32 = crate::host::context::dimension(world);
    let id: Option<Identifier> =
        crate::registry::with(|registries| registries.m_dimensions.byNumeric(numeric).map(|row| row.m_id.clone()));
    let Some((modIndex, handler)) = id.and_then(|id| generator(&id))
    else
    {
        return;
    };
    let Some(runtime) = crate::state::modAt(modIndex)
    else
    {
        return;
    };
    let chunk: mod_protocol::abi::ChunkInfo =
        mod_protocol::abi::ChunkInfo { world: numeric, x: chunkX, z: chunkZ, seed: world.getRandomSeed() };
    crate::host::context::withWorld(world, || runtime.call(&mod_protocol::abi::GuestCall::Generate { handler, chunk }));
}

// the dedicated server keeps one world, tracker and player manager per entry, vanilla dimensions first
pub fn serverDimensions() -> Vec<i32>
{
    let mut list: Vec<i32> = vec![0, -1];
    let mut modded: Vec<i32> = dimensions().iter().map(|dimension| dimension.m_numeric).collect();
    modded.sort_unstable();
    list.extend(modded);
    list
}

pub fn serverSlot(dimension: i32) -> usize
{
    serverDimensions().iter().position(|entry| *entry == dimension).unwrap_or(0)
}
