use crate::block::{BLOCK, Block, BlockClass, Material};
use std::rc::Rc;

fn getBlockTextureFromSide(this: &Rc<Block>, face: i32) -> i32
{
    if face == 1
    {
        return this.m_blockIndexInTexture.get() - 16;
    }
    if face == 0
    {
        return this.m_blockIndexInTexture.get() + 16;
    }
    this.m_blockIndexInTexture.get()
}

pub const CLASS: BlockClass =
    BlockClass { m_name: "BlockSandStone", m_parent: Some(&BLOCK), getBlockTextureFromSide, ..BLOCK };

pub fn create(id: i32) -> Rc<Block> { Block::constructWithSprite(&CLASS, Box::new(()), id, 192, Material::rock()) }
