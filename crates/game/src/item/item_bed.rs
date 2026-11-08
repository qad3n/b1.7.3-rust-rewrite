use crate::block::Block;
use crate::item::{ITEM, Item, ItemClass};
use crate::math::math_helper;
use std::rc::Rc;

pub const CLASS: ItemClass = ItemClass {
    m_name: "ItemBed",
    m_parent: Some(&ITEM),
    onItemUse: |_, stack, player, world, x, mut y, z, face| {
        if face != 1
        {
            return false;
        }
        y += 1;
        let blockBed: Rc<Block> = Block::blockBed();
        let n: i32 = math_helper::floor_double((player.m_rotationYaw.get() * 4.0 / 360.0) as f64 + 0.5) & 3;
        let mut n2: i32 = 0;
        let mut n3: i32 = 0;
        if n == 0
        {
            n3 = 1;
        }
        if n == 1
        {
            n2 = -1;
        }
        if n == 2
        {
            n3 = -1;
        }
        if n == 3
        {
            n2 = 1;
        }
        if world.isAirBlock(x, y, z)
            && world.isAirBlock(x + n2, y, z + n3)
            && world.isBlockNormalCube(x, y - 1, z)
            && world.isBlockNormalCube(x + n2, y - 1, z + n3)
        {
            world.setBlockAndMetadataWithNotify(x, y, z, blockBed.m_blockID, n);
            world.setBlockAndMetadataWithNotify(x + n2, y, z + n3, blockBed.m_blockID, n + 8);
            stack.m_stackSize.set(stack.m_stackSize.get() - 1);
            return true;
        }
        false
    },
    ..ITEM
};

pub fn create(id: i32) -> Rc<Item> { Item::construct(&CLASS, Box::new(()), id) }
