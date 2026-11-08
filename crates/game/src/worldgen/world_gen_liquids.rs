use crate::block::Block;
use crate::world::World;
use crate::worldgen::WorldGenerator;
use jvm::random::Random;
use jvm::throwable::OrNpe;
use std::rc::Rc;

pub struct WorldGenLiquids
{
    m_liquidBlockId: i32,
}

impl WorldGenLiquids
{
    pub fn new(liquid: i32) -> WorldGenLiquids { WorldGenLiquids { m_liquidBlockId: liquid } }
}

impl WorldGenerator for WorldGenLiquids
{
    fn generate(&self, world: &Rc<World>, random: &Random, x: i32, y: i32, z: i32) -> bool
    {
        let stone: i32 = Block::stone().m_blockID;
        if world.getBlockId(x, y + 1, z) != stone
        {
            return false;
        }
        if world.getBlockId(x, y - 1, z) != stone
        {
            return false;
        }
        if world.getBlockId(x, y, z) != 0 && world.getBlockId(x, y, z) != stone
        {
            return false;
        }
        let mut n: i32 = 0;
        if world.getBlockId(x - 1, y, z) == stone
        {
            n += 1;
        }
        if world.getBlockId(x + 1, y, z) == stone
        {
            n += 1;
        }
        if world.getBlockId(x, y, z - 1) == stone
        {
            n += 1;
        }
        if world.getBlockId(x, y, z + 1) == stone
        {
            n += 1;
        }
        let mut n2: i32 = 0;
        if world.isAirBlock(x - 1, y, z)
        {
            n2 += 1;
        }
        if world.isAirBlock(x + 1, y, z)
        {
            n2 += 1;
        }
        if world.isAirBlock(x, y, z - 1)
        {
            n2 += 1;
        }
        if world.isAirBlock(x, y, z + 1)
        {
            n2 += 1;
        }
        if n == 3 && n2 == 1
        {
            world.setBlockWithNotify(x, y, z, self.m_liquidBlockId);
            world.m_scheduledUpdatesAreImmediate.set(true);
            Block::byId(self.m_liquidBlockId).orNpe().updateTick(world, x, y, z, random);
            world.m_scheduledUpdatesAreImmediate.set(false);
        }
        true
    }
}
