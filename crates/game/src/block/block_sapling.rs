use crate::block::{Block, BlockClass, block_flower};
use crate::world::World;
use crate::worldgen::{WorldGenBigTree, WorldGenForest, WorldGenTaiga2, WorldGenTrees, WorldGenerator};
use jvm::random::Random;
use std::rc::Rc;

fn updateTick(this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32, random: &Random)
{
    if world.m_multiplayerWorld.get()
    {
        return;
    }
    (block_flower::CLASS.updateTick)(this, world, x, y, z, random);
    if world.getBlockLightValue(x, y + 1, z) >= 9 && random.nextIntBounded(30) == 0
    {
        let n: i32 = world.getBlockMetadata(x, y, z);
        if (n & 8) == 0
        {
            world.setBlockMetadataWithNotify(x, y, z, n | 8);
        }
        else
        {
            growTree(this, world, x, y, z, random);
        }
    }
}

pub fn growTree(this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32, rand: &Random)
{
    let n: i32 = world.getBlockMetadata(x, y, z) & 3;
    world.setBlock(x, y, z, 0);
    let worldGenerator: Box<dyn WorldGenerator> = if n == 1
    {
        Box::new(WorldGenTaiga2::new())
    }
    else if n == 2
    {
        Box::new(WorldGenForest::new())
    }
    else
    {
        let trees: Box<dyn WorldGenerator> = Box::new(WorldGenTrees::new());
        if rand.nextIntBounded(10) == 0
        {
            Box::new(WorldGenBigTree::new())
        }
        else
        {
            trees
        }
    };
    if !worldGenerator.generate(world, rand, x, y, z)
    {
        world.setBlockAndMetadata(x, y, z, this.m_blockID, n);
    }
}

pub const CLASS: BlockClass = BlockClass {
    m_name: "BlockSapling",
    m_parent: Some(&block_flower::CLASS),
    updateTick,
    getBlockTextureFromSideAndMetadata: |this, face, metadata| {
        let metadata: i32 = metadata & 0x3;
        match metadata
        {
            1 => 63,
            2 => 79,
            _ => (block_flower::CLASS.getBlockTextureFromSideAndMetadata)(this, face, metadata),
        }
    },
    damageDropped: |_, metadata| metadata & 3,
    ..block_flower::CLASS
};

pub fn create(id: i32, sprite: i32) -> Rc<Block>
{
    let block: Rc<Block> = block_flower::construct(&CLASS, Box::new(()), id, sprite);
    let f: f32 = 0.4;
    block.setBlockBounds(0.5 - f, 0.0, 0.5 - f, 0.5 + f, f * 2.0, 0.5 + f);
    block
}
