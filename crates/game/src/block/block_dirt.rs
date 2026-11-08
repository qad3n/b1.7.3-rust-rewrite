use crate::block::{BLOCK, Block, BlockClass, Material};
use std::rc::Rc;

pub const CLASS: BlockClass = BlockClass { m_name: "BlockDirt", m_parent: Some(&BLOCK), ..BLOCK };

pub fn create(id: i32, sprite: i32) -> Rc<Block>
{
    Block::constructWithSprite(&CLASS, Box::new(()), id, sprite, Material::ground())
}
