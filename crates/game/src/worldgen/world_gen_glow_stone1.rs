use crate::block::Block;
use crate::world::World;
use crate::worldgen::WorldGenerator;
use jvm::random::Random;
use std::rc::Rc;

pub struct WorldGenGlowStone1;

impl WorldGenerator for WorldGenGlowStone1
{
    fn generate(&self, world: &Rc<World>, random: &Random, x: i32, y: i32, z: i32) -> bool
    {
        generateGlowStone(world, random, x, y, z)
    }
}

pub(super) fn generateGlowStone(world: &Rc<World>, random: &Random, x: i32, y: i32, z: i32) -> bool
{
    if !world.isAirBlock(x, y, z)
    {
        return false;
    }
    if world.getBlockId(x, y + 1, z) != Block::netherrack().m_blockID
    {
        return false;
    }
    world.setBlockWithNotify(x, y, z, Block::glowStone().m_blockID);
    for _ in 0..1500
    {
        let j: i32 = x + random.nextIntBounded(8) - random.nextIntBounded(8);
        let k: i32 = y - random.nextIntBounded(12);
        let l: i32 = z + random.nextIntBounded(8) - random.nextIntBounded(8);
        if world.getBlockId(j, k, l) == 0
        {
            let mut i1: i32 = 0;
            for j1 in 0..6
            {
                let k1: i32 = match j1
                {
                    0 => world.getBlockId(j - 1, k, l),
                    1 => world.getBlockId(j + 1, k, l),
                    2 => world.getBlockId(j, k - 1, l),
                    3 => world.getBlockId(j, k + 1, l),
                    4 => world.getBlockId(j, k, l - 1),
                    _ => world.getBlockId(j, k, l + 1),
                };
                if k1 == Block::glowStone().m_blockID
                {
                    i1 += 1;
                }
            }
            if i1 == 1
            {
                world.setBlockWithNotify(j, k, l, Block::glowStone().m_blockID);
            }
        }
    }
    true
}
