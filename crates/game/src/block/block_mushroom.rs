use crate::block::{Block, BlockClass, block_flower};
use crate::world::World;
use jvm::random::Random;
use std::rc::Rc;

fn updateTick(this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32, random: &Random)
{
    if random.nextIntBounded(100) != 0
    {
        return;
    }
    let i: i32 = x + random.nextIntBounded(3) - 1;
    let j: i32 = y + random.nextIntBounded(2) - random.nextIntBounded(2);
    let k: i32 = z + random.nextIntBounded(3) - 1;
    if world.isAirBlock(i, j, k) && this.canBlockStay(world, i, j, k)
    {
        let _x: i32 = x + random.nextIntBounded(3) - 1;
        let _z: i32 = z + random.nextIntBounded(3) - 1;
        if world.isAirBlock(i, j, k) && this.canBlockStay(world, i, j, k)
        {
            world.setBlockWithNotify(i, j, k, this.m_blockID);
        }
    }
}

pub const CLASS: BlockClass = BlockClass {
    m_name: "BlockMushroom",
    m_parent: Some(&block_flower::CLASS),
    updateTick,
    canThisPlantGrowOnThisBlockID: |_, block| Block::opaqueCubeLookup(block),
    canBlockStay: |this, world, x, y, z| {
        if y < 0 || y >= 128
        {
            return false;
        }
        world.getFullBlockLightValue(x, y, z) < 13 && this.canThisPlantGrowOnThisBlockID(world.getBlockId(x, y - 1, z))
    },
    ..block_flower::CLASS
};

pub fn create(id: i32, sprite: i32) -> Rc<Block>
{
    let block: Rc<Block> = block_flower::construct(&CLASS, Box::new(()), id, sprite);
    let f: f32 = 0.2;
    block.setBlockBounds(0.5 - f, 0.0, 0.5 - f, 0.5 + f, f * 2.0, 0.5 + f);
    block.setTickOnLoad(true)
}
