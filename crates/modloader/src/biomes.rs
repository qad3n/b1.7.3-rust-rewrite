use crate::dimensions::{self, ModDimension};
use game::world::World;
use game::worldgen::{BiomeGenBase, WORLD_CHUNK_MANAGER_HELL, WorldChunkManager, WorldChunkManagerClass};
use mod_protocol::Identifier;
use mod_protocol::abi::{ChunkInfo, GuestCall, GuestReply, Handle};
use std::cell::{RefCell, RefMut};
use std::collections::{BTreeMap, HashMap};
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
    m_id: Identifier,
    m_world: Weak<World>,
    m_ids: Vec<Identifier>,
    m_biomes: Vec<Rc<BiomeGenBase>>,
    m_climates: Vec<(f64, f64)>,
    m_chunks: RefCell<HashMap<(i32, i32), Rc<Columns>>>,
}

// a column a mod turned to another biome, by dimension id and block x and z
type Column = (Identifier, i32, i32);

thread_local!
{
    // sources by the address of the chunk manager they answer for
    static SOURCES: RefCell<HashMap<usize, Rc<Source>>> = RefCell::new(HashMap::new());
    // columns mods turned to other biomes, saved with the world and sent to every client
    static OVERRIDES: RefCell<BTreeMap<Column, Identifier>> = const { RefCell::new(BTreeMap::new()) };
}

const OVERRIDES_FILE: &str = "biomes.dat";

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

        let reply: Option<GuestReply> = crate::host::context::withWorld(&world, ||
        {
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

        let mut asked: Columns = self.ask(chunkX, chunkZ);
        self.overlay(chunkX, chunkZ, &mut asked);

        let columns: Rc<Columns> = Rc::new(asked);
        let mut chunks: RefMut<HashMap<(i32, i32), Rc<Columns>>> = self.m_chunks.borrow_mut();
        if chunks.len() >= CACHE_LIMIT
        {
            chunks.clear();
        }

        chunks.insert((chunkX, chunkZ), Rc::clone(&columns));
        columns
    }

    // the columns of a chunk that mods turned, laid over what the source said
    fn overlay(&self, chunkX: i32, chunkZ: i32, columns: &mut Columns)
    {
        let (x0, z0): (i32, i32) = (chunkX * 16, chunkZ * 16);
        let from: Column = (self.m_id.clone(), x0, i32::MIN);
        let to: Column = (self.m_id.clone(), x0 + 15, i32::MAX);

        OVERRIDES.with(|overrides|
        {
            for ((_, x, z), biome) in overrides.borrow().range(from..=to)
            {
                let Some(index) = self.m_ids.iter().position(|id| id == biome).filter(|_| (z0..z0 + 16).contains(z))
                else
                {
                    continue;
                };

                columns.m_biomes[((z - z0) * 16 + (x - x0)) as usize] = index as u8;
            }
        });
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
pub const MOD_BIOME_SOURCE: WorldChunkManagerClass = WorldChunkManagerClass
{
    m_name: "WorldChunkManagerMod",
    m_parent: Some(&WORLD_CHUNK_MANAGER_HELL),
    getBiomeGenAtChunkCoord: |this, pos|
    {
        with(
            this,
            |source| source.biome(pos.m_chunkXPos << 4, pos.m_chunkZPos << 4),
            || (WORLD_CHUNK_MANAGER_HELL.getBiomeGenAtChunkCoord)(this, pos),
        )
    },

    getBiomeGenAt: |this, x, z|
    {
        with(this, |source| source.biome(x, z), || (WORLD_CHUNK_MANAGER_HELL.getBiomeGenAt)(this, x, z))
    },

    getTemperature: |this, x, z|
    {
        with(this, |source| source.climate(x, z).0, || (WORLD_CHUNK_MANAGER_HELL.getTemperature)(this, x, z))
    },

    getTemperatures: |this, temperatures, x, z, sizeX, sizeZ|
    {
        with(
            this,
            |source|
            {
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
        .map(|id|
        {
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

    let source: Source = Source
    {
        m_mod: modIndex,
        m_handler: handler,
        m_dimension: dimension.m_numeric,
        m_id: dimension.m_id.clone(),
        m_world: Rc::downgrade(world),
        m_ids: ids,
        m_biomes: biomes,
        m_climates: climates,
        m_chunks: RefCell::new(HashMap::new()),
    };
    SOURCES.with(|sources| sources.borrow_mut().insert(key(&manager), Rc::new(source)));
    Some(manager)
}

// the biome a column holds by name, a mod dimension's from its source and the overworld's from the vanilla rows
pub fn biomeAt(world: &World, x: i32, z: i32) -> Option<Identifier>
{
    let manager: Rc<WorldChunkManager> = world.getWorldChunkManager();
    let biome: Rc<BiomeGenBase> = manager.getBiomeGenAt(x, z);

    if let Some(source) = source(&manager)
    {
        let index: usize = source.m_biomes.iter().position(|known| Rc::ptr_eq(known, &biome))?;
        return source.m_ids.get(index).cloned();
    }

    crate::registry::BIOMES
        .iter()
        .find(|(_, vanilla)| Rc::ptr_eq(&vanilla(), &biome))
        .map(|(name, _)| Identifier::minecraft(name))
}

// a mod turns one column of its dimension to another biome of the dimension's source, on the side owning the world
pub fn setBiome(world: &Rc<World>, x: i32, z: i32, biome: Identifier) -> Result<(), String>
{
    let numeric: i32 = crate::host::context::dimension(world);
    let dimension: ModDimension =
        dimensions::dimension(numeric).ok_or_else(|| format!("dimension {numeric} has no biome source"))?;
    let listed: bool = dimensions::biomeSource(&dimension.m_id).is_some_and(|(_, _, ids)| ids.contains(&biome));
    if !listed
    {
        return Err(format!("{biome} is not a biome of {}", dimension.m_id));
    }

    turn(dimension.m_id.clone(), x, z, biome.clone());

    let message =
        crate::network::loader::BiomeColumn { dimension: dimension.m_id.to_string(), x, z, biome: biome.to_string() };

    for player in world.m_playerEntities.toVec()
    {
        crate::network::loader::send(crate::network::loader::BIOME, &player, &message);
    }

    Ok(())
}

// records a turned column, drops the chunks that held its old biome and redraws it where a client sees it
pub fn turn(dimension: Identifier, x: i32, z: i32, biome: Identifier)
{
    OVERRIDES.with(|overrides| overrides.borrow_mut().insert((dimension.clone(), x, z), biome));

    let sources: Vec<Rc<Source>> =
        SOURCES.with(|sources| sources.borrow().values().filter(|source| source.m_id == dimension).cloned().collect());

    for source in sources
    {
        source.m_chunks.borrow_mut().remove(&(x >> 4, z >> 4));

        if let Some(world) = source.m_world.upgrade()
        {
            world.markBlocksDirty(x, 0, z, x, world.m_height.get() - 1, z);
        }
    }
}

// every turned column, for a client that just joined
pub fn columns() -> Vec<crate::network::loader::BiomeColumn>
{
    OVERRIDES.with(|overrides|
    {
        overrides
            .borrow()
            .iter()
            .map(|((dimension, x, z), biome)| crate::network::loader::BiomeColumn
            {
                dimension: dimension.to_string(),
                x: *x,
                z: *z,
                biome: biome.to_string(),
            })
            .collect()
    })
}

pub fn load(world: &World)
{
    let rows: Vec<(String, i32, i32, String)> = crate::persist::read(world, OVERRIDES_FILE)
        .and_then(|bytes| crate::persist::decode(OVERRIDES_FILE, &bytes))
        .unwrap_or_default();
    let loaded: BTreeMap<Column, Identifier> = rows
        .into_iter()
        .filter_map(|(dimension, x, z, biome)|
        {
            Some(((Identifier::parse(&dimension).ok()?, x, z), Identifier::parse(&biome).ok()?))
        })
        .collect();
    OVERRIDES.with(|overrides| *overrides.borrow_mut() = loaded);
}

pub fn save(world: &World)
{
    let rows: Vec<(String, i32, i32, String)> = OVERRIDES.with(|overrides|
    {
        overrides
            .borrow()
            .iter()
            .map(|((dimension, x, z), biome)| (dimension.to_string(), *x, *z, biome.to_string()))
            .collect()
    });
    if let Err(error) = crate::persist::write(world, OVERRIDES_FILE, &mod_protocol::wire::encode(&rows))
    {
        crate::log::error(crate::log::LOADER, &format!("could not save turned biome columns: {error}"));
    }
}

// a source forgets its chunks with the world that owned it
pub fn forget(manager: &WorldChunkManager) { SOURCES.with(|sources| sources.borrow_mut().remove(&key(manager))); }
