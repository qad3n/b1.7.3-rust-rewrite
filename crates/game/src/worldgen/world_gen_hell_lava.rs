use crate::block::Block;
use crate::world::World;
use crate::worldgen::WorldGenerator;
use jvm::random::Random;
use jvm::throwable::OrNpe;
use std::rc::Rc;

pub struct WorldGenHellLava
{
    m_liquid: i32,
}

impl WorldGenHellLava
{
    pub fn new(liquid: i32) -> WorldGenHellLava { WorldGenHellLava { m_liquid: liquid } }
}

impl WorldGenerator for WorldGenHellLava
{
    fn generate(&self, world: &Rc<World>, random: &Random, x: i32, y: i32, z: i32) -> bool
    {
        let netherrack: i32 = Block::netherrack().m_blockID;
        if world.getBlockId(x, y + 1, z) != netherrack
        {
            return false;
        }
        if world.getBlockId(x, y, z) != 0 && world.getBlockId(x, y, z) != netherrack
        {
            return false;
        }
        let mut n: i32 = 0;
        if world.getBlockId(x - 1, y, z) == netherrack
        {
            n += 1;
        }
        if world.getBlockId(x + 1, y, z) == netherrack
        {
            n += 1;
        }
        if world.getBlockId(x, y, z - 1) == netherrack
        {
            n += 1;
        }
        if world.getBlockId(x, y, z + 1) == netherrack
        {
            n += 1;
        }
        if world.getBlockId(x, y - 1, z) == netherrack
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
        if world.isAirBlock(x, y - 1, z)
        {
            n2 += 1;
        }
        if n == 4 && n2 == 1
        {
            world.setBlockWithNotify(x, y, z, self.m_liquid);
            world.m_scheduledUpdatesAreImmediate.set(true);
            Block::byId(self.m_liquid).orNpe().updateTick(world, x, y, z, random);
            world.m_scheduledUpdatesAreImmediate.set(false);
        }
        true
    }
}
