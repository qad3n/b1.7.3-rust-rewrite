use crate::block::Block;
use crate::item::{ITEM, Item, ItemClass};
use std::rc::Rc;

pub const CLASS: ItemClass = ItemClass {
    m_name: "ItemShears",
    m_parent: Some(&ITEM),
    onBlockDestroyed: |this, stack, block, x, y, z, entity| {
        if block == Block::LEAVES || block == Block::WEB
        {
            stack.damageItem(1, entity);
        }
        (ITEM.onBlockDestroyed)(this, stack, block, x, y, z, entity)
    },
    canHarvestBlock: |_, block| block.m_blockID == Block::WEB,
    getStrVsBlock: |this, stack, block| {
        if block.m_blockID == Block::WEB || block.m_blockID == Block::LEAVES
        {
            return 15.0;
        }
        if block.m_blockID == Block::CLOTH
        {
            return 5.0;
        }
        (ITEM.getStrVsBlock)(this, stack, block)
    },
    ..ITEM
};

pub fn create(id: i32) -> Rc<Item> { Item::construct(&CLASS, Box::new(()), id).setMaxStackSize(1).setMaxDamage(238) }
