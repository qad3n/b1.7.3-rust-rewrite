use crate::block::{Block, Material};
use crate::world::World;
use crate::worldgen::WorldGenerator;
use jvm::random::Random;
use std::rc::Rc;

pub struct WorldGenReed;

impl WorldGenerator for WorldGenReed
{
    fn generate(&self, world: &Rc<World>, random: &Random, x: i32, y: i32, z: i32) -> bool
    {
        let isWater =
            |i: i32, j: i32, k: i32| -> bool { Rc::ptr_eq(&world.getBlockMaterial(i, j, k), &Material::water()) };
        for _ in 0..20
        {
            let j: i32 = x + random.nextIntBounded(4) - random.nextIntBounded(4);
            let k: i32 = y;
            let l: i32 = z + random.nextIntBounded(4) - random.nextIntBounded(4);
            if world.isAirBlock(j, k, l)
                && (isWater(j - 1, k - 1, l)
                    || isWater(j + 1, k - 1, l)
                    || isWater(j, k - 1, l - 1)
                    || isWater(j, k - 1, l + 1))
            {
                let i1: i32 = 2 + random.nextIntBounded(random.nextIntBounded(3) + 1);
                for j1 in 0..i1
                {
                    if Block::reed().canBlockStay(world, j, k + j1, l)
                    {
                        world.setBlock(j, k + j1, l, Block::reed().m_blockID);
                    }
                }
            }
        }
        true
    }
}
