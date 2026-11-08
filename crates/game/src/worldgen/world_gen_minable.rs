use crate::block::Block;
use crate::math::math_helper;
use crate::world::World;
use crate::worldgen::WorldGenerator;
use jvm::random::Random;
use std::rc::Rc;

pub struct WorldGenMinable
{
    m_minableBlockId: i32,
    m_numberOfBlocks: i32,
}

impl WorldGenMinable
{
    pub fn new(block: i32, size: i32) -> WorldGenMinable
    {
        WorldGenMinable { m_minableBlockId: block, m_numberOfBlocks: size }
    }
}

impl WorldGenerator for WorldGenMinable
{
    fn generate(&self, world: &Rc<World>, random: &Random, x: i32, y: i32, z: i32) -> bool
    {
        let size: i32 = self.m_numberOfBlocks;
        let f: f32 = random.nextFloat() * jvm::math::PI as f32;
        let d0: f64 = ((x + 8) as f32 + math_helper::sin(f) * size as f32 / 8.0f32) as f64;
        let d1: f64 = ((x + 8) as f32 - math_helper::sin(f) * size as f32 / 8.0f32) as f64;
        let d2: f64 = ((z + 8) as f32 + math_helper::cos(f) * size as f32 / 8.0f32) as f64;
        let d3: f64 = ((z + 8) as f32 - math_helper::cos(f) * size as f32 / 8.0f32) as f64;
        let d4: f64 = (y + random.nextIntBounded(3) + 2) as f64;
        let d5: f64 = (y + random.nextIntBounded(3) + 2) as f64;
        for i in 0..=size
        {
            let d6: f64 = d0 + (d1 - d0) * i as f64 / size as f64;
            let d7: f64 = d4 + (d5 - d4) * i as f64 / size as f64;
            let d8: f64 = d2 + (d3 - d2) * i as f64 / size as f64;
            let d9: f64 = random.nextDouble() * size as f64 / 16.0;
            let d10: f64 = (math_helper::sin(i as f32 * jvm::math::PI as f32 / size as f32) + 1.0f32) as f64 * d9 + 1.0;
            let d11: f64 = (math_helper::sin(i as f32 * jvm::math::PI as f32 / size as f32) + 1.0f32) as f64 * d9 + 1.0;
            let j: i32 = math_helper::floor_double(d6 - d10 / 2.0);
            let k: i32 = math_helper::floor_double(d7 - d11 / 2.0);
            let l: i32 = math_helper::floor_double(d8 - d10 / 2.0);
            let i1: i32 = math_helper::floor_double(d6 + d10 / 2.0);
            let j1: i32 = math_helper::floor_double(d7 + d11 / 2.0);
            let k1: i32 = math_helper::floor_double(d8 + d10 / 2.0);
            for l1 in j..=i1
            {
                let d12: f64 = (l1 as f64 + 0.5 - d6) / (d10 / 2.0);
                if d12 * d12 < 1.0
                {
                    for i2 in k..=j1
                    {
                        let d13: f64 = (i2 as f64 + 0.5 - d7) / (d11 / 2.0);
                        if d12 * d12 + d13 * d13 < 1.0
                        {
                            for j2 in l..=k1
                            {
                                let d14: f64 = (j2 as f64 + 0.5 - d8) / (d10 / 2.0);
                                if d12 * d12 + d13 * d13 + d14 * d14 < 1.0
                                    && world.getBlockId(l1, i2, j2) == Block::stone().m_blockID
                                {
                                    world.setBlock(l1, i2, j2, self.m_minableBlockId);
                                }
                            }
                        }
                    }
                }
            }
        }
        true
    }
}
