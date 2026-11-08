use crate::block::Block;
use crate::world::World;
use crate::worldgen::WorldGenerator;
use jvm::random::Random;
use std::rc::Rc;

pub struct WorldGenTrees;

impl WorldGenTrees
{
    pub fn new() -> WorldGenTrees { WorldGenTrees }
}

impl WorldGenerator for WorldGenTrees
{
    fn generate(&self, world: &Rc<World>, random: &Random, x: i32, y: i32, z: i32) -> bool
    {
        generateTree(world, random, x, y, z, 4, None)
    }
}

pub(super) fn generateTree(
    world: &Rc<World>,
    random: &Random,
    x: i32,
    y: i32,
    z: i32,
    minHeight: i32,
    metadata: Option<i32>,
) -> bool
{
    let place = |i: i32, j: i32, k: i32, block: i32| match metadata
    {
        Some(meta) => world.setBlockAndMetadata(i, j, k, block, meta),
        None => world.setBlock(i, j, k, block),
    };
    let n: i32 = random.nextIntBounded(3) + minHeight;
    let mut n2: i32 = 1;
    if y < 1 || y + n + 1 > 128
    {
        return false;
    }
    for i in y..=y + 1 + n
    {
        let mut n3: i32 = 1;
        if i == y
        {
            n3 = 0;
        }
        if i >= y + 1 + n - 2
        {
            n3 = 2;
        }
        let mut x2: i32 = x - n3;
        while x2 <= x + n3 && n2 != 0
        {
            let mut z2: i32 = z - n3;
            while z2 <= z + n3 && n2 != 0
            {
                if (0..128).contains(&i)
                {
                    let blockId: i32 = world.getBlockId(x2, i, z2);
                    if blockId != 0 && blockId != Block::leaves().m_blockID
                    {
                        n2 = 0;
                    }
                }
                else
                {
                    n2 = 0;
                }
                z2 += 1;
            }
            x2 += 1;
        }
    }
    if n2 == 0
    {
        return false;
    }
    let blockId2: i32 = world.getBlockId(x, y - 1, z);
    if (blockId2 != Block::grass().m_blockID && blockId2 != Block::dirt().m_blockID) || y >= 128 - n - 1
    {
        return false;
    }
    world.setBlock(x, y - 1, z, Block::dirt().m_blockID);
    for j in y - 3 + n..=y + n
    {
        let n4: i32 = j - (y + n);
        let n5: i32 = 1 - n4 / 2;
        for k in x - n5..=x + n5
        {
            let a: i32 = k - x;
            for l in z - n5..=z + n5
            {
                let a2: i32 = l - z;
                if a.abs() == n5 && a2.abs() == n5
                {
                    if random.nextIntBounded(2) == 0
                    {
                        continue;
                    }
                    if n4 == 0
                    {
                        continue;
                    }
                }
                if !Block::opaqueCubeLookup(world.getBlockId(k, j, l))
                {
                    place(k, j, l, Block::leaves().m_blockID);
                }
            }
        }
    }
    for n6 in 0..n
    {
        let blockId3: i32 = world.getBlockId(x, y + n6, z);
        if blockId3 == 0 || blockId3 == Block::leaves().m_blockID
        {
            place(x, y + n6, z, Block::wood().m_blockID);
        }
    }
    true
}
