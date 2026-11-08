use crate::entity::{entity_arrow, entity_player};
use crate::item::{ITEM, Item, ItemClass};
use std::rc::Rc;

pub const CLASS: ItemClass = ItemClass {
    m_name: "ItemBow",
    m_parent: Some(&ITEM),
    onItemRightClick: |_, stack, world, player| {
        if entity_player::inventory(player).consumeInventoryItem(Item::arrow().m_shiftedIndex)
        {
            world.playSoundAtEntity(player, "random.bow", 1.0, 1.0 / (Item::itemRand().nextFloat() * 0.4 + 0.8));
            if !world.m_multiplayerWorld.get()
            {
                world.entityJoinedWorld(&entity_arrow::createFromShooter(world, player));
            }
        }
        Some(Rc::clone(stack))
    },
    ..ITEM
};

pub fn create(id: i32) -> Rc<Item>
{
    let item: Rc<Item> = Item::construct(&CLASS, Box::new(()), id);
    item.m_maxStackSize.set(1);
    item
}
