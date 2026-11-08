use crate::block::Block;
use crate::world::{Chunk, IChunkProvider, World};
use crate::worldgen::chunk_provider_generate::{
    chunk, chunkSeed, populateDecorations, populateOres, populateSnow, populateTrees, setFallInstantly,
};
use crate::worldgen::{BiomeGenBase, MapGenBase, WorldChunkManager, WorldGenFlowers, WorldGenerator, map_gen_caves};
use common::noise::NoiseGeneratorOctaves;
use common::world::IProgressUpdate;
use jvm::field::Field;
use jvm::random::Random;
use jvm::throwable::OrNpe;
use std::rc::Rc;

pub struct ChunkProviderSky
{
    m_random: Random,
    m_minLimitPerlinNoise: NoiseGeneratorOctaves,
    m_maxLimitPerlinNoise: NoiseGeneratorOctaves,
    m_perlinNoise1: NoiseGeneratorOctaves,
    m_perlinNoise2: NoiseGeneratorOctaves,
    m_perlinNoise3: NoiseGeneratorOctaves,
    pub m_scaleNoise: NoiseGeneratorOctaves,
    pub m_depthNoise: NoiseGeneratorOctaves,
    pub m_forestNoise: NoiseGeneratorOctaves,
    m_world: Rc<World>,
    m_heightMap: Field<Option<Vec<f64>>>,
    m_sandBuffer: Field<Option<Vec<f64>>>,
    m_gravelBuffer: Field<Option<Vec<f64>>>,
    m_depthBuffer: Field<Option<Vec<f64>>>,
    m_cave: Rc<MapGenBase>,
    m_biomes: Field<Option<Vec<Option<Rc<BiomeGenBase>>>>>,
    m_perlinNoiseBuffer: Field<Option<Vec<f64>>>,
    m_minLimitPerlinNoiseBuffer: Field<Option<Vec<f64>>>,
    m_maxLimitPerlinNoiseBuffer: Field<Option<Vec<f64>>>,
    m_scaleNoiseBuffer: Field<Option<Vec<f64>>>,
    m_depthNoiseBuffer: Field<Option<Vec<f64>>>,
    m_temperatures: Field<Option<Vec<f64>>>,
}

impl ChunkProviderSky
{
    fn world(&self) -> Rc<World> { Rc::clone(&self.m_world) }

    pub fn new(world: &Rc<World>, seed: i64) -> Rc<ChunkProviderSky>
    {
        let cave: Rc<MapGenBase> = map_gen_caves::create();
        let random: Random = Random::withSeed(seed);
        Rc::new(ChunkProviderSky {
            m_minLimitPerlinNoise: NoiseGeneratorOctaves::new(&random, 16),
            m_maxLimitPerlinNoise: NoiseGeneratorOctaves::new(&random, 16),
            m_perlinNoise1: NoiseGeneratorOctaves::new(&random, 8),
            m_perlinNoise2: NoiseGeneratorOctaves::new(&random, 4),
            m_perlinNoise3: NoiseGeneratorOctaves::new(&random, 4),
            m_scaleNoise: NoiseGeneratorOctaves::new(&random, 10),
            m_depthNoise: NoiseGeneratorOctaves::new(&random, 16),
            m_forestNoise: NoiseGeneratorOctaves::new(&random, 8),
            m_random: random,
            m_world: Rc::clone(world),
            m_heightMap: Field::new(None),
            m_sandBuffer: Field::new(Some(vec![0.0; chunk::COLUMN_COUNT])),
            m_gravelBuffer: Field::new(Some(vec![0.0; chunk::COLUMN_COUNT])),
            m_depthBuffer: Field::new(Some(vec![0.0; chunk::COLUMN_COUNT])),
            m_cave: cave,
            m_biomes: Field::new(None),
            m_perlinNoiseBuffer: Field::new(None),
            m_minLimitPerlinNoiseBuffer: Field::new(None),
            m_maxLimitPerlinNoiseBuffer: Field::new(None),
            m_scaleNoiseBuffer: Field::new(None),
            m_depthNoiseBuffer: Field::new(None),
            m_temperatures: Field::new(None),
        })
    }

    pub fn buildTerrain(
        &self,
        chunkX: i32,
        chunkZ: i32,
        blocks: &mut [i8],
        _biomes: &[Option<Rc<BiomeGenBase>>],
        _temperatures: &[f64],
    )
    {
        let n: i32 = 2;
        let n2: i32 = n + 1;
        let n3: i32 = 33;
        let n4: i32 = n + 1;
        let heightMap: Vec<f64> =
            self.generateHeightMap(self.m_heightMap.take(), chunkX * n, 0, chunkZ * n, n2, n3, n4);
        let at = |index: i32| -> f64 { heightMap[index as usize] };
        for i in 0..n
        {
            for j in 0..n
            {
                for k in 0..32
                {
                    let d: f64 = 0.25;
                    let mut d2: f64 = at(((i + 0) * n4 + (j + 0)) * n3 + (k + 0));
                    let mut d3: f64 = at(((i + 0) * n4 + (j + 1)) * n3 + (k + 0));
                    let mut d4: f64 = at(((i + 1) * n4 + (j + 0)) * n3 + (k + 0));
                    let mut d5: f64 = at(((i + 1) * n4 + (j + 1)) * n3 + (k + 0));
                    let d6: f64 = (at(((i + 0) * n4 + (j + 0)) * n3 + (k + 1)) - d2) * d;
                    let d7: f64 = (at(((i + 0) * n4 + (j + 1)) * n3 + (k + 1)) - d3) * d;
                    let d8: f64 = (at(((i + 1) * n4 + (j + 0)) * n3 + (k + 1)) - d4) * d;
                    let d9: f64 = (at(((i + 1) * n4 + (j + 1)) * n3 + (k + 1)) - d5) * d;
                    for i2 in 0..4
                    {
                        let d10: f64 = 0.125;
                        let mut d11: f64 = d2;
                        let mut d12: f64 = d3;
                        let d13: f64 = (d4 - d2) * d10;
                        let d14: f64 = (d5 - d3) * d10;
                        for i3 in 0..8
                        {
                            let mut n5: i32 = (i3 + i * 8) << 11 | (j * 8) << 7 | (k * 4 + i2);
                            let n6: i32 = 128;
                            let d15: f64 = 0.125;
                            let mut d16: f64 = d11;
                            let d17: f64 = (d12 - d11) * d15;
                            for _ in 0..8
                            {
                                let mut n7: i32 = 0;
                                if d16 > 0.0
                                {
                                    n7 = Block::stone().m_blockID;
                                }
                                blocks[n5 as usize] = n7 as i8;
                                n5 += n6;
                                d16 += d17;
                            }
                            d11 += d13;
                            d12 += d14;
                        }
                        d2 += d6;
                        d3 += d7;
                        d4 += d8;
                        d5 += d9;
                    }
                }
            }
        }
        self.m_heightMap.set(Some(heightMap));
    }

    pub fn buildSurfaces(&self, chunkX: i32, chunkZ: i32, blocks: &mut [i8], biomes: &[Option<Rc<BiomeGenBase>>])
    {
        let d0: f64 = 0.03125;
        let x: f64 = (chunkX * 16) as f64;
        let z: f64 = (chunkZ * 16) as f64;
        let sandBuffer: Vec<f64> =
            self.m_perlinNoise2.generateNoiseOctaves(self.m_sandBuffer.take(), x, z, 0.0, 16, 16, 1, d0, d0, 1.0);
        let gravelBuffer: Vec<f64> = self.m_perlinNoise2.generateNoiseOctaves(
            self.m_gravelBuffer.take(),
            x,
            109.0134,
            z,
            16,
            1,
            16,
            d0,
            1.0,
            d0,
        );
        let depthBuffer: Vec<f64> = self.m_perlinNoise3.generateNoiseOctaves(
            self.m_depthBuffer.take(),
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
                let k: i32 =
                    (depthBuffer[(i + j * 16) as usize] / 3.0 + 3.0 + self.m_random.nextDouble() * 0.25) as i32;
                let mut l: i32 = -1;
                let mut b0: i8 = biome.m_topBlock.get();
                let mut b1: i8 = biome.m_fillerBlock.get();
                for i1 in (0..=127).rev()
                {
                    let j1: usize = ((j * 16 + i) * 128 + i1) as usize;
                    let b2: i32 = blocks[j1] as i32;
                    if b2 == 0
                    {
                        l = -1;
                    }
                    else if b2 == stone
                    {
                        if l == -1
                        {
                            if k <= 0
                            {
                                b0 = 0;
                                b1 = stone as i8;
                            }
                            l = k;
                            blocks[j1] = if i1 >= 0 { b0 } else { b1 };
                        }
                        else if l > 0
                        {
                            l -= 1;
                            blocks[j1] = b1;
                            if l == 0 && b1 as i32 == sand
                            {
                                l = self.m_random.nextIntBounded(4);
                                b1 = Block::sandStone().m_blockID as i8;
                            }
                        }
                    }
                }
            }
        }
        self.m_sandBuffer.set(Some(sandBuffer));
        self.m_gravelBuffer.set(Some(gravelBuffer));
        self.m_depthBuffer.set(Some(depthBuffer));
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
        let mut d0: f64 = 684.412;
        let d1: f64 = 684.412;
        let manager: Rc<WorldChunkManager> = self.world().getWorldChunkManager();
        let adouble: Vec<f64> = manager.m_temperature.get().orNpe();
        let adouble1: Vec<f64> = manager.m_humidity.get().orNpe();
        let scaleNoise: Vec<f64> =
            self.m_scaleNoise.getRegion(self.m_scaleNoiseBuffer.take(), x, z, sizeX, sizeZ, 1.121, 1.121, 0.5);
        self.m_depthNoiseBuffer.set(Some(self.m_depthNoise.getRegion(
            self.m_depthNoiseBuffer.take(),
            x,
            z,
            sizeX,
            sizeZ,
            200.0,
            200.0,
            0.5,
        )));
        d0 *= 2.0;
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
                if d5 < 0.0
                {
                    d5 = 0.0;
                }
                d5 += 0.5;
                j += 1;
                let d7: f64 = sizeY as f64 / 2.0;
                for l1 in 0..sizeY
                {
                    let mut d9: f64 = (l1 as f64 - d7) * 8.0 / d5;
                    if d9 < 0.0
                    {
                        d9 *= -1.0;
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
                    d8 -= 8.0;
                    let mut b0: i32 = 32;
                    if l1 > sizeY - b0
                    {
                        let d13: f64 = ((l1 - (sizeY - b0)) as f32 / (b0 as f32 - 1.0f32)) as f64;
                        d8 = d8 * (1.0 - d13) + -30.0 * d13;
                    }
                    b0 = 8;
                    if l1 < b0
                    {
                        let d14: f64 = ((b0 - l1) as f32 / (b0 as f32 - 1.0f32)) as f64;
                        d8 = d8 * (1.0 - d14) + -30.0 * d14;
                    }
                    heightMap[i] = d8;
                    i += 1;
                }
            }
        }
        self.m_scaleNoiseBuffer.set(Some(scaleNoise));
        self.m_perlinNoiseBuffer.set(Some(perlinNoise));
        self.m_minLimitPerlinNoiseBuffer.set(Some(minLimitNoise));
        self.m_maxLimitPerlinNoiseBuffer.set(Some(maxLimitNoise));
        heightMap
    }
}

impl IChunkProvider for ChunkProviderSky
{
    fn chunkExists(&self, _chunkX: i32, _chunkZ: i32) -> bool { true }

    fn provideChunk(&self, chunkX: i32, chunkZ: i32) -> Rc<Chunk>
    {
        let world: &Rc<World> = &self.world();
        self.m_random.setSeed(chunkSeed(chunkX, chunkZ));
        let mut blocks: Vec<i8> = vec![0; chunk::BLOCK_COUNT];
        let manager: Rc<WorldChunkManager> = world.getWorldChunkManager();
        let biomes: Vec<Option<Rc<BiomeGenBase>>> =
            manager.loadBlockGeneratorData(self.m_biomes.take(), chunkX * 16, chunkZ * 16, 16, 16);
        let temperatures: Vec<f64> = manager.m_temperature.get().orNpe();
        self.buildTerrain(chunkX, chunkZ, &mut blocks, &biomes, &temperatures);
        self.buildSurfaces(chunkX, chunkZ, &mut blocks, &biomes);
        self.m_biomes.set(Some(biomes));
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
        let random: &Random = &self.m_random;
        let i: i32 = chunkX * 16;
        let j: i32 = chunkZ * 16;
        let biome: Rc<BiomeGenBase> = world.getWorldChunkManager().getBiomeGenAt(i + 16, j + 16);
        populateOres(world, random, chunkX, chunkZ);
        populateTrees(world, random, &self.m_forestNoise, &biome, i, j);
        for _ in 0..2
        {
            let l15: i32 = i + random.nextIntBounded(16) + 8;
            let j18: i32 = random.nextIntBounded(128);
            let l20: i32 = j + random.nextIntBounded(16) + 8;
            WorldGenFlowers::new(Block::plantYellow().m_blockID).generate(world, random, l15, j18, l20);
        }
        populateDecorations(world, random, &biome, i, j);
        populateSnow(world, &self.m_temperatures, i, j);
        setFallInstantly(false);
    }

    fn saveChunks(&self, _saveEntities: bool, _listener: Option<&dyn IProgressUpdate>) -> bool { true }

    fn unload100OldestChunks(&self) -> bool { false }

    fn canSave(&self) -> bool { true }

    fn makeString(&self) -> String { String::from("RandomLevelSource") }
}
