use crate::block::Block;
use crate::item::{EnumToolMaterial, ITEM, Item, ItemClass};
use std::rc::Rc;

pub struct SwordExt
{
    m_weaponDamage: i32,
    // rewrite: kept so mod harvest rules can read a sword's tier
    pub m_toolMaterial: EnumToolMaterial,
}

pub const CLASS: ItemClass = ItemClass
{
    m_name: "ItemSword",
    m_parent: Some(&ITEM),
    getStrVsBlock: |_, _, block|
    {
        if block.m_blockID == Block::WEB
        {
            return 15.0;
        }

        1.5
    },

    hitEntity: |_, stack, _, attacker|
    {
        stack.damageItem(1, attacker);
        true
    },

    onBlockDestroyed: |_, stack, _, _, _, _, entity|
    {
        stack.damageItem(2, entity);
        true
    },
    getDamageVsEntity: |this, _| this.ext::<SwordExt>().m_weaponDamage,
    isFull3D: |_| true,
    canHarvestBlock: |_, block| block.m_blockID == Block::WEB,
    ..ITEM
};

pub fn create(id: i32, tier: EnumToolMaterial) -> Rc<Item>
{
    let ext: SwordExt = SwordExt { m_weaponDamage: 4 + tier.getDamageVsEntity() * 2, m_toolMaterial: tier };
    let item: Rc<Item> = Item::construct(&CLASS, Box::new(ext), id);
    item.m_maxStackSize.set(1);
    item.setMaxDamage(tier.getMaxUses())
}
