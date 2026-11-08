use crate::block::{BLOCK, Block, BlockClass, Material};
use std::rc::Rc;

pub const CLASS: BlockClass = BlockClass {
    m_name: "BlockBookshelf",
    m_parent: Some(&BLOCK),
    getBlockTextureFromSide: |this, face| {
        if face <= 1
        {
            4
        }
        else
        {
            this.m_blockIndexInTexture.get()
        }
    },
    quantityDropped: |_, _| 0,
    ..BLOCK
};

pub fn create(id: i32, sprite: i32) -> Rc<Block>
{
    Block::constructWithSprite(&CLASS, Box::new(()), id, sprite, Material::wood())
}
