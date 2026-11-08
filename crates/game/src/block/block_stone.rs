use crate::block::{BLOCK, Block, BlockClass, Material};
use std::rc::Rc;

pub const CLASS: BlockClass = BlockClass {
    m_name: "BlockStone",
    m_parent: Some(&BLOCK),
    idDropped: |_, _, _| Block::cobblestone().m_blockID,
    ..BLOCK
};

pub fn construct(class: &'static BlockClass, id: i32, sprite: i32) -> Rc<Block>
{
    Block::constructWithSprite(class, Box::new(()), id, sprite, Material::rock())
}

pub fn create(id: i32, sprite: i32) -> Rc<Block> { construct(&CLASS, id, sprite) }
