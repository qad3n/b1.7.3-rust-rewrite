use crate::item::{ITEM, Item, ItemClass};
use std::any::Any;
use std::rc::Rc;

pub const CLASS: ItemClass = ItemClass {
    m_name: "ItemMapBase",
    m_parent: Some(&ITEM),
    isNetworkSynced: |_| true,
    getUpdatePacket: |_, _, _, _| None,
    ..ITEM
};

pub fn construct(class: &'static ItemClass, ext: Box<dyn Any>, id: i32) -> Rc<Item> { Item::construct(class, ext, id) }
