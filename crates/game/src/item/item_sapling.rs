use crate::block::Block;
use crate::item::{Item, ItemClass, item_block};
use std::rc::Rc;

pub const CLASS: ItemClass = ItemClass {
    m_name: "ItemSapling",
    m_parent: Some(&item_block::CLASS),
    getPlacedBlockMetadata: |_, metadata| metadata,
    getIconFromDamage: |_, metadata| Block::sapling().getBlockTextureFromSideAndMetadata(0, metadata),
    ..item_block::CLASS
};

pub fn create(n: i32) -> Rc<Item> { item_block::construct(&CLASS, n).setMaxDamage(0).setHasSubtypes(true) }
