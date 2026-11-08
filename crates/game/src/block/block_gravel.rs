use crate::block::{Block, BlockClass, block_sand};
use crate::item::Item;
use std::rc::Rc;

pub const CLASS: BlockClass = BlockClass {
    m_name: "BlockGravel",
    m_parent: Some(&block_sand::CLASS),
    idDropped: |this, _, random| {
        if random.nextIntBounded(10) == 0
        {
            Item::flint().m_shiftedIndex
        }
        else
        {
            this.m_blockID
        }
    },
    ..block_sand::CLASS
};

pub fn create(id: i32, sprite: i32) -> Rc<Block> { block_sand::construct(&CLASS, id, sprite) }
