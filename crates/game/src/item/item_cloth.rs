use crate::block::{Block, block_cloth};
use crate::item::{Item, ItemClass, item_block, item_dye};
use std::rc::Rc;

pub const CLASS: ItemClass = ItemClass {
    m_name: "ItemCloth",
    m_parent: Some(&item_block::CLASS),
    getIconFromDamage: |_, metadata| {
        Block::cloth().getBlockTextureFromSideAndMetadata(2, block_cloth::getBlockMetadataForItemMetadata(metadata))
    },
    getPlacedBlockMetadata: |_, metadata| metadata,
    getItemNameIS: |this, stack| {
        let name: String = (item_block::CLASS.getItemName)(this).unwrap_or_else(|| "null".to_string());
        Some(format!(
            "{name}.{}",
            item_dye::DYE_COLORS[block_cloth::getBlockMetadataForItemMetadata(stack.getItemDamage()) as usize]
        ))
    },
    ..item_block::CLASS
};

pub fn create(n: i32) -> Rc<Item> { item_block::construct(&CLASS, n).setMaxDamage(0).setHasSubtypes(true) }
