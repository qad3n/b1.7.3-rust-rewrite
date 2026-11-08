use crate::item::{ITEM, Item, ItemClass};
use std::rc::Rc;

const DAMAGE_REDUCE_AMOUNT_ARRAY: [i32; 4] = [3, 8, 6, 3];
const MAX_DAMAGE_ARRAY: [i32; 4] = [11, 16, 15, 13];

pub struct ArmorExt
{
    pub m_armorLevel: i32,
    pub m_armorType: i32,
    pub m_damageReduceAmount: i32,
    pub m_renderIndex: i32,
}

pub fn fields(this: &Item) -> &ArmorExt { this.ext::<ArmorExt>() }

pub const CLASS: ItemClass = ItemClass { m_name: "ItemArmor", m_parent: Some(&ITEM), ..ITEM };

pub fn create(id: i32, durability: i32, materialId: i32, slot: i32) -> Rc<Item>
{
    let ext: ArmorExt = ArmorExt {
        m_armorLevel: durability,
        m_armorType: slot,
        m_damageReduceAmount: DAMAGE_REDUCE_AMOUNT_ARRAY[slot as usize],
        m_renderIndex: materialId,
    };
    let item: Rc<Item> =
        Item::construct(&CLASS, Box::new(ext), id).setMaxDamage(MAX_DAMAGE_ARRAY[slot as usize] * 3 << durability);
    item.m_maxStackSize.set(1);
    item
}
