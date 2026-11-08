use crate::block::{Block, BlockClass, block_flower};
use std::rc::Rc;

pub const CLASS: BlockClass = BlockClass {
    m_name: "BlockDeadBush",
    m_parent: Some(&block_flower::CLASS),
    canThisPlantGrowOnThisBlockID: |_, block| block == Block::sand().m_blockID,
    getBlockTextureFromSideAndMetadata: |this, _, _| this.m_blockIndexInTexture.get(),
    idDropped: |_, _, _| -1,
    ..block_flower::CLASS
};

pub fn create(id: i32, sprite: i32) -> Rc<Block>
{
    let block: Rc<Block> = block_flower::construct(&CLASS, Box::new(()), id, sprite);
    let f: f32 = 0.4;
    block.setBlockBounds(0.5 - f, 0.0, 0.5 - f, 0.5 + f, 0.8, 0.5 + f);
    block
}
