use crate::item::{Item, ItemClass, item_block};
use std::rc::Rc;

pub const CLASS: ItemClass = ItemClass {
    m_name: "ItemPiston",
    m_parent: Some(&item_block::CLASS),
    getPlacedBlockMetadata: |_, _| 7,
    ..item_block::CLASS
};

pub fn create(n: i32) -> Rc<Item> { item_block::construct(&CLASS, n) }
