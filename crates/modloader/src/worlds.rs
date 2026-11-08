use crate::dimensions::{self, ModDimension};
use common::world::IProgressUpdate;
use game::block::Block;
use game::math::Vec3D;
use game::world::{Chunk, IChunkProvider, WORLD_PROVIDER, World, WorldProvider, WorldProviderClass};
use game::worldgen::{BiomeGenBase, WorldChunkManager};
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
}

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
            let biome: Rc<BiomeGenBase> = BiomeGenBase::alloc(&game::worldgen::BIOME_GEN_BASE);
            biome.m_biomeName.set(Box::leak(modBiome.m_spec.name.clone().into_boxed_str()));
            biome.m_topBlock.set(blockId(modBiome.m_spec.top_block.as_ref(), Block::grass().m_blockID) as i8);
            biome.m_fillerBlock.set(blockId(modBiome.m_spec.filler_block.as_ref(), Block::dirt().m_blockID) as i8);
            biome
        }
        None => BiomeGenBase::plains(),
    };
    BIOMES.with(|biomes| biomes.borrow_mut().insert(dimension.m_numeric, Rc::clone(&biome)));
    biome
}

// fills each chunk with the dimension's block up to its ground height on bedrock, mods add the rest when populating
struct FlatGenerator
{
    m_world: Weak<World>,
    m_fill: i32,
    m_ground: i32,
}

impl IChunkProvider for FlatGenerator
{
    fn chunkExists(&self, _chunkX: i32, _chunkZ: i32) -> bool { true }

    fn provideChunk(&self, chunkX: i32, chunkZ: i32) -> Rc<Chunk>
    {
        let world: Rc<World> = self.m_world.upgrade().expect("a generator lives inside its world");
        let mut blocks: Vec<i8> = vec![0; chunk::BLOCK_COUNT];
        let low: i8 = (self.m_fill & 0xFF) as i8;
        for column in 0..256_i32
        {
            let base: usize = (column << 7) as usize;
            blocks[base] = Block::bedrock().m_blockID as i8;
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

fn fogColor(this: &Rc<WorldProvider>, timeOfDay: f32, tickDelta: f32) -> Rc<Vec3D>
{
    match spec(this).filter(|dimension| dimension.m_spec.fog_color != 0)
    {
        Some(dimension) =>
        {
            let color: u32 = dimension.m_spec.fog_color;
            let channel = |shift: u32| ((color >> shift) & 0xFF) as f64 / 255.0;
            Vec3D::createVector(channel(16), channel(8), channel(0))
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
        let spec: Option<ModDimension> = spec(this);
        let fill: i32 =
            blockId(spec.as_ref().and_then(|spec| spec.m_spec.fill_block.as_ref()), Block::stone().m_blockID);
        let ground: i32 = spec.map_or(DEFAULT_GROUND, |spec| {
            if spec.m_spec.ground_height > 0
            {
                spec.m_spec.ground_height
            }
            else
            {
                DEFAULT_GROUND
            }
        });
        Rc::new(FlatGenerator { m_world: Rc::downgrade(&world), m_fill: fill, m_ground: ground })
    },
    getFogColor: fogColor,
    canCoordinateBeSpawn: |this, x, z| {
        this.world().getChunkFromChunkCoords(x >> 4, z >> 4).getHeightValue(x & 15, z & 15) > 0
    },
    canRespawnHere: |_| false,
    ..WORLD_PROVIDER
};

pub fn provider(dimension: i32) -> Option<Rc<WorldProvider>>
{
    dimensions::dimension(dimension)?;
    let provider: Rc<WorldProvider> = WorldProvider::alloc(&MOD_PROVIDER);
    provider.m_worldType.set(dimension);
    Some(provider)
}
