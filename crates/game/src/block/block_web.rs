use crate::block::{BLOCK, Block, BlockClass, Material};
use crate::item::Item;
use std::rc::Rc;

pub const CLASS: BlockClass = BlockClass {
    m_name: "BlockWeb",
    m_parent: Some(&BLOCK),
    onEntityCollidedWithBlock: |_, _, _, _, _, entity| entity.m_isInWeb.set(true),
    isOpaqueCube: |_| false,
    getCollisionBoundingBoxFromPool: |_, _, _, _, _| None,
    getRenderType: |_| 1,
    renderAsNormalBlock: |_| false,
    idDropped: |_, _, _| Item::silk().m_shiftedIndex,
    ..BLOCK
};

pub fn create(id: i32, sprite: i32) -> Rc<Block>
{
    Block::constructWithSprite(&CLASS, Box::new(()), id, sprite, Material::COBWEB())
}
