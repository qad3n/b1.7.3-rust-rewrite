use crate::block::Block;
use crate::item::{EnumToolMaterial, Item, ItemClass, item_tool};
use std::rc::Rc;

pub const CLASS: ItemClass = ItemClass { m_name: "ItemAxe", m_parent: Some(&item_tool::CLASS), ..item_tool::CLASS };

pub fn create(id: i32, tier: EnumToolMaterial) -> Rc<Item>
{
    item_tool::construct(&CLASS, id, 3, tier, vec![Block::planks(), Block::bookShelf(), Block::wood(), Block::chest()])
}
