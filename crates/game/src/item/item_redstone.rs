use crate::block::Block;
use crate::item::{ITEM, Item, ItemClass};
use std::rc::Rc;

pub const CLASS: ItemClass = ItemClass {
    m_name: "ItemRedstone",
    m_parent: Some(&ITEM),
    onItemUse: |_, stack, _, world, mut x, mut y, mut z, face| {
        if world.getBlockId(x, y, z) != Block::snow().m_blockID
        {
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
            if !world.isAirBlock(x, y, z)
            {
                return false;
            }
        }
        if Block::redstoneWire().canPlaceBlockAt(world, x, y, z)
        {
            stack.m_stackSize.set(stack.m_stackSize.get() - 1);
            world.setBlockWithNotify(x, y, z, Block::redstoneWire().m_blockID);
        }
        true
    },
    ..ITEM
};

pub fn create(id: i32) -> Rc<Item> { Item::construct(&CLASS, Box::new(()), id) }
