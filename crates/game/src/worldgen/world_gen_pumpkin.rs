use crate::block::Block;
use crate::world::World;
use crate::worldgen::WorldGenerator;
use jvm::random::Random;
use std::rc::Rc;

pub struct WorldGenPumpkin;

impl WorldGenerator for WorldGenPumpkin
{
    fn generate(&self, world: &Rc<World>, random: &Random, x: i32, y: i32, z: i32) -> bool
    {
        for _ in 0..64
        {
            let j: i32 = x + random.nextIntBounded(8) - random.nextIntBounded(8);
            let k: i32 = y + random.nextIntBounded(4) - random.nextIntBounded(4);
            let l: i32 = z + random.nextIntBounded(8) - random.nextIntBounded(8);
            if world.isAirBlock(j, k, l)
                && world.getBlockId(j, k - 1, l) == Block::grass().m_blockID
                && Block::pumpkin().canPlaceBlockAt(world, j, k, l)
            {
                world.setBlockAndMetadata(j, k, l, Block::pumpkin().m_blockID, random.nextIntBounded(4));
            }
        }
        true
    }
}
