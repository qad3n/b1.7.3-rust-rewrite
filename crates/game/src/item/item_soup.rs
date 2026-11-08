use crate::item::{Item, ItemClass, ItemStack, item_food};
use std::rc::Rc;

pub const CLASS: ItemClass = ItemClass {
    m_name: "ItemSoup",
    m_parent: Some(&item_food::CLASS),
    onItemRightClick: |this, stack, world, player| {
        (item_food::CLASS.onItemRightClick)(this, stack, world, player);
        Some(ItemStack::withItem(&Item::bowlEmpty()))
    },
    ..item_food::CLASS
};

pub fn create(id: i32, hungerPoints: i32) -> Rc<Item> { item_food::construct(&CLASS, id, hungerPoints, false) }
