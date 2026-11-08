use crate::block::{BLOCK, Block, BlockClass, Material};
use crate::item::Item;
use std::rc::Rc;

pub const CLASS: BlockClass = BlockClass {
    m_name: "BlockClay",
    m_parent: Some(&BLOCK),
    idDropped: |_, _, _| Item::clay().m_shiftedIndex,
    quantityDropped: |_, _| 4,
    ..BLOCK
};

pub fn create(id: i32, sprite: i32) -> Rc<Block>
{
    Block::constructWithSprite(&CLASS, Box::new(()), id, sprite, Material::clay())
}
