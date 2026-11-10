use crate::dimensions::{self, ModDimension};
use game::world::World;
use game::worldgen::{BiomeGenBase, WORLD_CHUNK_MANAGER_HELL, WorldChunkManager, WorldChunkManagerClass};
use mod_protocol::Identifier;
use mod_protocol::abi::{ChunkInfo, GuestCall, GuestReply, Handle};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::{Rc, Weak};

const COLUMNS: usize = 256;
// chunks of biome columns a source keeps before it starts over
const CACHE_LIMIT: usize = 4096;

// what a biome source said about one chunk, x fastest
struct Columns
{
    m_biomes: Vec<u8>,
}

// a mod dimension whose biomes the mod names column by column, one per world chunk manager
struct Source
{
    m_mod: usize,
    m_handler: Handle,
    m_dimension: i32,
    m_world: Weak<World>,
    m_biomes: Vec<Rc<BiomeGenBase>>,
    m_climates: Vec<(f64, f64)>,
    m_chunks: RefCell<HashMap<(i32, i32), Rc<Columns>>>,
}

thread_local! {
    // sources by the address of the chunk manager they answer for
    static SOURCES: RefCell<HashMap<usize, Rc<Source>>> = RefCell::new(HashMap::new());
}

fn key(manager: &WorldChunkManager) -> usize { std::ptr::from_ref(manager) as usize }

fn source(manager: &WorldChunkManager) -> Option<Rc<Source>>
{
    SOURCES.with(|sources| sources.borrow().get(&key(manager)).cloned())
}

fn fallback() -> Columns { Columns { m_biomes: vec![0; COLUMNS] } }

impl Source
{
    fn ask(&self, chunkX: i32, chunkZ: i32) -> Columns
    {
        let Some((world, runtime)) = self.m_world.upgrade().zip(crate::state::modAt(self.m_mod))
        else
        {
            return fallback();
        };
        let chunk: ChunkInfo = ChunkInfo { world: self.m_dimension, x: chunkX, z: chunkZ, seed: world.getRandomSeed() };
        let reply: Option<GuestReply> = crate::host::context::withWorld(&world, || {
            runtime.call(&GuestCall::BiomeSource { handler: self.m_handler, chunk })
        });
        match reply
        {
            Some(GuestReply::Biomes { biomes }) if biomes.len() == COLUMNS => Columns { m_biomes: biomes },
            _ =>
            {
                crate::log::warn(runtime.id(), &format!("biome source gave no columns for chunk {chunkX} {chunkZ}"));
                fallback()
            }
        }
    }

    fn chunk(&self, chunkX: i32, chunkZ: i32) -> Rc<Columns>
    {
        if let Some(columns) = self.m_chunks.borrow().get(&(chunkX, chunkZ))
        {
            return Rc::clone(columns);
        }
        let columns: Rc<Columns> = Rc::new(self.ask(chunkX, chunkZ));
        let mut chunks = self.m_chunks.borrow_mut();
        if chunks.len() >= CACHE_LIMIT
        {
            chunks.clear();
        }
        chunks.insert((chunkX, chunkZ), Rc::clone(&columns));
        columns
    }

    fn index(&self, x: i32, z: i32) -> usize
    {
        let columns: Rc<Columns> = self.chunk(x >> 4, z >> 4);
        let biome: usize = usize::from(columns.m_biomes[((z & 15) * 16 + (x & 15)) as usize]);
        biome.min(self.m_biomes.len() - 1)
    }

    fn biome(&self, x: i32, z: i32) -> Rc<BiomeGenBase> { Rc::clone(&self.m_biomes[self.index(x, z)]) }

    fn climate(&self, x: i32, z: i32) -> (f64, f64) { self.m_climates[self.index(x, z)] }
}

fn with<T>(this: &Rc<WorldChunkManager>, run: impl FnOnce(&Source) -> T, otherwise: impl FnOnce() -> T) -> T
{
    match source(this)
    {
        Some(source) => run(&source),
        None => otherwise(),
    }
}

fn loadBlockGeneratorData(
    this: &Rc<WorldChunkManager>,
    biomes: Option<Vec<Option<Rc<BiomeGenBase>>>>,
    x: i32,
    z: i32,
    sizeX: i32,
    sizeZ: i32,
) -> Vec<Option<Rc<BiomeGenBase>>>
{
    let Some(source) = source(this)
    else
    {
        return (WORLD_CHUNK_MANAGER_HELL.loadBlockGeneratorData)(this, biomes, x, z, sizeX, sizeZ);
    };
    let n: usize = (sizeX * sizeZ) as usize;
    let mut biomes: Vec<Option<Rc<BiomeGenBase>>> = match biomes
    {
        Some(values) if values.len() >= n => values,
        _ => vec![None; n],
    };
    let mut temperature: Vec<f64> = vec![0.0; n];
    let mut humidity: Vec<f64> = vec![0.0; n];
    let mut index: usize = 0;
    for dx in 0..sizeX
    {
        for dz in 0..sizeZ
        {
            let (heat, rain): (f64, f64) = source.climate(x + dx, z + dz);
            biomes[index] = Some(source.biome(x + dx, z + dz));
            temperature[index] = heat;
            humidity[index] = rain;
            index += 1;
        }
    }
    this.m_temperature.set(Some(temperature));
    this.m_humidity.set(Some(humidity));
    biomes
}

// WorldChunkManagerHell with every column asked of the mod
pub const MOD_BIOME_SOURCE: WorldChunkManagerClass = WorldChunkManagerClass {
    m_name: "WorldChunkManagerMod",
    m_parent: Some(&WORLD_CHUNK_MANAGER_HELL),
    getBiomeGenAtChunkCoord: |this, pos| {
        with(
            this,
            |source| source.biome(pos.m_chunkXPos << 4, pos.m_chunkZPos << 4),
            || (WORLD_CHUNK_MANAGER_HELL.getBiomeGenAtChunkCoord)(this, pos),
        )
    },
    getBiomeGenAt: |this, x, z| {
        with(this, |source| source.biome(x, z), || (WORLD_CHUNK_MANAGER_HELL.getBiomeGenAt)(this, x, z))
    },
    getTemperature: |this, x, z| {
        with(this, |source| source.climate(x, z).0, || (WORLD_CHUNK_MANAGER_HELL.getTemperature)(this, x, z))
    },
    getTemperatures: |this, temperatures, x, z, sizeX, sizeZ| {
        with(
            this,
            |source| {
                let mut values: Vec<f64> = Vec::with_capacity((sizeX * sizeZ) as usize);
                for dx in 0..sizeX
                {
                    for dz in 0..sizeZ
                    {
                        values.push(source.climate(x + dx, z + dz).0);
                    }
                }
                values
            },
            || (WORLD_CHUNK_MANAGER_HELL.getTemperatures)(this, temperatures, x, z, sizeX, sizeZ),
        )
    },
    loadBlockGeneratorData,
    ..WORLD_CHUNK_MANAGER_HELL
};

// the chunk manager of a mod dimension with a biome source, none for one without
pub fn manager(
    world: &Rc<World>,
    dimension: &ModDimension,
    makeBiome: impl Fn(&Identifier) -> Rc<BiomeGenBase>,
) -> Option<Rc<WorldChunkManager>>
{
    let (modIndex, handler, ids): (usize, Handle, Vec<Identifier>) = dimensions::biomeSource(&dimension.m_id)?;
    if ids.is_empty()
    {
        return None;
    }
    let specs: Vec<dimensions::ModBiome> = dimensions::biomes();
    let climates: Vec<(f64, f64)> = ids
        .iter()
        .map(|id| {
            specs
                .iter()
                .find(|biome| biome.m_id == *id)
                .map_or((0.5, 0.5), |biome| (f64::from(biome.m_spec.temperature), f64::from(biome.m_spec.rainfall)))
        })
        .collect();
    let biomes: Vec<Rc<BiomeGenBase>> = ids.iter().map(makeBiome).collect();
    let (temperature, rainfall): (f64, f64) = climates[0];
    // a fresh hell manager takes the mod class before anything else holds it, so vanilla code stays untouched
    let mut manager: Rc<WorldChunkManager> = WorldChunkManager::newHell(Rc::clone(&biomes[0]), temperature, rainfall);
    Rc::get_mut(&mut manager).expect("a new chunk manager has one owner").m_class = &MOD_BIOME_SOURCE;
    let source: Source = Source {
        m_mod: modIndex,
        m_handler: handler,
        m_dimension: dimension.m_numeric,
        m_world: Rc::downgrade(world),
        m_biomes: biomes,
        m_climates: climates,
        m_chunks: RefCell::new(HashMap::new()),
    };
    SOURCES.with(|sources| sources.borrow_mut().insert(key(&manager), Rc::new(source)));
    Some(manager)
}

// a source forgets its chunks with the world that owned it
pub fn forget(manager: &WorldChunkManager) { SOURCES.with(|sources| sources.borrow_mut().remove(&key(manager))); }
