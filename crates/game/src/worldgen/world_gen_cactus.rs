use crate::block::Block;
use crate::world::World;
use crate::worldgen::WorldGenerator;
use jvm::random::Random;
use std::rc::Rc;

pub struct WorldGenCactus;

impl WorldGenerator for WorldGenCactus
{
    fn generate(&self, world: &Rc<World>, random: &Random, x: i32, y: i32, z: i32) -> bool
    {
        for _ in 0..10
        {
            let j: i32 = x + random.nextIntBounded(8) - random.nextIntBounded(8);
            let k: i32 = y + random.nextIntBounded(4) - random.nextIntBounded(4);
            let l: i32 = z + random.nextIntBounded(8) - random.nextIntBounded(8);
            if world.isAirBlock(j, k, l)
            {
                let i1: i32 = 1 + random.nextIntBounded(random.nextIntBounded(3) + 1);
                for j1 in 0..i1
                {
                    if Block::cactus().canBlockStay(world, j, k + j1, l)
                    {
                        world.setBlock(j, k + j1, l, Block::cactus().m_blockID);
                    }
                }
            }
        }
        true
    }
}
