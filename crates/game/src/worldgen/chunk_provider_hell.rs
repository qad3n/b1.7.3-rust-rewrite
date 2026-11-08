use crate::block::Block;
use crate::world::{Chunk, IChunkProvider, World};
use crate::worldgen::chunk_provider_generate::{chunk, chunkSeed, setFallInstantly};
use crate::worldgen::{
    MapGenBase, WorldGenFire, WorldGenFlowers, WorldGenGlowStone1, WorldGenGlowStone2, WorldGenHellLava,
    WorldGenerator, map_gen_caves_hell,
};
use common::noise::NoiseGeneratorOctaves;
use common::world::IProgressUpdate;
use jvm::field::Field;
use jvm::random::Random;
use std::rc::Rc;

pub struct ChunkProviderHell
{
    m_hellRNG: Random,
    m_minLimitPerlinNoise: NoiseGeneratorOctaves,
    m_maxLimitPerlinNoise: NoiseGeneratorOctaves,
    m_perlinNoise1: NoiseGeneratorOctaves,
    m_perlinNoise2: NoiseGeneratorOctaves,
    m_perlinNoise3: NoiseGeneratorOctaves,
    pub m_scaleNoise: NoiseGeneratorOctaves,
    pub m_depthNoise: NoiseGeneratorOctaves,
    m_worldObj: Rc<World>,
    m_heightMap: Field<Option<Vec<f64>>>,
    m_sandBuffer: Field<Option<Vec<f64>>>,
    m_gravelBuffer: Field<Option<Vec<f64>>>,
    m_depthBuffer: Field<Option<Vec<f64>>>,
    m_cave: Rc<MapGenBase>,
    m_perlinNoiseBuffer: Field<Option<Vec<f64>>>,
    m_minLimitPerlinNoiseBuffer: Field<Option<Vec<f64>>>,
    m_maxLimitPerlinNoiseBuffer: Field<Option<Vec<f64>>>,
    m_scaleNoiseBuffer: Field<Option<Vec<f64>>>,
    m_depthNoiseBuffer: Field<Option<Vec<f64>>>,
}

impl ChunkProviderHell
{
    fn world(&self) -> Rc<World> { Rc::clone(&self.m_worldObj) }

    pub fn new(world: &Rc<World>, seed: i64) -> Rc<ChunkProviderHell>
    {
        let cave: Rc<MapGenBase> = map_gen_caves_hell::create();
        let hellRNG: Random = Random::withSeed(seed);
        Rc::new(ChunkProviderHell {
            m_minLimitPerlinNoise: NoiseGeneratorOctaves::new(&hellRNG, 16),
            m_maxLimitPerlinNoise: NoiseGeneratorOctaves::new(&hellRNG, 16),
            m_perlinNoise1: NoiseGeneratorOctaves::new(&hellRNG, 8),
            m_perlinNoise2: NoiseGeneratorOctaves::new(&hellRNG, 4),
            m_perlinNoise3: NoiseGeneratorOctaves::new(&hellRNG, 4),
            m_scaleNoise: NoiseGeneratorOctaves::new(&hellRNG, 10),
            m_depthNoise: NoiseGeneratorOctaves::new(&hellRNG, 16),
            m_hellRNG: hellRNG,
            m_worldObj: Rc::clone(world),
            m_heightMap: Field::new(None),
            m_sandBuffer: Field::new(Some(vec![0.0; chunk::COLUMN_COUNT])),
            m_gravelBuffer: Field::new(Some(vec![0.0; chunk::COLUMN_COUNT])),
            m_depthBuffer: Field::new(Some(vec![0.0; chunk::COLUMN_COUNT])),
            m_cave: cave,
            m_perlinNoiseBuffer: Field::new(None),
            m_minLimitPerlinNoiseBuffer: Field::new(None),
            m_maxLimitPerlinNoiseBuffer: Field::new(None),
            m_scaleNoiseBuffer: Field::new(None),
            m_depthNoiseBuffer: Field::new(None),
        })
    }

    pub fn buildTerrain(&self, chunkX: i32, chunkZ: i32, blocks: &mut [i8])
    {
        let n: i32 = 4;
        let n2: i32 = 32;
        let n3: i32 = n + 1;
        let n4: i32 = 17;
        let n5: i32 = n + 1;
        let heightMap: Vec<f64> =
            self.generateHeightMap(self.m_heightMap.take(), chunkX * n, 0, chunkZ * n, n3, n4, n5);
        let at = |index: i32| -> f64 { heightMap[index as usize] };
        for i in 0..n
        {
            for j in 0..n
            {
                for k in 0..16
                {
                    let d: f64 = 0.125;
                    let mut d2: f64 = at(((i + 0) * n5 + (j + 0)) * n4 + (k + 0));
                    let mut d3: f64 = at(((i + 0) * n5 + (j + 1)) * n4 + (k + 0));
                    let mut d4: f64 = at(((i + 1) * n5 + (j + 0)) * n4 + (k + 0));
                    let mut d5: f64 = at(((i + 1) * n5 + (j + 1)) * n4 + (k + 0));
                    let d6: f64 = (at(((i + 0) * n5 + (j + 0)) * n4 + (k + 1)) - d2) * d;
                    let d7: f64 = (at(((i + 0) * n5 + (j + 1)) * n4 + (k + 1)) - d3) * d;
                    let d8: f64 = (at(((i + 1) * n5 + (j + 0)) * n4 + (k + 1)) - d4) * d;
                    let d9: f64 = (at(((i + 1) * n5 + (j + 1)) * n4 + (k + 1)) - d5) * d;
                    for i2 in 0..8
                    {
                        let d10: f64 = 0.25;
                        let mut d11: f64 = d2;
                        let mut d12: f64 = d3;
                        let d13: f64 = (d4 - d2) * d10;
                        let d14: f64 = (d5 - d3) * d10;
                        for i3 in 0..4
                        {
                            let mut n6: i32 = (i3 + i * 4) << 11 | (j * 4) << 7 | (k * 8 + i2);
                            let n7: i32 = 128;
                            let d15: f64 = 0.25;
                            let mut d16: f64 = d11;
                            let d17: f64 = (d12 - d11) * d15;
                            for _ in 0..4
                            {
                                let mut n8: i32 = 0;
                                if k * 8 + i2 < n2
                                {
                                    n8 = Block::lavaStill().m_blockID;
                                }
                                if d16 > 0.0
                                {
                                    n8 = Block::netherrack().m_blockID;
                                }
                                blocks[n6 as usize] = n8 as i8;
                                n6 += n7;
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

    pub fn buildSurfaces(&self, chunkX: i32, chunkZ: i32, blocks: &mut [i8])
    {
        let b0: i32 = 64;
        let d0: f64 = 0.03125;
        let rng: &Random = &self.m_hellRNG;
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
        let netherrack: i8 = Block::netherrack().m_blockID as i8;
        let bedrock: i8 = Block::bedrock().m_blockID as i8;
        for i in 0..16
        {
            for j in 0..16
            {
                let flag: bool = sandBuffer[(i + j * 16) as usize] + rng.nextDouble() * 0.2 > 0.0;
                let flag1: bool = gravelBuffer[(i + j * 16) as usize] + rng.nextDouble() * 0.2 > 0.0;
                let k: i32 = (depthBuffer[(i + j * 16) as usize] / 3.0 + 3.0 + rng.nextDouble() * 0.25) as i32;
                let mut l: i32 = -1;
                let mut b1: i8 = netherrack;
                let mut b2: i8 = netherrack;
                for i1 in (0..=127).rev()
                {
                    let j1: usize = ((j * 16 + i) * 128 + i1) as usize;
                    if i1 >= 127 - rng.nextIntBounded(5)
                    {
                        blocks[j1] = bedrock;
                        continue;
                    }
                    if i1 <= 0 + rng.nextIntBounded(5)
                    {
                        blocks[j1] = bedrock;
                        continue;
                    }
                    let b3: i8 = blocks[j1];
                    if b3 == 0
                    {
                        l = -1;
                    }
                    else if b3 == netherrack
                    {
                        if l == -1
                        {
                            if k <= 0
                            {
                                b1 = 0;
                                b2 = netherrack;
                            }
                            else if i1 >= b0 - 4 && i1 <= b0 + 1
                            {
                                b1 = netherrack;
                                b2 = netherrack;
                                if flag1
                                {
                                    b1 = Block::gravel().m_blockID as i8;
                                }
                                if flag1
                                {
                                    b2 = netherrack;
                                }
                                if flag
                                {
                                    b1 = Block::slowSand().m_blockID as i8;
                                }
                                if flag
                                {
                                    b2 = Block::slowSand().m_blockID as i8;
                                }
                            }
                            if i1 < b0 && b1 == 0
                            {
                                b1 = Block::lavaStill().m_blockID as i8;
                            }
                            l = k;
                            blocks[j1] = if i1 >= b0 - 1 { b1 } else { b2 };
                        }
                        else if l > 0
                        {
                            l -= 1;
                            blocks[j1] = b2;
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
        let d0: f64 = 684.412;
        let d1: f64 = 2053.236;
        let (fx, fy, fz): (f64, f64, f64) = (x as f64, y as f64, z as f64);
        self.m_scaleNoiseBuffer.set(Some(self.m_scaleNoise.generateNoiseOctaves(
            self.m_scaleNoiseBuffer.take(),
            fx,
            fy,
            fz,
            sizeX,
            1,
            sizeZ,
            1.0,
            0.0,
            1.0,
        )));
        self.m_depthNoiseBuffer.set(Some(self.m_depthNoise.generateNoiseOctaves(
            self.m_depthNoiseBuffer.take(),
            fx,
            fy,
            fz,
            sizeX,
            1,
            sizeZ,
            100.0,
            0.0,
            100.0,
        )));
        let perlinNoise: Vec<f64> = self.m_perlinNoise1.generateNoiseOctaves(
            self.m_perlinNoiseBuffer.take(),
            fx,
            fy,
            fz,
            sizeX,
            sizeY,
            sizeZ,
            d0 / 80.0,
            d1 / 60.0,
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
        let mut adouble: Vec<f64> = vec![0.0; sizeY as usize];
        for k in 0..sizeY
        {
            adouble[k as usize] = jvm::math::cos(k as f64 * jvm::math::PI * 6.0 / sizeY as f64) * 2.0;
            let mut d2: f64 = k as f64;
            if k > sizeY / 2
            {
                d2 = (sizeY - 1 - k) as f64;
            }
            if d2 < 4.0
            {
                d2 = 4.0 - d2;
                adouble[k as usize] -= d2 * d2 * d2 * 10.0;
            }
        }
        for _ in 0..sizeX
        {
            for _ in 0..sizeZ
            {
                let d4: f64 = 0.0;
                for l in 0..sizeY
                {
                    let d7: f64 = adouble[l as usize];
                    let d8: f64 = minLimitNoise[i] / 512.0;
                    let d9: f64 = maxLimitNoise[i] / 512.0;
                    let d10: f64 = (perlinNoise[i] / 10.0 + 1.0) / 2.0;
                    let mut d6: f64 = if d10 < 0.0
                    {
                        d8
                    }
                    else if d10 > 1.0
                    {
                        d9
                    }
                    else
                    {
                        d8 + (d9 - d8) * d10
                    };
                    d6 -= d7;
                    if l > sizeY - 4
                    {
                        let d11: f64 = ((l - (sizeY - 4)) as f32 / 3.0f32) as f64;
                        d6 = d6 * (1.0 - d11) + -10.0 * d11;
                    }
                    if (l as f64) < d4
                    {
                        let mut d12: f64 = (d4 - l as f64) / 4.0;
                        d12 = d12.clamp(0.0, 1.0);
                        d6 = d6 * (1.0 - d12) + -10.0 * d12;
                    }
                    heightMap[i] = d6;
                    i += 1;
                }
            }
        }
        self.m_perlinNoiseBuffer.set(Some(perlinNoise));
        self.m_minLimitPerlinNoiseBuffer.set(Some(minLimitNoise));
        self.m_maxLimitPerlinNoiseBuffer.set(Some(maxLimitNoise));
        heightMap
    }
}

impl IChunkProvider for ChunkProviderHell
{
    fn chunkExists(&self, _chunkX: i32, _chunkZ: i32) -> bool { true }

    fn provideChunk(&self, chunkX: i32, chunkZ: i32) -> Rc<Chunk>
    {
        self.m_hellRNG.setSeed(chunkSeed(chunkX, chunkZ));
        let mut blocks: Vec<i8> = vec![0; chunk::BLOCK_COUNT];
        self.buildTerrain(chunkX, chunkZ, &mut blocks);
        self.buildSurfaces(chunkX, chunkZ, &mut blocks);
        self.m_cave.place(self, &self.world(), chunkX, chunkZ, &mut blocks);
        Chunk::withBlocks(&self.world(), blocks, chunkX, chunkZ)
    }

    fn prepareChunk(&self, chunkX: i32, chunkZ: i32) -> Rc<Chunk> { self.provideChunk(chunkX, chunkZ) }

    fn populate(&self, _source: &Rc<dyn IChunkProvider>, chunkX: i32, chunkZ: i32)
    {
        setFallInstantly(true);
        let world: &Rc<World> = &self.world();
        let rng: &Random = &self.m_hellRNG;
        let i: i32 = chunkX * 16;
        let j: i32 = chunkZ * 16;
        for _ in 0..8
        {
            let l: i32 = i + rng.nextIntBounded(16) + 8;
            let i1: i32 = rng.nextIntBounded(120) + 4;
            let j1: i32 = j + rng.nextIntBounded(16) + 8;
            WorldGenHellLava::new(Block::lavaMoving().m_blockID).generate(world, rng, l, i1, j1);
        }
        let mut l1: i32 = rng.nextIntBounded(rng.nextIntBounded(10) + 1) + 1;
        for _ in 0..l1
        {
            let j3: i32 = i + rng.nextIntBounded(16) + 8;
            let k4: i32 = rng.nextIntBounded(120) + 4;
            let k1: i32 = j + rng.nextIntBounded(16) + 8;
            WorldGenFire.generate(world, rng, j3, k4, k1);
        }
        l1 = rng.nextIntBounded(rng.nextIntBounded(10) + 1);
        for _ in 0..l1
        {
            let k3: i32 = i + rng.nextIntBounded(16) + 8;
            let l4: i32 = rng.nextIntBounded(120) + 4;
            let l5: i32 = j + rng.nextIntBounded(16) + 8;
            WorldGenGlowStone1.generate(world, rng, k3, l4, l5);
        }
        for _ in 0..10
        {
            let l3: i32 = i + rng.nextIntBounded(16) + 8;
            let i5: i32 = rng.nextIntBounded(128);
            let i6: i32 = j + rng.nextIntBounded(16) + 8;
            WorldGenGlowStone2.generate(world, rng, l3, i5, i6);
        }
        for mushroom in [Block::mushroomBrown().m_blockID, Block::mushroomRed().m_blockID]
        {
            if rng.nextIntBounded(1) == 0
            {
                let x: i32 = i + rng.nextIntBounded(16) + 8;
                let y: i32 = rng.nextIntBounded(128);
                let z: i32 = j + rng.nextIntBounded(16) + 8;
                WorldGenFlowers::new(mushroom).generate(world, rng, x, y, z);
            }
        }
        setFallInstantly(false);
    }

    fn saveChunks(&self, _saveEntities: bool, _listener: Option<&dyn IProgressUpdate>) -> bool { true }

    fn unload100OldestChunks(&self) -> bool { false }

    fn canSave(&self) -> bool { true }

    fn makeString(&self) -> String { String::from("HellRandomLevelSource") }
}
