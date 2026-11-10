use crate::item::{Item, ItemClass, item_food};
use std::rc::Rc;

pub const CLASS: ItemClass = ItemClass { m_name: "ItemCookie", m_parent: Some(&item_food::CLASS), ..item_food::CLASS };

pub fn create(id: i32, hungerPoints: i32, maxStackSize: i32) -> Rc<Item>
{
    let item: Rc<Item> = item_food::construct(&CLASS, id, hungerPoints);
    item.m_maxStackSize.set(maxStackSize);
    item
}
