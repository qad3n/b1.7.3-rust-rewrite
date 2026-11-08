use crate::block::block_rail;
use crate::entity::{Entity, entity_minecart};
use crate::item::{ITEM, Item, ItemClass};
use std::rc::Rc;

pub struct MinecartExt
{
    pub m_minecartType: i32,
}

pub const CLASS: ItemClass = ItemClass {
    m_name: "ItemMinecart",
    m_parent: Some(&ITEM),
    onItemUse: |this, stack, _, world, x, y, z, _| {
        let n: i32 = world.getBlockId(x, y, z);
        if block_rail::isRailBlock(n)
        {
            if !world.m_multiplayerWorld.get()
            {
                let cart: Rc<Entity> = entity_minecart::createAt(
                    world,
                    (x as f32 + 0.5) as f64,
                    (y as f32 + 0.5) as f64,
                    (z as f32 + 0.5) as f64,
                    this.ext::<MinecartExt>().m_minecartType,
                );
                world.entityJoinedWorld(&cart);
            }
            stack.m_stackSize.set(stack.m_stackSize.get() - 1);
            return true;
        }
        false
    },
    ..ITEM
};

pub fn create(id: i32, n: i32) -> Rc<Item>
{
    let item: Rc<Item> = Item::construct(&CLASS, Box::new(MinecartExt { m_minecartType: n }), id);
    item.m_maxStackSize.set(1);
    item
}
