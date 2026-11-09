use crate::dimensions::{self, ModDimension};
use common::world::{IProgressUpdate, NibbleArray};
use game::block::Block;
use game::math::Vec3D;
use game::world::{Chunk, IChunkProvider, WORLD_PROVIDER, World, WorldProvider, WorldProviderClass};
use game::worldgen::{BiomeGenBase, BiomeGenBaseClass, WorldChunkManager};
use mod_protocol::abi::{ChunkInfo, GuestCall, GuestReply, Handle};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::{Rc, Weak};

mod chunk
{
    pub const BLOCK_COUNT: usize = 32768;
    pub const HEIGHT: i32 = 128;
}

const DEFAULT_GROUND: i32 = 64;

thread_local! {
    static BIOMES: RefCell<HashMap<i32, Rc<BiomeGenBase>>> = RefCell::new(HashMap::new());
    // sky color of each mod dimension biome, keyed by the biome object
    static SKIES: RefCell<HashMap<usize, i32>> = RefCell::new(HashMap::new());
    // times mods pinned a dimension's sky to, by dimension
    static SKY_TIMES: RefCell<HashMap<i32, i64>> = RefCell::new(HashMap::new());
}

pub fn setSkyTime(dimension: i32, time: Option<i64>)
{
    SKY_TIMES.with(|times| match time
    {
        Some(time) => times.borrow_mut().insert(dimension, time),
        None => times.borrow_mut().remove(&dimension),
    });
}

fn biomeKey(biome: &Rc<BiomeGenBase>) -> usize { Rc::as_ptr(biome) as usize }

// a mod dimension's biome paints the sky with the dimension's color, as BiomeGenSky does for the sky dimension
const MOD_BIOME: BiomeGenBaseClass = BiomeGenBaseClass {
    m_name: "BiomeGenMod",
    m_parent: Some(&game::worldgen::BIOME_GEN_BASE),
    getSkyColorByTemp: |this, temperature| {
        SKIES
            .with(|skies| skies.borrow().get(&biomeKey(this)).copied())
            .unwrap_or_else(|| (game::worldgen::BIOME_GEN_BASE.getSkyColorByTemp)(this, temperature))
    },
    ..game::worldgen::BIOME_GEN_BASE
};

fn spec(provider: &WorldProvider) -> Option<ModDimension> { dimensions::dimension(provider.m_worldType.get()) }

fn blockId(id: Option<&mod_protocol::Identifier>, fallback: i32) -> i32
{
    id.and_then(|id| crate::registry::with(|registries| registries.blockId(id))).unwrap_or(fallback)
}

// one biome object per mod dimension, built from its biome spec or plains
fn biomeFor(dimension: &ModDimension) -> Rc<BiomeGenBase>
{
    if let Some(biome) = BIOMES.with(|biomes| biomes.borrow().get(&dimension.m_numeric).cloned())
    {
        return biome;
    }
    let spec =
        dimension.m_spec.biome.as_ref().and_then(|id| dimensions::biomes().into_iter().find(|biome| biome.m_id == *id));
    let biome: Rc<BiomeGenBase> = match spec
    {
        Some(modBiome) =>
        {
            let biome: Rc<BiomeGenBase> = BiomeGenBase::alloc(&MOD_BIOME);
            biome.m_biomeName.set(Box::leak(modBiome.m_spec.name.clone().into_boxed_str()));
            biome.m_topBlock.set(blockId(modBiome.m_spec.top_block.as_ref(), Block::GRASS) as i8);
            biome.m_fillerBlock.set(blockId(modBiome.m_spec.filler_block.as_ref(), Block::DIRT) as i8);
            // a mod biome spawns only what mods list for it, never the vanilla default animals and monsters
            biome.m_spawnableMonsterList.clear();
            biome.m_spawnableCreatureList.clear();
            biome.m_spawnableWaterCreatureList.clear();
            crate::entities::addSpawns(&biome, &modBiome.m_id);
            biome
        }
        None => BiomeGenBase::plains(),
    };
    if dimension.m_spec.sky_color != 0
    {
        SKIES.with(|skies| skies.borrow_mut().insert(biomeKey(&biome), dimension.m_spec.sky_color as i32));
    }
    BIOMES.with(|biomes| biomes.borrow_mut().insert(dimension.m_numeric, Rc::clone(&biome)));
    biome
}

// fills each chunk with the dimension's block up to its ground height on bedrock, mods add the rest when populating
struct FlatGenerator
{
    m_world: Weak<World>,
    m_fill: i32,
    m_ground: i32,
    m_dimension: i32,
    m_terrain: Option<(usize, Handle)>,
}

impl FlatGenerator
{
    // asks the mod that owns the dimension's terrain for every block of the chunk
    fn guestChunk(&self, world: &Rc<World>, chunkX: i32, chunkZ: i32) -> Option<Rc<Chunk>>
    {
        let (modIndex, handler) = self.m_terrain?;
        let runtime = crate::state::modAt(modIndex)?;
        let chunk: ChunkInfo = ChunkInfo { world: self.m_dimension, x: chunkX, z: chunkZ, seed: world.getRandomSeed() };
        let reply: Option<GuestReply> =
            crate::host::context::withWorld(world, || runtime.call(&GuestCall::Terrain { handler, chunk }));
        let Some(GuestReply::Chunk { blocks, meta }) = reply
        else
        {
            return None;
        };
        if blocks.len() != chunk::BLOCK_COUNT || meta.len() != chunk::BLOCK_COUNT
        {
            crate::log::warn(
                runtime.id(),
                &format!("terrain chunk has {} blocks, wanted {}", blocks.len(), chunk::BLOCK_COUNT),
            );
            return None;
        }
        let low: Vec<i8> = blocks.iter().map(|block| (*block & 0xFF) as i8).collect();
        let result: Rc<Chunk> = Chunk::withBlocks(world, low, chunkX, chunkZ);
        for (index, block) in blocks.iter().enumerate()
        {
            if *block > 0xFF
            {
                result.setBlockHigh(index as i32, *block as i32);
            }
        }
        let packed: Vec<i8> = meta.chunks(2).map(|pair| ((pair[0] & 0xF) | (pair[1] & 0xF) << 4) as i8).collect();
        result.m_data.set(Some(Rc::new(NibbleArray::withData(packed))));
        result.generateHeightMap();
        Some(result)
    }
}

impl IChunkProvider for FlatGenerator
{
    fn chunkExists(&self, _chunkX: i32, _chunkZ: i32) -> bool { true }

    fn provideChunk(&self, chunkX: i32, chunkZ: i32) -> Rc<Chunk>
    {
        let world: Rc<World> = self.m_world.upgrade().expect("a generator lives inside its world");
        if let Some(chunk) = self.guestChunk(&world, chunkX, chunkZ)
        {
            return chunk;
        }
        let mut blocks: Vec<i8> = vec![0; chunk::BLOCK_COUNT];
        let low: i8 = (self.m_fill & 0xFF) as i8;
        for column in 0..256_i32
        {
            let base: usize = (column << 7) as usize;
            blocks[base] = Block::BEDROCK as i8;
            for y in 1..self.m_ground.clamp(1, chunk::HEIGHT)
            {
                blocks[base + y as usize] = low;
            }
        }
        let chunk: Rc<Chunk> = Chunk::withBlocks(&world, blocks, chunkX, chunkZ);
        if self.m_fill > 0xFF
        {
            for column in 0..256_i32
            {
                for y in 1..self.m_ground.clamp(1, chunk::HEIGHT)
                {
                    chunk.setBlockHigh(column << 7 | y, self.m_fill);
                }
            }
        }
        chunk.generateHeightMap();
        chunk
    }

    fn prepareChunk(&self, chunkX: i32, chunkZ: i32) -> Rc<Chunk> { self.provideChunk(chunkX, chunkZ) }

    fn populate(&self, _source: &Rc<dyn IChunkProvider>, chunkX: i32, chunkZ: i32)
    {
        if let Some(world) = self.m_world.upgrade()
        {
            game::hooks::populate(&world, chunkX, chunkZ);
        }
    }

    fn saveChunks(&self, _saveEntities: bool, _listener: Option<&dyn IProgressUpdate>) -> bool { true }

    fn unload100OldestChunks(&self) -> bool { false }

    fn canSave(&self) -> bool { true }

    fn makeString(&self) -> String { "ModFlatLevelSource".to_owned() }
}

// a dimension with a sky dims its fog color through the day like the overworld, one without keeps it flat
fn fogColor(this: &Rc<WorldProvider>, timeOfDay: f32, tickDelta: f32) -> Rc<Vec3D>
{
    match spec(this).filter(|dimension| dimension.m_spec.fog_color != 0)
    {
        Some(dimension) =>
        {
            let color: u32 = dimension.m_spec.fog_color;
            let channel = |shift: u32| ((color >> shift) & 0xFF) as f32 / 255.0;
            let daylight: f32 = if dimension.m_spec.has_sky
            {
                (game::math::math_helper::cos(timeOfDay * std::f32::consts::PI * 2.0) * 2.0 + 0.5).clamp(0.0, 1.0)
            }
            else
            {
                1.0
            };
            Vec3D::createVector(
                (channel(16) * (daylight * 0.94 + 0.06)) as f64,
                (channel(8) * (daylight * 0.94 + 0.06)) as f64,
                (channel(0) * (daylight * 0.91 + 0.09)) as f64,
            )
        }
        None => (WORLD_PROVIDER.getFogColor)(this, timeOfDay, tickDelta),
    }
}

pub const MOD_PROVIDER: WorldProviderClass = WorldProviderClass {
    m_name: "WorldProviderMod",
    m_parent: Some(&WORLD_PROVIDER),
    registerWorldChunkManager: |this| {
        let dimension: i32 = this.m_worldType.get();
        let Some(spec) = dimensions::dimension(dimension)
        else
        {
            return (WORLD_PROVIDER.registerWorldChunkManager)(this);
        };
        this.m_worldChunkMgr.set(Some(WorldChunkManager::newHell(biomeFor(&spec), 0.5, 0.0)));
        this.m_hasNoSky.set(!spec.m_spec.has_sky);
    },
    getChunkProvider: |this| {
        let world: Rc<World> = this.world();
        let flat: Option<ModDimension> = spec(this);
        let fill: i32 = blockId(flat.as_ref().and_then(|spec| spec.m_spec.fill_block.as_ref()), Block::STONE);
        let ground: i32 = flat.map_or(DEFAULT_GROUND, |spec| {
            if spec.m_spec.ground_height > 0
            {
                spec.m_spec.ground_height
            }
            else
            {
                DEFAULT_GROUND
            }
        });
        let dimension: i32 = this.m_worldType.get();
        let terrain: Option<(usize, Handle)> = spec(this).and_then(|spec| dimensions::terrain(&spec.m_id));
        Rc::new(FlatGenerator {
            m_world: Rc::downgrade(&world),
            m_fill: fill,
            m_ground: ground,
            m_dimension: dimension,
            m_terrain: terrain,
        })
    },
    getFogColor: fogColor,
    calculateCelestialAngle: |this, time, tickDelta| {
        let dimension: i32 = this.m_worldType.get();
        let shown: i64 = SKY_TIMES.with(|times| times.borrow().get(&dimension).copied()).unwrap_or(time);
        (WORLD_PROVIDER.calculateCelestialAngle)(this, shown, tickDelta)
    },
    canCoordinateBeSpawn: |this, x, z| {
        this.world().getChunkFromChunkCoords(x >> 4, z >> 4).getHeightValue(x & 15, z & 15) > 0
    },
    canRespawnHere: |this| spec(this).is_some_and(|dimension| dimension.m_spec.beds),
    ..WORLD_PROVIDER
};

pub fn provider(dimension: i32) -> Option<Rc<WorldProvider>>
{
    dimensions::dimension(dimension)?;
    let provider: Rc<WorldProvider> = WorldProvider::alloc(&MOD_PROVIDER);
    provider.m_worldType.set(dimension);
    Some(provider)
}
