use crate::entity::entity_egg;
use crate::item::{ITEM, Item, ItemClass};
use std::rc::Rc;

pub const CLASS: ItemClass = ItemClass {
    m_name: "ItemEgg",
    m_parent: Some(&ITEM),
    onItemRightClick: |_, stack, world, player| {
        stack.m_stackSize.set(stack.m_stackSize.get() - 1);
        world.playSoundAtEntity(player, "random.bow", 0.5, 0.4 / (Item::itemRand().nextFloat() * 0.4 + 0.8));
        if !world.m_multiplayerWorld.get()
        {
            world.entityJoinedWorld(&entity_egg::createFromThrower(world, player));
        }
        Some(Rc::clone(stack))
    },
    ..ITEM
};

pub fn create(id: i32) -> Rc<Item>
{
    let item: Rc<Item> = Item::construct(&CLASS, Box::new(()), id);
    item.m_maxStackSize.set(16);
    item
}
