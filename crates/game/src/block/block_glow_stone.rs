use crate::block::{BLOCK, Block, BlockClass, Material};
use crate::item::Item;
use std::rc::Rc;

pub const CLASS: BlockClass = BlockClass {
    m_name: "BlockGlowStone",
    m_parent: Some(&BLOCK),
    quantityDropped: |_, random| 2 + random.nextIntBounded(3),
    idDropped: |_, _, _| Item::lightStoneDust().m_shiftedIndex,
    ..BLOCK
};

pub fn create(id: i32, sprite: i32, material: Rc<Material>) -> Rc<Block>
{
    Block::constructWithSprite(&CLASS, Box::new(()), id, sprite, material)
}
