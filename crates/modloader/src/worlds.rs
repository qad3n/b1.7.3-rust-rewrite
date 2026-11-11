use crate::dimensions::{self, ModBiome, ModDimension};
use crate::state::ModRuntime;
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
    // terrain handlers fill the vanilla 128 high layout, a taller dimension is padded with air above it
    pub const TERRAIN_HEIGHT: usize = 128;
    pub const COLUMNS: usize = game::world::COLUMNS as usize;
}

const DEFAULT_GROUND: i32 = 64;

thread_local!
{
    // one biome per mod dimension, keyed by its identifier so a remapped numeric id never picks another's biome
    static BIOMES: RefCell<HashMap<mod_protocol::Identifier, Rc<BiomeGenBase>>> = RefCell::new(HashMap::new());
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
const MOD_BIOME: BiomeGenBaseClass = BiomeGenBaseClass
{
    m_name: "BiomeGenMod",
    m_parent: Some(&game::worldgen::BIOME_GEN_BASE),
    getSkyColorByTemp: |this, temperature|
    {
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

thread_local!
{
    // one biome object per mod biome, shared by every dimension that lists it
    static MOD_BIOMES: RefCell<HashMap<mod_protocol::Identifier, Rc<BiomeGenBase>>> = RefCell::new(HashMap::new());
}

// a mod biome as a vanilla biome object, spawning only what mods list for it under the dimension's sky
fn makeBiome(id: &mod_protocol::Identifier, sky: u32) -> Rc<BiomeGenBase>
{
    if let Some(biome) = MOD_BIOMES.with(|biomes| biomes.borrow().get(id).cloned())
    {
        return biome;
    }

    let spec: Option<ModBiome> = dimensions::biomes().into_iter().find(|biome| biome.m_id == *id);
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

    if sky != 0
    {
        SKIES.with(|skies| skies.borrow_mut().insert(biomeKey(&biome), sky as i32));
    }

    MOD_BIOMES.with(|biomes| biomes.borrow_mut().insert(id.clone(), Rc::clone(&biome)));
    biome
}

// one biome object per mod dimension, built from its biome spec or plains
fn biomeFor(dimension: &ModDimension) -> Rc<BiomeGenBase>
{
    if let Some(biome) = BIOMES.with(|biomes| biomes.borrow().get(&dimension.m_id).cloned())
    {
        return biome;
    }

    let biome: Rc<BiomeGenBase> = match dimension.m_spec.biome.as_ref()
    {
        Some(id) => makeBiome(id, dimension.m_spec.sky_color),
        None => BiomeGenBase::plains(),
    };

    if dimension.m_spec.sky_color != 0
    {
        SKIES.with(|skies| skies.borrow_mut().insert(biomeKey(&biome), dimension.m_spec.sky_color as i32));
    }

    BIOMES.with(|biomes| biomes.borrow_mut().insert(dimension.m_id.clone(), Rc::clone(&biome)));
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
    m_failed: std::cell::Cell<bool>,
}

impl FlatGenerator
{
    // asks the mod that owns the dimension's terrain for every block of the chunk
    fn guestChunk(&self, world: &Rc<World>, chunkX: i32, chunkZ: i32) -> Option<Rc<Chunk>>
    {
        let (modIndex, handler) = self.m_terrain?;
        let runtime: Rc<ModRuntime> = crate::state::modAt(modIndex)?;
        let chunk: ChunkInfo = ChunkInfo { world: self.m_dimension, x: chunkX, z: chunkZ, seed: world.getRandomSeed() };
        let reply: Option<GuestReply> =
            crate::host::context::withWorld(world, || runtime.call(&GuestCall::Terrain { handler, chunk }));

        let Some(GuestReply::Chunk { blocks, meta }) = reply
        else
        {
            return None;
        };

        let wanted: usize = chunk::TERRAIN_HEIGHT * chunk::COLUMNS;
        if blocks.len() != wanted || meta.len() != wanted
        {
            crate::log::warn(runtime.id(), &format!("terrain chunk has {} blocks, wanted {wanted}", blocks.len()));
            return None;
        }

        let blocks: Vec<u16> = pad(&blocks, world.height() as usize);
        let meta: Vec<u8> = pad(&meta, world.height() as usize);
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

        let height: i32 = world.height();
        let mut blocks: Vec<i8> = vec![0; world.volume()];
        let low: i8 = (self.m_fill & 0xFF) as i8;

        for column in 0..game::world::COLUMNS
        {
            let base: usize = (column * height) as usize;
            blocks[base] = Block::BEDROCK as i8;

            for y in 1..self.m_ground.clamp(1, height)
            {
                blocks[base + y as usize] = low;
            }
        }

        let chunk: Rc<Chunk> = Chunk::withBlocks(&world, blocks, chunkX, chunkZ);

        if self.m_fill > 0xFF
        {
            for column in 0..game::world::COLUMNS
            {
                for y in 1..self.m_ground.clamp(1, height)
                {
                    chunk.setBlockHigh(column * height + y, self.m_fill);
                }
            }
        }

        chunk.generateHeightMap();

        // a stand in for a terrain mod that failed is never saved, so the real terrain comes back once the mod works
        if self.m_terrain.is_some()
        {
            chunk.m_neverSave.set(true);

            if !self.m_failed.replace(true)
            {
                crate::log::error(
                    crate::log::LOADER,
                    &format!(
                        "dimension {} has no working terrain mod, its new chunks stay flat and unsaved",
                        self.m_dimension
                    ),
                );
            }
        }

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

pub const MOD_PROVIDER: WorldProviderClass = WorldProviderClass
{
    m_name: "WorldProviderMod",
    m_parent: Some(&WORLD_PROVIDER),
    registerWorldChunkManager: |this|
    {
        let dimension: i32 = this.m_worldType.get();
        let Some(spec) = dimensions::dimension(dimension)
        else
        {
            return (WORLD_PROVIDER.registerWorldChunkManager)(this);
        };

        let sky: u32 = spec.m_spec.sky_color;
        let sourced: Option<Rc<WorldChunkManager>> =
            crate::biomes::manager(&this.world(), &spec, |id| makeBiome(id, sky));
        this.m_worldChunkMgr
            .set(Some(sourced.unwrap_or_else(|| WorldChunkManager::newHell(biomeFor(&spec), 0.5, 0.0))));
        this.m_hasNoSky.set(!spec.m_spec.has_sky);
    },

    getChunkProvider: |this|
    {
        let world: Rc<World> = this.world();
        let flat: Option<ModDimension> = spec(this);
        let fill: i32 = blockId(flat.as_ref().and_then(|spec| spec.m_spec.fill_block.as_ref()), Block::STONE);

        let ground: i32 = flat.map_or(DEFAULT_GROUND, |spec|
        {
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

        Rc::new(FlatGenerator
        {
            m_world: Rc::downgrade(&world),
            m_fill: fill,
            m_ground: ground,
            m_dimension: dimension,
            m_terrain: terrain,
            m_failed: std::cell::Cell::new(false),
        })
    },
    getFogColor: fogColor,
    calculateCelestialAngle: |this, time, tickDelta|
    {
        let dimension: i32 = this.m_worldType.get();
        let shown: i64 = SKY_TIMES.with(|times| times.borrow().get(&dimension).copied()).unwrap_or(time);
        (WORLD_PROVIDER.calculateCelestialAngle)(this, shown, tickDelta)
    },

    canCoordinateBeSpawn: |this, x, z|
    {
        this.world().getChunkFromChunkCoords(x >> 4, z >> 4).getHeightValue(x & 15, z & 15) > 0
    },
    canRespawnHere: |this| spec(this).is_some_and(|dimension| dimension.m_spec.beds),
    ..WORLD_PROVIDER
};

pub fn provider(dimension: i32) -> Option<Rc<WorldProvider>>
{
    let spec: ModDimension = dimensions::dimension(dimension)?;
    let provider: Rc<WorldProvider> = WorldProvider::alloc(&MOD_PROVIDER);
    provider.m_worldType.set(dimension);

    let height: dimensions::Height = dimensions::height(&spec.m_id);
    provider.m_height.set(height.m_world);
    provider.m_buildLimit.set(height.m_build);
    provider.m_seed.set(dimensions::seed(&spec.m_id));
    Some(provider)
}

// a terrain handler's 128 high columns stretched to the dimension's height, air above
fn pad<T: Copy + Default>(cells: &[T], height: usize) -> Vec<T>
{
    if height == chunk::TERRAIN_HEIGHT
    {
        return cells.to_vec();
    }

    let mut padded: Vec<T> = vec![T::default(); chunk::COLUMNS * height];
    let kept: usize = chunk::TERRAIN_HEIGHT.min(height);

    for (column, cells) in cells.chunks(chunk::TERRAIN_HEIGHT).enumerate()
    {
        padded[column * height..column * height + kept].copy_from_slice(&cells[..kept]);
    }

    padded
}
