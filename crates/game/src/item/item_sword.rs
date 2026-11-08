use crate::block::Block;
use crate::item::{EnumToolMaterial, ITEM, Item, ItemClass};
use std::rc::Rc;

pub struct SwordExt
{
    m_weaponDamage: i32,
}

pub const CLASS: ItemClass = ItemClass {
    m_name: "ItemSword",
    m_parent: Some(&ITEM),
    getStrVsBlock: |_, _, block| {
        if block.m_blockID == Block::web().m_blockID
        {
            return 15.0;
        }
        1.5
    },
    hitEntity: |_, stack, _, attacker| {
        stack.damageItem(1, attacker);
        true
    },
    onBlockDestroyed: |_, stack, _, _, _, _, entity| {
        stack.damageItem(2, entity);
        true
    },
    getDamageVsEntity: |this, _| this.ext::<SwordExt>().m_weaponDamage,
    isFull3D: |_| true,
    canHarvestBlock: |_, block| block.m_blockID == Block::web().m_blockID,
    ..ITEM
};

pub fn create(id: i32, tier: EnumToolMaterial) -> Rc<Item>
{
    let item: Rc<Item> =
        Item::construct(&CLASS, Box::new(SwordExt { m_weaponDamage: 4 + tier.getDamageVsEntity() * 2 }), id);
    item.m_maxStackSize.set(1);
    item.setMaxDamage(tier.getMaxUses())
}
