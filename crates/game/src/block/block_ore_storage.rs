use crate::block::{BLOCK, Block, BlockClass, Material};
use std::rc::Rc;

pub const CLASS: BlockClass = BlockClass {
    m_name: "BlockOreStorage",
    m_parent: Some(&BLOCK),
    getBlockTextureFromSide: |this, _| this.m_blockIndexInTexture.get(),
    ..BLOCK
};

pub fn create(id: i32, sprite: i32) -> Rc<Block>
{
    let block: Rc<Block> = Block::construct(&CLASS, Box::new(()), id, Material::iron());
    block.m_blockIndexInTexture.set(sprite);
    block
}
