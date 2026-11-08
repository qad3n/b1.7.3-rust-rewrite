use crate::block::Block;
use crate::world::World;
use crate::worldgen::WorldGenerator;
use crate::worldgen::world_gen_taiga2::canGrowSpruce;
use jvm::random::Random;
use std::rc::Rc;

pub struct WorldGenTaiga1;

impl WorldGenTaiga1
{
    pub fn new() -> WorldGenTaiga1 { WorldGenTaiga1 }
}

impl WorldGenerator for WorldGenTaiga1
{
    fn generate(&self, world: &Rc<World>, random: &Random, x: i32, y: i32, z: i32) -> bool
    {
        let n6: i32 = random.nextIntBounded(5) + 7;
        let n7: i32 = n6 - random.nextIntBounded(2) - 3;
        let n8: i32 = n6 - n7;
        let n9: i32 = 1 + random.nextIntBounded(n8 + 1);
        if y < 1 || y + n6 + 1 > 128
        {
            return false;
        }
        if !canGrowSpruce(world, x, y, z, n6, n7, n9)
        {
            return false;
        }
        let n5: i32 = world.getBlockId(x, y - 1, z);
        if n5 != Block::grass().m_blockID && n5 != Block::dirt().m_blockID || y >= 128 - n6 - 1
        {
            return false;
        }
        world.setBlock(x, y - 1, z, Block::dirt().m_blockID);
        let mut n4: i32 = 0;
        for n3 in (y + n7..=y + n6).rev()
        {
            for n2 in x - n4..=x + n4
            {
                let n: i32 = n2 - x;
                for i in z - n4..=z + n4
                {
                    let n10: i32 = i - z;
                    if n.abs() == n4 && n10.abs() == n4 && n4 > 0
                        || Block::opaqueCubeLookup(world.getBlockId(n2, n3, i))
                    {
                        continue;
                    }
                    world.setBlockAndMetadata(n2, n3, i, Block::leaves().m_blockID, 1);
                }
            }
            if n4 >= 1 && n3 == y + n7 + 1
            {
                n4 -= 1;
                continue;
            }
            if n4 >= n9
            {
                continue;
            }
            n4 += 1;
        }
        for n3 in 0..n6 - 1
        {
            let n2: i32 = world.getBlockId(x, y + n3, z);
            if n2 != 0 && n2 != Block::leaves().m_blockID
            {
                continue;
            }
            world.setBlockAndMetadata(x, y + n3, z, Block::wood().m_blockID, 1);
        }
        true
    }
}
