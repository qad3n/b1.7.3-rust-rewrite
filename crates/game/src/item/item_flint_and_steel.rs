use crate::block::Block;
use crate::item::{ITEM, Item, ItemClass};
use std::rc::Rc;

pub const CLASS: ItemClass = ItemClass {
    m_name: "ItemFlintAndSteel",
    m_parent: Some(&ITEM),
    onItemUse: |_, stack, player, world, mut x, mut y, mut z, face| {
        if face == 0
        {
            y -= 1;
        }
        if face == 1
        {
            y += 1;
        }
        if face == 2
        {
            z -= 1;
        }
        if face == 3
        {
            z += 1;
        }
        if face == 4
        {
            x -= 1;
        }
        if face == 5
        {
            x += 1;
        }
        let i: i32 = world.getBlockId(x, y, z);
        if i == 0
        {
            world.playSoundEffect(
                x as f64 + 0.5,
                y as f64 + 0.5,
                z as f64 + 0.5,
                "fire.ignite",
                1.0,
                Item::itemRand().nextFloat() * 0.4 + 0.8,
            );
            world.setBlockWithNotify(x, y, z, Block::fire().m_blockID);
        }
        stack.damageItem(1, player);
        true
    },
    ..ITEM
};

pub fn create(id: i32) -> Rc<Item>
{
    let item: Rc<Item> = Item::construct(&CLASS, Box::new(()), id);
    item.m_maxStackSize.set(1);
    item.setMaxDamage(64)
}
