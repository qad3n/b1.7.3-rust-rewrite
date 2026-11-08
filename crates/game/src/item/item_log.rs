use crate::block::Block;
use crate::item::{Item, ItemClass, item_block};
use std::rc::Rc;

pub const CLASS: ItemClass = ItemClass {
    m_name: "ItemLog",
    m_parent: Some(&item_block::CLASS),
    getIconFromDamage: |_, metadata| Block::wood().getBlockTextureFromSideAndMetadata(2, metadata),
    getPlacedBlockMetadata: |_, metadata| metadata,
    ..item_block::CLASS
};

pub fn create(n: i32) -> Rc<Item> { item_block::construct(&CLASS, n).setMaxDamage(0).setHasSubtypes(true) }
