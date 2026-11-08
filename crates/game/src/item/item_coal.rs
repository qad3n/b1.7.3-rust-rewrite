use crate::item::{ITEM, Item, ItemClass};
use std::rc::Rc;

pub const CLASS: ItemClass = ItemClass {
    m_name: "ItemCoal",
    m_parent: Some(&ITEM),
    getItemNameIS: |_, stack| {
        if stack.getItemDamage() == 1
        {
            return Some("item.charcoal".to_string());
        }
        Some("item.coal".to_string())
    },
    ..ITEM
};

pub fn create(id: i32) -> Rc<Item> { Item::construct(&CLASS, Box::new(()), id).setHasSubtypes(true).setMaxDamage(0) }
