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

// how tall a mod dimension's world is and the limit players place blocks below, vanilla's 128 unless a mod sets them
#[derive(Clone, Copy)]
pub struct Height
{
    pub m_world: i32,
    pub m_build: i32,
}

mod limits
{
    // chunk sections are 16 high
    pub const SECTION: i32 = 16;
    // block change packets carry y in one unsigned byte
    pub const MAX_HEIGHT: i32 = 256;
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
    // dimension id to the guest generators that populate its chunks in registration order, vanilla dimensions too
    m_generators: BTreeMap<Identifier, Vec<(usize, Handle)>>,
    // dimension id to the guest that fills each new chunk, absent means flat terrain from the spec
    m_terrains: BTreeMap<Identifier, (usize, Handle)>,
    // dimension id to the guest that names its biomes and the biome list its indices point into
    m_biomeSources: BTreeMap<Identifier, (usize, Handle, Vec<Identifier>)>,
    // dimension id to its world height and build limit, absent means vanilla's
    m_heights: BTreeMap<Identifier, Height>,
    // dimension id to the guest that names what may spawn at a spot, absent means the biome lists
    m_spawnLists: BTreeMap<Identifier, (usize, Handle)>,
    // dimension id to the seed it generates from instead of the save's, absent means the save's
    m_seeds: BTreeMap<Identifier, i64>,
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
    TABLES.with(|tables| tables.borrow_mut().m_generators.entry(dimension).or_default().push((modIndex, handler)));
}

// a dimension's shape, biomes, height and spawns belong to the mod that registered it, so two mods never fight over one
fn owned(modIndex: usize, dimension: &Identifier) -> Result<(), String>
{
    let owner: Option<usize> =
        TABLES.with(|tables| tables.borrow().m_dimensions.get(dimension).map(|registered| registered.m_mod));
    if owner != Some(modIndex)
    {
        return Err(format!("dimension {dimension} is not registered by this mod"));
    }
    Ok(())
}

pub fn registerTerrain(modIndex: usize, dimension: Identifier, handler: Handle) -> Result<(), String>
{
    owned(modIndex, &dimension)?;
    TABLES.with(|tables| tables.borrow_mut().m_terrains.insert(dimension, (modIndex, handler)));
    Ok(())
}

pub fn registerBiomeSource(
    modIndex: usize,
    dimension: Identifier,
    biomes: Vec<Identifier>,
    handler: Handle,
) -> Result<(), String>
{
    owned(modIndex, &dimension)?;
    TABLES.with(|tables| tables.borrow_mut().m_biomeSources.insert(dimension, (modIndex, handler, biomes)));
    Ok(())
}

pub fn registerSpawns(modIndex: usize, dimension: Identifier, handler: Handle) -> Result<(), String>
{
    owned(modIndex, &dimension)?;
    TABLES.with(|tables| tables.borrow_mut().m_spawnLists.insert(dimension, (modIndex, handler)));
    Ok(())
}
pub fn hasSpawnList(numeric: i32) -> bool
{
    idOf(numeric).is_some_and(|id| TABLES.with(|tables| tables.borrow().m_spawnLists.contains_key(&id)))
}
// what may spawn of a kind at a spot of the world, None when no guest answers for its dimension
pub fn spawnList(
    world: &std::rc::Rc<game::world::World>,
    kind: mod_protocol::spec::CreatureKind,
    pos: mod_protocol::abi::BlockPos,
) -> Option<Vec<mod_protocol::abi::SpawnEntry>>
{
    let numeric: i32 = crate::host::context::dimension(world);
    let (modIndex, handler) = TABLES.with(|tables| tables.borrow().m_spawnLists.get(&idOf(numeric)?).copied())?;
    let runtime = crate::state::modAt(modIndex)?;
    let call: mod_protocol::abi::GuestCall =
        mod_protocol::abi::GuestCall::Spawns { handler, world: numeric, kind, pos };
    match crate::host::context::withWorld(world, || runtime.call(&call))?
    {
        mod_protocol::abi::GuestReply::Spawns { entries } => entries,
        _ => None,
    }
}

fn idOf(numeric: i32) -> Option<Identifier>
{
    crate::registry::with(|registries| registries.m_dimensions.byNumeric(numeric).map(|row| row.m_id.clone()))
}

pub fn biomeSource(dimension: &Identifier) -> Option<(usize, Handle, Vec<Identifier>)>
{
    TABLES.with(|tables| tables.borrow().m_biomeSources.get(dimension).cloned())
}

// only the mod that registered a dimension sets its height, before any world of it exists
pub fn registerHeight(modIndex: usize, dimension: Identifier, height: i32, buildLimit: i32) -> Result<(), String>
{
    owned(modIndex, &dimension)?;
    if height % limits::SECTION != 0 || !(limits::SECTION..=limits::MAX_HEIGHT).contains(&height)
    {
        return Err(format!("dimension height {height} must be a multiple of 16 from 16 to {}", limits::MAX_HEIGHT));
    }
    if !(1..=height).contains(&buildLimit)
    {
        return Err(format!("build limit {buildLimit} must lie in 1..={height}"));
    }
    TABLES.with(|tables| {
        tables.borrow_mut().m_heights.insert(dimension, Height { m_world: height, m_build: buildLimit })
    });
    Ok(())
}

// only the mod that registered a dimension sets its seed, before any world of it exists
pub fn registerSeed(modIndex: usize, dimension: Identifier, seed: i64) -> Result<(), String>
{
    owned(modIndex, &dimension)?;
    TABLES.with(|tables| tables.borrow_mut().m_seeds.insert(dimension, seed));
    Ok(())
}

pub fn seed(dimension: &Identifier) -> Option<i64>
{
    TABLES.with(|tables| tables.borrow().m_seeds.get(dimension).copied())
}

pub fn height(dimension: &Identifier) -> Height
{
    TABLES
        .with(|tables| tables.borrow().m_heights.get(dimension).copied())
        .unwrap_or(Height { m_world: game::world::HEIGHT, m_build: game::world::HEIGHT })
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

pub fn generators(dimension: &Identifier) -> Vec<(usize, Handle)>
{
    TABLES.with(|tables| tables.borrow().m_generators.get(dimension).cloned()).unwrap_or_default()
}

pub fn isModded(numeric: i32) -> bool { dimension(numeric).is_some() }

// every generator registered for a dimension runs once per populated chunk with that chunk's seed
pub fn populate(world: &std::rc::Rc<game::world::World>, chunkX: i32, chunkZ: i32)
{
    let numeric: i32 = crate::host::context::dimension(world);
    let Some(id) = idOf(numeric)
    else
    {
        return;
    };

    let chunk: mod_protocol::abi::ChunkInfo =
        mod_protocol::abi::ChunkInfo { world: numeric, x: chunkX, z: chunkZ, seed: world.getRandomSeed() };
    for (modIndex, handler) in generators(&id)
    {
        if let Some(runtime) = crate::state::modAt(modIndex)
        {
            let call: mod_protocol::abi::GuestCall =
                mod_protocol::abi::GuestCall::Generate { handler, chunk: chunk.clone() };
            crate::host::context::withWorld(world, || runtime.call(&call));
        }
    }
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
