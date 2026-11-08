use crate::block::{Block, block_step};
use crate::item::{Item, ItemClass, item_block};
use std::rc::Rc;

pub const CLASS: ItemClass = ItemClass {
    m_name: "ItemSlab",
    m_parent: Some(&item_block::CLASS),
    getIconFromDamage: |_, metadata| Block::stairSingle().getBlockTextureFromSideAndMetadata(2, metadata),
    getPlacedBlockMetadata: |_, metadata| metadata,
    getItemNameIS: |this, stack| {
        let name: String = (item_block::CLASS.getItemName)(this).unwrap_or_else(|| "null".to_string());
        Some(format!("{name}.{}", block_step::VARIANTS[stack.getItemDamage() as usize]))
    },
    ..item_block::CLASS
};

pub fn create(n: i32) -> Rc<Item> { item_block::construct(&CLASS, n).setMaxDamage(0).setHasSubtypes(true) }
