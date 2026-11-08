use crate::block::Block;
use crate::world::World;
use crate::worldgen::WorldGenerator;
use jvm::random::Random;
use std::rc::Rc;

pub struct WorldGenTaiga2;

impl WorldGenTaiga2
{
    pub fn new() -> WorldGenTaiga2 { WorldGenTaiga2 }
}

pub(super) fn canGrowSpruce(world: &Rc<World>, x: i32, y: i32, z: i32, height: i32, trunk: i32, radius: i32) -> bool
{
    let mut bl: bool = true;
    let mut n7: i32 = y;
    while n7 <= y + 1 + height && bl
    {
        let n6: i32 = if n7 - y < trunk { 0 } else { radius };
        let mut n5: i32 = x - n6;
        while n5 <= x + n6 && bl
        {
            let mut n4: i32 = z - n6;
            while n4 <= z + n6 && bl
            {
                if (0..128).contains(&n7)
                {
                    let n3: i32 = world.getBlockId(n5, n7, n4);
                    if n3 != 0 && n3 != Block::leaves().m_blockID
                    {
                        bl = false;
                    }
                }
                else
                {
                    bl = false;
                }
                n4 += 1;
            }
            n5 += 1;
        }
        n7 += 1;
    }
    bl
}

impl WorldGenerator for WorldGenTaiga2
{
    fn generate(&self, world: &Rc<World>, random: &Random, x: i32, y: i32, z: i32) -> bool
    {
        let n8: i32 = random.nextIntBounded(4) + 6;
        let n9: i32 = 1 + random.nextIntBounded(2);
        let n10: i32 = n8 - n9;
        let n11: i32 = 2 + random.nextIntBounded(2);
        if y < 1 || y + n8 + 1 > 128
        {
            return false;
        }
        if !canGrowSpruce(world, x, y, z, n8, n9, n11)
        {
            return false;
        }
        let n7: i32 = world.getBlockId(x, y - 1, z);
        if n7 != Block::grass().m_blockID && n7 != Block::dirt().m_blockID || y >= 128 - n8 - 1
        {
            return false;
        }
        world.setBlock(x, y - 1, z, Block::dirt().m_blockID);
        let mut n6: i32 = random.nextIntBounded(2);
        let mut n5: i32 = 1;
        let mut n4: i32 = 0;
        for n3 in 0..=n10
        {
            let n2: i32 = y + n8 - n3;
            for n in x - n6..=x + n6
            {
                let n12: i32 = n - x;
                for i in z - n6..=z + n6
                {
                    let n13: i32 = i - z;
                    if n12.abs() == n6 && n13.abs() == n6 && n6 > 0
                        || Block::opaqueCubeLookup(world.getBlockId(n, n2, i))
                    {
                        continue;
                    }
                    world.setBlockAndMetadata(n, n2, i, Block::leaves().m_blockID, 1);
                }
            }
            if n6 >= n5
            {
                n6 = n4;
                n4 = 1;
                n5 += 1;
                if n5 <= n11
                {
                    continue;
                }
                n5 = n11;
                continue;
            }
            n6 += 1;
        }
        let n3: i32 = random.nextIntBounded(3);
        for n2 in 0..n8 - n3
        {
            let n: i32 = world.getBlockId(x, y + n2, z);
            if n != 0 && n != Block::leaves().m_blockID
            {
                continue;
            }
            world.setBlockAndMetadata(x, y + n2, z, Block::wood().m_blockID, 1);
        }
        true
    }
}
