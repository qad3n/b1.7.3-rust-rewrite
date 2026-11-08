use crate::entity::{Entity, entity_painting};
use crate::item::{ITEM, Item, ItemClass};
use std::rc::Rc;

pub const CLASS: ItemClass = ItemClass {
    m_name: "ItemPainting",
    m_parent: Some(&ITEM),
    onItemUse: |_, stack, _, world, x, y, z, face| {
        if face == 0
        {
            return false;
        }
        if face == 1
        {
            return false;
        }
        let mut b0: i8 = 0;
        if face == 4
        {
            b0 = 1;
        }
        if face == 3
        {
            b0 = 2;
        }
        if face == 5
        {
            b0 = 3;
        }
        let entitypainting: Rc<Entity> = entity_painting::createAt(world, x, y, z, b0 as i32);
        if entity_painting::canSurvive(&entitypainting)
        {
            if !world.m_multiplayerWorld.get()
            {
                world.entityJoinedWorld(&entitypainting);
            }
            stack.m_stackSize.set(stack.m_stackSize.get() - 1);
        }
        true
    },
    ..ITEM
};

pub fn create(id: i32) -> Rc<Item> { Item::construct(&CLASS, Box::new(()), id) }
