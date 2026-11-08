use crate::entity::{Entity, entity_pig};
use crate::item::{ITEM, Item, ItemClass, ItemStack};
use std::rc::Rc;

fn saddleEntity(_this: &Rc<Item>, stack: &Rc<ItemStack>, entity: &Rc<Entity>)
{
    if entity.instanceOf("EntityPig") && !entity_pig::getSaddled(entity)
    {
        entity_pig::setSaddled(entity, true);
        stack.m_stackSize.set(stack.m_stackSize.get() - 1);
    }
}

pub const CLASS: ItemClass = ItemClass {
    m_name: "ItemSaddle",
    m_parent: Some(&ITEM),
    saddleEntity,
    hitEntity: |this, stack, target, _| {
        this.saddleEntity(stack, target);
        true
    },
    ..ITEM
};

pub fn create(id: i32) -> Rc<Item>
{
    let item: Rc<Item> = Item::construct(&CLASS, Box::new(()), id);
    item.m_maxStackSize.set(1);
    item
}
