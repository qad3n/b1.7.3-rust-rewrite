use crate::block::block_sand::FALL_INSTANTLY;
use crate::block::{Block, Material};
use crate::world::{Chunk, IChunkProvider, World};
use crate::worldgen::{
    BiomeGenBase, MapGenBase, WorldChunkManager, WorldGenCactus, WorldGenClay, WorldGenDeadBush, WorldGenDungeons,
    WorldGenFlowers, WorldGenLakes, WorldGenLiquids, WorldGenMinable, WorldGenPumpkin, WorldGenReed, WorldGenTallGrass,
    WorldGenerator, map_gen_caves,
};
use common::noise::NoiseGeneratorOctaves;
use common::world::IProgressUpdate;
use jvm::field::Field;
use jvm::random::Random;
use jvm::throwable::OrNpe;
use std::cell::Cell;
use std::rc::Rc;

pub(super) mod chunk
{
    pub const BLOCK_COUNT: usize = 32768;
    pub const COLUMN_COUNT: usize = 256;
    pub const SEED_X: i64 = 341873128712;
    pub const SEED_Z: i64 = 132897987541;
}

pub struct ChunkProviderGenerate
{
    m_rand: Random,
    m_minLimitPerlinNoise: NoiseGeneratorOctaves,
    m_maxLimitPerlinNoise: NoiseGeneratorOctaves,
    m_perlinNoise1: NoiseGeneratorOctaves,
    m_perlinNoise2: NoiseGeneratorOctaves,
    m_perlinNoise3: NoiseGeneratorOctaves,
    pub m_scaleNoise: NoiseGeneratorOctaves,
    pub m_depthNoise: NoiseGeneratorOctaves,
    pub m_mobSpawnerNoise: NoiseGeneratorOctaves,
    m_worldObj: Rc<World>,
    m_heightMap: Field<Option<Vec<f64>>>,
    m_sandNoise: Field<Option<Vec<f64>>>,
    m_gravelNoise: Field<Option<Vec<f64>>>,
    m_stoneNoise: Field<Option<Vec<f64>>>,
    m_cave: Rc<MapGenBase>,
    m_biomesForGeneration: Field<Option<Vec<Option<Rc<BiomeGenBase>>>>>,
    m_perlinNoiseBuffer: Field<Option<Vec<f64>>>,
    m_minLimitPerlinNoiseBuffer: Field<Option<Vec<f64>>>,
    m_maxLimitPerlinNoiseBuffer: Field<Option<Vec<f64>>>,
    m_scaleNoiseBuffer: Field<Option<Vec<f64>>>,
    m_depthNoiseBuffer: Field<Option<Vec<f64>>>,
    m_generatedTemperatures: Field<Option<Vec<f64>>>,
}

impl ChunkProviderGenerate
{
    fn world(&self) -> Rc<World> { Rc::clone(&self.m_worldObj) }

    pub fn new(world: &Rc<World>, seed: i64) -> Rc<ChunkProviderGenerate>
    {
        let cave: Rc<MapGenBase> = map_gen_caves::create();
        let rand: Random = Random::withSeed(seed);
        Rc::new(ChunkProviderGenerate {
            m_minLimitPerlinNoise: NoiseGeneratorOctaves::new(&rand, 16),
            m_maxLimitPerlinNoise: NoiseGeneratorOctaves::new(&rand, 16),
            m_perlinNoise1: NoiseGeneratorOctaves::new(&rand, 8),
            m_perlinNoise2: NoiseGeneratorOctaves::new(&rand, 4),
            m_perlinNoise3: NoiseGeneratorOctaves::new(&rand, 4),
            m_scaleNoise: NoiseGeneratorOctaves::new(&rand, 10),
            m_depthNoise: NoiseGeneratorOctaves::new(&rand, 16),
            m_mobSpawnerNoise: NoiseGeneratorOctaves::new(&rand, 8),
            m_rand: rand,
            m_worldObj: Rc::clone(world),
            m_heightMap: Field::new(None),
            m_sandNoise: Field::new(Some(vec![0.0; chunk::COLUMN_COUNT])),
            m_gravelNoise: Field::new(Some(vec![0.0; chunk::COLUMN_COUNT])),
            m_stoneNoise: Field::new(Some(vec![0.0; chunk::COLUMN_COUNT])),
            m_cave: cave,
            m_biomesForGeneration: Field::new(None),
            m_perlinNoiseBuffer: Field::new(None),
            m_minLimitPerlinNoiseBuffer: Field::new(None),
            m_maxLimitPerlinNoiseBuffer: Field::new(None),
            m_scaleNoiseBuffer: Field::new(None),
            m_depthNoiseBuffer: Field::new(None),
            m_generatedTemperatures: Field::new(None),
        })
    }

    pub fn generateTerrain(
        &self,
        chunkX: i32,
        chunkZ: i32,
        blocks: &mut [i8],
        _biomes: &[Option<Rc<BiomeGenBase>>],
        temperatures: &[f64],
    )
    {
        let b0: i32 = 4;
        let b1: i32 = 64;
        let i: i32 = b0 + 1;
        let b2: i32 = 17;
        let j: i32 = b0 + 1;
        let heightMap: Vec<f64> =
            self.generateHeightMap(self.m_heightMap.take(), chunkX * b0, 0, chunkZ * b0, i, b2, j);
        let at = |index: i32| -> f64 { heightMap[index as usize] };
        for k in 0..b0
        {
            for l in 0..b0
            {
                for i1 in 0..16
                {
                    let d0: f64 = 0.125;
                    let mut d1: f64 = at(((k + 0) * j + l + 0) * b2 + i1 + 0);
                    let mut d2: f64 = at(((k + 0) * j + l + 1) * b2 + i1 + 0);
                    let mut d3: f64 = at(((k + 1) * j + l + 0) * b2 + i1 + 0);
                    let mut d4: f64 = at(((k + 1) * j + l + 1) * b2 + i1 + 0);
                    let d5: f64 = (at(((k + 0) * j + l + 0) * b2 + i1 + 1) - d1) * d0;
                    let d6: f64 = (at(((k + 0) * j + l + 1) * b2 + i1 + 1) - d2) * d0;
                    let d7: f64 = (at(((k + 1) * j + l + 0) * b2 + i1 + 1) - d3) * d0;
                    let d8: f64 = (at(((k + 1) * j + l + 1) * b2 + i1 + 1) - d4) * d0;
                    for j1 in 0..8
                    {
                        let d9: f64 = 0.25;
                        let mut d10: f64 = d1;
                        let mut d11: f64 = d2;
                        let d12: f64 = (d3 - d1) * d9;
                        let d13: f64 = (d4 - d2) * d9;
                        for k1 in 0..4
                        {
                            let mut l1: i32 = (k1 + k * 4) << 11 | (l * 4) << 7 | (i1 * 8 + j1);
                            let short1: i32 = 128;
                            let d14: f64 = 0.25;
                            let mut d15: f64 = d10;
                            let d16: f64 = (d11 - d10) * d14;
                            for i2 in 0..4
                            {
                                let d17: f64 = temperatures[((k * 4 + k1) * 16 + l * 4 + i2) as usize];
                                let mut j2: i32 = 0;
                                if i1 * 8 + j1 < b1
                                {
                                    if d17 < 0.5 && i1 * 8 + j1 >= b1 - 1
                                    {
                                        j2 = Block::ice().m_blockID;
                                    }
                                    else
                                    {
                                        j2 = Block::waterStill().m_blockID;
                                    }
                                }
                                if d15 > 0.0
                                {
                                    j2 = Block::stone().m_blockID;
                                }
                                blocks[l1 as usize] = j2 as i8;
                                l1 += short1;
                                d15 += d16;
                            }
                            d10 += d12;
                            d11 += d13;
                        }
                        d1 += d5;
                        d2 += d6;
                        d3 += d7;
                        d4 += d8;
                    }
                }
            }
        }
        self.m_heightMap.set(Some(heightMap));
    }

    pub fn replaceBlocksForBiome(
        &self,
        chunkX: i32,
        chunkZ: i32,
        blocks: &mut [i8],
        biomes: &[Option<Rc<BiomeGenBase>>],
    )
    {
        let b0: i32 = 64;
        let d0: f64 = 0.03125;
        let x: f64 = (chunkX * 16) as f64;
        let z: f64 = (chunkZ * 16) as f64;
        let sandNoise: Vec<f64> =
            self.m_perlinNoise2.generateNoiseOctaves(self.m_sandNoise.take(), x, z, 0.0, 16, 16, 1, d0, d0, 1.0);
        let gravelNoise: Vec<f64> =
            self.m_perlinNoise2.generateNoiseOctaves(self.m_gravelNoise.take(), x, 109.0134, z, 16, 1, 16, d0, 1.0, d0);
        let stoneNoise: Vec<f64> = self.m_perlinNoise3.generateNoiseOctaves(
            self.m_stoneNoise.take(),
            x,
            z,
            0.0,
            16,
            16,
            1,
            d0 * 2.0,
            d0 * 2.0,
            d0 * 2.0,
        );
        let stone: i32 = Block::stone().m_blockID;
        let sand: i32 = Block::sand().m_blockID;
        for i in 0..16
        {
            for j in 0..16
            {
                let biome: Rc<BiomeGenBase> = biomes[(i + j * 16) as usize].clone().orNpe();
                let flag: bool = sandNoise[(i + j * 16) as usize] + self.m_rand.nextDouble() * 0.2 > 0.0;
                let flag1: bool = gravelNoise[(i + j * 16) as usize] + self.m_rand.nextDouble() * 0.2 > 3.0;
                let k: i32 = (stoneNoise[(i + j * 16) as usize] / 3.0 + 3.0 + self.m_rand.nextDouble() * 0.25) as i32;
                let mut l: i32 = -1;
                let mut b1: i8 = biome.m_topBlock.get();
                let mut b2: i8 = biome.m_fillerBlock.get();
                for i1 in (0..=127).rev()
                {
                    let j1: usize = ((j * 16 + i) * 128 + i1) as usize;
                    if i1 <= 0 + self.m_rand.nextIntBounded(5)
                    {
                        blocks[j1] = Block::bedrock().m_blockID as i8;
                        continue;
                    }
                    let b3: i32 = blocks[j1] as i32;
                    if b3 == 0
                    {
                        l = -1;
                    }
                    else if b3 == stone
                    {
                        if l == -1
                        {
                            if k <= 0
                            {
                                b1 = 0;
                                b2 = stone as i8;
                            }
                            else if i1 >= b0 - 4 && i1 <= b0 + 1
                            {
                                b1 = biome.m_topBlock.get();
                                b2 = biome.m_fillerBlock.get();
                                if flag1
                                {
                                    b1 = 0;
                                }
                                if flag1
                                {
                                    b2 = Block::gravel().m_blockID as i8;
                                }
                                if flag
                                {
                                    b1 = sand as i8;
                                }
                                if flag
                                {
                                    b2 = sand as i8;
                                }
                            }
                            if i1 < b0 && b1 == 0
                            {
                                b1 = Block::waterStill().m_blockID as i8;
                            }
                            l = k;
                            blocks[j1] = if i1 >= b0 - 1 { b1 } else { b2 };
                        }
                        else if l > 0
                        {
                            l -= 1;
                            blocks[j1] = b2;
                            if l == 0 && b2 as i32 == sand
                            {
                                l = self.m_rand.nextIntBounded(4);
                                b2 = Block::sandStone().m_blockID as i8;
                            }
                        }
                    }
                }
            }
        }
        self.m_sandNoise.set(Some(sandNoise));
        self.m_gravelNoise.set(Some(gravelNoise));
        self.m_stoneNoise.set(Some(stoneNoise));
    }

    fn generateHeightMap(
        &self,
        heightMap: Option<Vec<f64>>,
        x: i32,
        y: i32,
        z: i32,
        sizeX: i32,
        sizeY: i32,
        sizeZ: i32,
    ) -> Vec<f64>
    {
        let mut heightMap: Vec<f64> = heightMap.unwrap_or_else(|| vec![0.0; (sizeX * sizeY * sizeZ) as usize]);
        let d0: f64 = 684.412;
        let d1: f64 = 684.412;
        let manager: Rc<WorldChunkManager> = self.world().getWorldChunkManager();
        let adouble: Vec<f64> = manager.m_temperature.get().orNpe();
        let adouble1: Vec<f64> = manager.m_humidity.get().orNpe();
        let scaleNoise: Vec<f64> =
            self.m_scaleNoise.getRegion(self.m_scaleNoiseBuffer.take(), x, z, sizeX, sizeZ, 1.121, 1.121, 0.5);
        let depthNoise: Vec<f64> =
            self.m_depthNoise.getRegion(self.m_depthNoiseBuffer.take(), x, z, sizeX, sizeZ, 200.0, 200.0, 0.5);
        let (fx, fy, fz): (f64, f64, f64) = (x as f64, y as f64, z as f64);
        let perlinNoise: Vec<f64> = self.m_perlinNoise1.generateNoiseOctaves(
            self.m_perlinNoiseBuffer.take(),
            fx,
            fy,
            fz,
            sizeX,
            sizeY,
            sizeZ,
            d0 / 80.0,
            d1 / 160.0,
            d0 / 80.0,
        );
        let minLimitNoise: Vec<f64> = self.m_minLimitPerlinNoise.generateNoiseOctaves(
            self.m_minLimitPerlinNoiseBuffer.take(),
            fx,
            fy,
            fz,
            sizeX,
            sizeY,
            sizeZ,
            d0,
            d1,
            d0,
        );
        let maxLimitNoise: Vec<f64> = self.m_maxLimitPerlinNoise.generateNoiseOctaves(
            self.m_maxLimitPerlinNoiseBuffer.take(),
            fx,
            fy,
            fz,
            sizeX,
            sizeY,
            sizeZ,
            d0,
            d1,
            d0,
        );
        let mut i: usize = 0;
        let mut j: usize = 0;
        let k: i32 = 16 / sizeX;
        for l in 0..sizeX
        {
            let i1: i32 = l * k + k / 2;
            for j1 in 0..sizeZ
            {
                let k1: i32 = j1 * k + k / 2;
                let d2: f64 = adouble[(i1 * 16 + k1) as usize];
                let d3: f64 = adouble1[(i1 * 16 + k1) as usize] * d2;
                let mut d4: f64 = 1.0 - d3;
                d4 *= d4;
                d4 *= d4;
                d4 = 1.0 - d4;
                let mut d5: f64 = (scaleNoise[j] + 256.0) / 512.0;
                d5 *= d4;
                if d5 > 1.0
                {
                    d5 = 1.0;
                }
                let mut d6: f64 = depthNoise[j] / 8000.0;
                if d6 < 0.0
                {
                    d6 = -d6 * 0.3;
                }
                d6 = d6 * 3.0 - 2.0;
                if d6 < 0.0
                {
                    d6 /= 2.0;
                    if d6 < -1.0
                    {
                        d6 = -1.0;
                    }
                    d6 /= 1.4;
                    d6 /= 2.0;
                    d5 = 0.0;
                }
                else
                {
                    if d6 > 1.0
                    {
                        d6 = 1.0;
                    }
                    d6 /= 8.0;
                }
                if d5 < 0.0
                {
                    d5 = 0.0;
                }
                d5 += 0.5;
                d6 = d6 * sizeY as f64 / 16.0;
                let d7: f64 = sizeY as f64 / 2.0 + d6 * 4.0;
                j += 1;
                for l1 in 0..sizeY
                {
                    let mut d9: f64 = (l1 as f64 - d7) * 12.0 / d5;
                    if d9 < 0.0
                    {
                        d9 *= 4.0;
                    }
                    let d10: f64 = minLimitNoise[i] / 512.0;
                    let d11: f64 = maxLimitNoise[i] / 512.0;
                    let d12: f64 = (perlinNoise[i] / 10.0 + 1.0) / 2.0;
                    let mut d8: f64 = if d12 < 0.0
                    {
                        d10
                    }
                    else if d12 > 1.0
                    {
                        d11
                    }
                    else
                    {
                        d10 + (d11 - d10) * d12
                    };
                    d8 -= d9;
                    if l1 > sizeY - 4
                    {
                        let d13: f64 = ((l1 - (sizeY - 4)) as f32 / 3.0f32) as f64;
                        d8 = d8 * (1.0 - d13) + -10.0 * d13;
                    }
                    heightMap[i] = d8;
                    i += 1;
                }
            }
        }
        self.m_scaleNoiseBuffer.set(Some(scaleNoise));
        self.m_depthNoiseBuffer.set(Some(depthNoise));
        self.m_perlinNoiseBuffer.set(Some(perlinNoise));
        self.m_minLimitPerlinNoiseBuffer.set(Some(minLimitNoise));
        self.m_maxLimitPerlinNoiseBuffer.set(Some(maxLimitNoise));
        heightMap
    }
}

pub(super) fn chunkSeed(chunkX: i32, chunkZ: i32) -> i64
{
    chunkX as i64 * chunk::SEED_X + chunkZ as i64 * chunk::SEED_Z
}

pub(super) fn populateOres(world: &Rc<World>, rand: &Random, chunkX: i32, chunkZ: i32)
{
    let i: i32 = chunkX * 16;
    let j: i32 = chunkZ * 16;
    rand.setSeed(world.getRandomSeed());
    let k: i64 = rand.nextLong() / 2 * 2 + 1;
    let l: i64 = rand.nextLong() / 2 * 2 + 1;
    rand.setSeed((chunkX as i64 * k + chunkZ as i64 * l) ^ world.getRandomSeed());
    if rand.nextIntBounded(4) == 0
    {
        let i1: i32 = i + rand.nextIntBounded(16) + 8;
        let j1: i32 = rand.nextIntBounded(128);
        let k1: i32 = j + rand.nextIntBounded(16) + 8;
        WorldGenLakes::new(Block::waterStill().m_blockID).generate(world, rand, i1, j1, k1);
    }
    if rand.nextIntBounded(8) == 0
    {
        let j3: i32 = i + rand.nextIntBounded(16) + 8;
        let j6: i32 = rand.nextIntBounded(rand.nextIntBounded(120) + 8);
        let j9: i32 = j + rand.nextIntBounded(16) + 8;
        if j6 < 64 || rand.nextIntBounded(10) == 0
        {
            WorldGenLakes::new(Block::lavaStill().m_blockID).generate(world, rand, j3, j6, j9);
        }
    }
    for _ in 0..8
    {
        let k6: i32 = i + rand.nextIntBounded(16) + 8;
        let k9: i32 = rand.nextIntBounded(128);
        let l1: i32 = j + rand.nextIntBounded(16) + 8;
        WorldGenDungeons.generate(world, rand, k6, k9, l1);
    }
    for _ in 0..10
    {
        let l6: i32 = i + rand.nextIntBounded(16);
        let l9: i32 = rand.nextIntBounded(128);
        let j12: i32 = j + rand.nextIntBounded(16);
        WorldGenClay::new(32).generate(world, rand, l6, l9, j12);
    }
    let ores: [(i32, i32, i32, i32); 7] = [
        (Block::dirt().m_blockID, 32, 20, 128),
        (Block::gravel().m_blockID, 32, 10, 128),
        (Block::oreCoal().m_blockID, 16, 20, 128),
        (Block::oreIron().m_blockID, 8, 20, 64),
        (Block::oreGold().m_blockID, 8, 2, 32),
        (Block::oreRedstone().m_blockID, 7, 8, 16),
        (Block::oreDiamond().m_blockID, 7, 1, 16),
    ];
    for (block, size, count, maxY) in ores
    {
        for _ in 0..count
        {
            let x: i32 = i + rand.nextIntBounded(16);
            let y: i32 = rand.nextIntBounded(maxY);
            let z: i32 = j + rand.nextIntBounded(16);
            WorldGenMinable::new(block, size).generate(world, rand, x, y, z);
        }
    }
    {
        let l8: i32 = i + rand.nextIntBounded(16);
        let l11: i32 = rand.nextIntBounded(16) + rand.nextIntBounded(16);
        let j14: i32 = j + rand.nextIntBounded(16);
        WorldGenMinable::new(Block::oreLapis().m_blockID, 6).generate(world, rand, l8, l11, j14);
    }
}

pub(super) fn populateTrees(
    world: &Rc<World>,
    rand: &Random,
    noise: &NoiseGeneratorOctaves,
    biome: &Rc<BiomeGenBase>,
    i: i32,
    j: i32,
)
{
    let is = |other: Rc<BiomeGenBase>| -> bool { Rc::ptr_eq(biome, &other) };
    let d0: f64 = 0.5;
    let i6: i32 = ((noise.getValue(i as f64 * d0, j as f64 * d0) / 8.0 + rand.nextDouble() * 4.0 + 4.0) / 3.0) as i32;
    let mut i9: i32 = 0;
    if rand.nextIntBounded(10) == 0
    {
        i9 += 1;
    }
    if is(BiomeGenBase::forest())
    {
        i9 += i6 + 5;
    }
    if is(BiomeGenBase::rainforest())
    {
        i9 += i6 + 5;
    }
    if is(BiomeGenBase::seasonalForest())
    {
        i9 += i6 + 2;
    }
    if is(BiomeGenBase::taiga())
    {
        i9 += i6 + 5;
    }
    if is(BiomeGenBase::desert())
    {
        i9 -= 20;
    }
    if is(BiomeGenBase::tundra())
    {
        i9 -= 20;
    }
    if is(BiomeGenBase::plains())
    {
        i9 -= 20;
    }
    for _ in 0..i9
    {
        let k14: i32 = i + rand.nextIntBounded(16) + 8;
        let i2: i32 = j + rand.nextIntBounded(16) + 8;
        let worldGenerator: Box<dyn WorldGenerator> = biome.getRandomWorldGenForTrees(rand);
        worldGenerator.prepare(1.0, 1.0, 1.0);
        worldGenerator.generate(world, rand, k14, world.getHeightValue(k14, i2), i2);
    }
}

pub(super) fn populateDecorations(world: &Rc<World>, rand: &Random, biome: &Rc<BiomeGenBase>, i: i32, j: i32)
{
    let flowers: [(i32, i32); 3] =
        [(Block::plantRed().m_blockID, 2), (Block::mushroomBrown().m_blockID, 4), (Block::mushroomRed().m_blockID, 8)];
    for (flower, chance) in flowers
    {
        if rand.nextIntBounded(chance) == 0
        {
            let x: i32 = i + rand.nextIntBounded(16) + 8;
            let y: i32 = rand.nextIntBounded(128);
            let z: i32 = j + rand.nextIntBounded(16) + 8;
            WorldGenFlowers::new(flower).generate(world, rand, x, y, z);
        }
    }
    for _ in 0..10
    {
        let j18: i32 = i + rand.nextIntBounded(16) + 8;
        let i21: i32 = rand.nextIntBounded(128);
        let l22: i32 = j + rand.nextIntBounded(16) + 8;
        WorldGenReed.generate(world, rand, j18, i21, l22);
    }
    if rand.nextIntBounded(32) == 0
    {
        let l16: i32 = i + rand.nextIntBounded(16) + 8;
        let k18: i32 = rand.nextIntBounded(128);
        let j21: i32 = j + rand.nextIntBounded(16) + 8;
        WorldGenPumpkin.generate(world, rand, l16, k18, j21);
    }
    let mut b2: i32 = 0;
    if Rc::ptr_eq(biome, &BiomeGenBase::desert())
    {
        b2 += 10;
    }
    for _ in 0..b2
    {
        let k21: i32 = i + rand.nextIntBounded(16) + 8;
        let i23: i32 = rand.nextIntBounded(128);
        let i24: i32 = j + rand.nextIntBounded(16) + 8;
        WorldGenCactus.generate(world, rand, k21, i23, i24);
    }
    for _ in 0..50
    {
        let l21: i32 = i + rand.nextIntBounded(16) + 8;
        let j23: i32 = rand.nextIntBounded(rand.nextIntBounded(120) + 8);
        let j24: i32 = j + rand.nextIntBounded(16) + 8;
        WorldGenLiquids::new(Block::waterMoving().m_blockID).generate(world, rand, l21, j23, j24);
    }
    for _ in 0..20
    {
        let i22: i32 = i + rand.nextIntBounded(16) + 8;
        let k23: i32 = rand.nextIntBounded(rand.nextIntBounded(rand.nextIntBounded(112) + 8) + 8);
        let k24: i32 = j + rand.nextIntBounded(16) + 8;
        WorldGenLiquids::new(Block::lavaMoving().m_blockID).generate(world, rand, i22, k23, k24);
    }
}

pub(super) fn populateSnow(world: &Rc<World>, temperatures: &Field<Option<Vec<f64>>>, i: i32, j: i32)
{
    temperatures.set(Some(world.getWorldChunkManager().getTemperatures(temperatures.take(), i + 8, j + 8, 16, 16)));
    for k19 in i + 8..i + 8 + 16
    {
        for j22 in j + 8..j + 8 + 16
        {
            let l23: i32 = k19 - (i + 8);
            let l24: i32 = j22 - (j + 8);
            let i3: i32 = world.findTopSolidBlock(k19, j22);
            let temperature: f64 = temperatures.with(|values| values.as_ref().orNpe()[(l23 * 16 + l24) as usize]);
            let d1: f64 = temperature - (i3 - 64) as f64 / 64.0 * 0.3;
            if d1 < 0.5
                && i3 > 0
                && i3 < 128
                && world.isAirBlock(k19, i3, j22)
                && world.getBlockMaterial(k19, i3 - 1, j22).getIsSolid()
                && !Rc::ptr_eq(&world.getBlockMaterial(k19, i3 - 1, j22), &Material::ice())
            {
                world.setBlockWithNotify(k19, i3, j22, Block::snow().m_blockID);
            }
        }
    }
}

pub(super) fn setFallInstantly(value: bool) { FALL_INSTANTLY.with(|flag: &Cell<bool>| flag.set(value)); }

impl IChunkProvider for ChunkProviderGenerate
{
    fn chunkExists(&self, _chunkX: i32, _chunkZ: i32) -> bool { true }

    fn provideChunk(&self, chunkX: i32, chunkZ: i32) -> Rc<Chunk>
    {
        let world: &Rc<World> = &self.world();
        self.m_rand.setSeed(chunkSeed(chunkX, chunkZ));
        let mut blocks: Vec<i8> = vec![0; chunk::BLOCK_COUNT];
        let manager: Rc<WorldChunkManager> = world.getWorldChunkManager();
        let biomes: Vec<Option<Rc<BiomeGenBase>>> =
            manager.loadBlockGeneratorData(self.m_biomesForGeneration.take(), chunkX * 16, chunkZ * 16, 16, 16);
        let temperatures: Vec<f64> = manager.m_temperature.get().orNpe();
        self.generateTerrain(chunkX, chunkZ, &mut blocks, &biomes, &temperatures);
        self.replaceBlocksForBiome(chunkX, chunkZ, &mut blocks, &biomes);
        self.m_biomesForGeneration.set(Some(biomes));
        self.m_cave.place(self, world, chunkX, chunkZ, &mut blocks);
        let chunk: Rc<Chunk> = Chunk::withBlocks(world, blocks, chunkX, chunkZ);
        chunk.populateHeightMap();
        chunk
    }

    fn prepareChunk(&self, chunkX: i32, chunkZ: i32) -> Rc<Chunk> { self.provideChunk(chunkX, chunkZ) }

    fn populate(&self, _source: &Rc<dyn IChunkProvider>, chunkX: i32, chunkZ: i32)
    {
        setFallInstantly(true);
        let world: &Rc<World> = &self.world();
        let rand: &Random = &self.m_rand;
        let i: i32 = chunkX * 16;
        let j: i32 = chunkZ * 16;
        let biome: Rc<BiomeGenBase> = world.getWorldChunkManager().getBiomeGenAt(i + 16, j + 16);
        let is = |other: Rc<BiomeGenBase>| -> bool { Rc::ptr_eq(&biome, &other) };
        populateOres(world, rand, chunkX, chunkZ);
        populateTrees(world, rand, &self.m_mobSpawnerNoise, &biome, i, j);
        let mut b0: i32 = 0;
        if is(BiomeGenBase::forest())
        {
            b0 = 2;
        }
        if is(BiomeGenBase::seasonalForest())
        {
            b0 = 4;
        }
        if is(BiomeGenBase::taiga())
        {
            b0 = 2;
        }
        if is(BiomeGenBase::plains())
        {
            b0 = 3;
        }
        for _ in 0..b0
        {
            let i15: i32 = i + rand.nextIntBounded(16) + 8;
            let i17: i32 = rand.nextIntBounded(128);
            let j2: i32 = j + rand.nextIntBounded(16) + 8;
            WorldGenFlowers::new(Block::plantYellow().m_blockID).generate(world, rand, i15, i17, j2);
        }
        let mut b1: i32 = 0;
        if is(BiomeGenBase::forest())
        {
            b1 = 2;
        }
        if is(BiomeGenBase::rainforest())
        {
            b1 = 10;
        }
        if is(BiomeGenBase::seasonalForest())
        {
            b1 = 2;
        }
        if is(BiomeGenBase::taiga())
        {
            b1 = 1;
        }
        if is(BiomeGenBase::plains())
        {
            b1 = 10;
        }
        for _ in 0..b1
        {
            let mut b3: i32 = 1;
            if is(BiomeGenBase::rainforest()) && rand.nextIntBounded(3) != 0
            {
                b3 = 2;
            }
            let l19: i32 = i + rand.nextIntBounded(16) + 8;
            let k2: i32 = rand.nextIntBounded(128);
            let l2: i32 = j + rand.nextIntBounded(16) + 8;
            WorldGenTallGrass::new(Block::tallGrass().m_blockID, b3).generate(world, rand, l19, k2, l2);
        }
        b1 = 0;
        if is(BiomeGenBase::desert())
        {
            b1 = 2;
        }
        for _ in 0..b1
        {
            let j17: i32 = i + rand.nextIntBounded(16) + 8;
            let i20: i32 = rand.nextIntBounded(128);
            let k22: i32 = j + rand.nextIntBounded(16) + 8;
            WorldGenDeadBush::new(Block::deadBush().m_blockID).generate(world, rand, j17, i20, k22);
        }
        populateDecorations(world, rand, &biome, i, j);
        populateSnow(world, &self.m_generatedTemperatures, i, j);
        setFallInstantly(false);
    }

    fn saveChunks(&self, _saveEntities: bool, _listener: Option<&dyn IProgressUpdate>) -> bool { true }

    fn unload100OldestChunks(&self) -> bool { false }

    fn canSave(&self) -> bool { true }

    fn makeString(&self) -> String { String::from("RandomLevelSource") }
}
