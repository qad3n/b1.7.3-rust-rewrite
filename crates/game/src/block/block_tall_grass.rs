use crate::block::{Block, BlockClass, block_flower};
use crate::item::Item;
use crate::world::IBlockAccess;
use crate::worldgen::ColorizerGrass;
use std::rc::Rc;

fn getBlockTextureFromSideAndMetadata(this: &Rc<Block>, _face: i32, metadata: i32) -> i32
{
    let sprite: i32 = this.m_blockIndexInTexture.get();
    match metadata
    {
        1 => sprite,
        2 => sprite + 16 + 1,
        0 => sprite + 16,
        _ => sprite,
    }
}

fn colorMultiplier(_this: &Rc<Block>, world: &dyn IBlockAccess, x: i32, y: i32, z: i32) -> i32
{
    let n: i32 = world.getBlockMetadata(x, y, z);
    if n == 0
    {
        return 0xFFFFFF;
    }
    let mut l: i64 = x.wrapping_mul(3129871).wrapping_add(z.wrapping_mul(6129781)).wrapping_add(y) as i64;
    l = l.wrapping_mul(l).wrapping_mul(42317861).wrapping_add(l.wrapping_mul(11));
    let x: i32 = (x as i64 + (l >> 14 & 0x1F)) as i32;
    let _y: i32 = (y as i64 + (l >> 19 & 0x1F)) as i32;
    let z: i32 = (z as i64 + (l >> 24 & 0x1F)) as i32;
    world.getWorldChunkManager().getBiomes(x, z, 1, 1);
    let d: f64 = world.getWorldChunkManager().temperature(0);
    let d2: f64 = world.getWorldChunkManager().humidity(0);
    ColorizerGrass::getGrassColor(d, d2)
}

pub const CLASS: BlockClass = BlockClass {
    m_name: "BlockTallGrass",
    m_parent: Some(&block_flower::CLASS),
    getBlockTextureFromSideAndMetadata,
    colorMultiplier,
    idDropped: |_, _, random| {
        if random.nextIntBounded(8) == 0
        {
            Item::seeds().m_shiftedIndex
        }
        else
        {
            -1
        }
    },
    ..block_flower::CLASS
};

pub fn create(id: i32, sprite: i32) -> Rc<Block>
{
    let block: Rc<Block> = block_flower::construct(&CLASS, Box::new(()), id, sprite);
    let f: f32 = 0.4;
    block.setBlockBounds(0.5 - f, 0.0, 0.5 - f, 0.5 + f, 0.8, 0.5 + f);
    block
}
