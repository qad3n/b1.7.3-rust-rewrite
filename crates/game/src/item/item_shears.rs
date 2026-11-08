use crate::block::Block;
use crate::item::{ITEM, Item, ItemClass};
use std::rc::Rc;

pub const CLASS: ItemClass = ItemClass {
    m_name: "ItemShears",
    m_parent: Some(&ITEM),
    onBlockDestroyed: |this, stack, block, x, y, z, entity| {
        if block == Block::leaves().m_blockID || block == Block::web().m_blockID
        {
            stack.damageItem(1, entity);
        }
        (ITEM.onBlockDestroyed)(this, stack, block, x, y, z, entity)
    },
    canHarvestBlock: |_, block| block.m_blockID == Block::web().m_blockID,
    getStrVsBlock: |this, stack, block| {
        if block.m_blockID == Block::web().m_blockID || block.m_blockID == Block::leaves().m_blockID
        {
            return 15.0;
        }
        if block.m_blockID == Block::cloth().m_blockID
        {
            return 5.0;
        }
        (ITEM.getStrVsBlock)(this, stack, block)
    },
    ..ITEM
};

pub fn create(id: i32) -> Rc<Item> { Item::construct(&CLASS, Box::new(()), id).setMaxStackSize(1).setMaxDamage(238) }
