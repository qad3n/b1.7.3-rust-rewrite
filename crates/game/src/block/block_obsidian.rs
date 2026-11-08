use crate::block::{Block, BlockClass, block_stone};
use std::rc::Rc;

pub const CLASS: BlockClass = BlockClass {
    m_name: "BlockObsidian",
    m_parent: Some(&block_stone::CLASS),
    quantityDropped: |_, _| 1,
    idDropped: |_, _, _| Block::obsidian().m_blockID,
    ..block_stone::CLASS
};

pub fn create(id: i32, sprite: i32) -> Rc<Block> { block_stone::construct(&CLASS, id, sprite) }
