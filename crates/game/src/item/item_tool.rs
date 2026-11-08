use crate::block::Block;
use crate::item::{EnumToolMaterial, ITEM, Item, ItemClass};
use std::rc::Rc;

pub struct ToolExt
{
    m_blocksEffectiveAgainst: Vec<Rc<Block>>,
    m_efficiencyOnProperMaterial: f32,
    m_damageVsEntity: i32,
    pub m_toolMaterial: EnumToolMaterial,
}

pub fn fields(this: &Item) -> &ToolExt { this.ext::<ToolExt>() }

pub const CLASS: ItemClass = ItemClass {
    m_name: "ItemTool",
    m_parent: Some(&ITEM),
    getStrVsBlock: |this, _, block| {
        for effective in &fields(this).m_blocksEffectiveAgainst
        {
            if Rc::ptr_eq(effective, block)
            {
                return fields(this).m_efficiencyOnProperMaterial;
            }
        }
        1.0
    },
    hitEntity: |_, stack, _, attacker| {
        stack.damageItem(2, attacker);
        true
    },
    onBlockDestroyed: |_, stack, _, _, _, _, entity| {
        stack.damageItem(1, entity);
        true
    },
    getDamageVsEntity: |this, _| fields(this).m_damageVsEntity,
    isFull3D: |_| true,
    ..ITEM
};

pub fn construct(
    class: &'static ItemClass,
    id: i32,
    attackDamage: i32,
    tier: EnumToolMaterial,
    effectiveBlocks: Vec<Rc<Block>>,
) -> Rc<Item>
{
    let ext: ToolExt = ToolExt {
        m_blocksEffectiveAgainst: effectiveBlocks,
        m_efficiencyOnProperMaterial: tier.getEfficiencyOnProperMaterial(),
        m_damageVsEntity: attackDamage + tier.getDamageVsEntity(),
        m_toolMaterial: tier,
    };
    let item: Rc<Item> = Item::construct(class, Box::new(ext), id);
    item.m_maxStackSize.set(1);
    item.setMaxDamage(tier.getMaxUses())
}
