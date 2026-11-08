use crate::block::Block;
use crate::world::World;
use crate::worldgen::WorldGenerator;
use jvm::random::Random;
use jvm::throwable::OrNpe;
use std::rc::Rc;

pub struct WorldGenDeadBush
{
    m_deadBush: i32,
}

impl WorldGenDeadBush
{
    pub fn new(deadBush: i32) -> WorldGenDeadBush { WorldGenDeadBush { m_deadBush: deadBush } }
}

impl WorldGenerator for WorldGenDeadBush
{
    fn generate(&self, world: &Rc<World>, random: &Random, x: i32, mut y: i32, z: i32) -> bool
    {
        loop
        {
            let i: i32 = world.getBlockId(x, y, z);
            if !((i == 0 || i == Block::leaves().m_blockID) && y > 0)
            {
                break;
            }
            y -= 1;
        }
        for _ in 0..4
        {
            let k: i32 = x + random.nextIntBounded(8) - random.nextIntBounded(8);
            let l: i32 = y + random.nextIntBounded(4) - random.nextIntBounded(4);
            let i1: i32 = z + random.nextIntBounded(8) - random.nextIntBounded(8);
            if world.isAirBlock(k, l, i1) && Block::byId(self.m_deadBush).orNpe().canBlockStay(world, k, l, i1)
            {
                world.setBlock(k, l, i1, self.m_deadBush);
            }
        }
        true
    }
}
