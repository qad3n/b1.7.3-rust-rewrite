use crate::block::{Block, Material};
use crate::item::{ITEM, Item, ItemClass};
use crate::math::math_helper;
use std::rc::Rc;

pub struct DoorExt
{
    m_doorMaterial: Rc<Material>,
}

pub const CLASS: ItemClass = ItemClass {
    m_name: "ItemDoor",
    m_parent: Some(&ITEM),
    onItemUse: |this, stack, player, world, x, mut y, z, face| {
        if face != 1
        {
            return false;
        }
        y += 1;
        let block: Rc<Block> = if Rc::ptr_eq(&this.ext::<DoorExt>().m_doorMaterial, &Material::wood())
        {
            Block::doorWood()
        }
        else
        {
            Block::doorSteel()
        };
        if !block.canPlaceBlockAt(world, x, y, z)
        {
            return false;
        }
        let mut i: i32 =
            math_helper::floor_double(((player.m_rotationYaw.get() + 180.0) * 4.0 / 360.0) as f64 - 0.5) & 3;
        let mut b0: i32 = 0;
        let mut b1: i32 = 0;
        if i == 0
        {
            b1 = 1;
        }
        if i == 1
        {
            b0 = -1;
        }
        if i == 2
        {
            b1 = -1;
        }
        if i == 3
        {
            b0 = 1;
        }
        let j: i32 =
            world.isBlockNormalCube(x - b0, y, z - b1) as i32 + world.isBlockNormalCube(x - b0, y + 1, z - b1) as i32;
        let k: i32 =
            world.isBlockNormalCube(x + b0, y, z + b1) as i32 + world.isBlockNormalCube(x + b0, y + 1, z + b1) as i32;
        let flag: bool = world.getBlockId(x - b0, y, z - b1) == block.m_blockID
            || world.getBlockId(x - b0, y + 1, z - b1) == block.m_blockID;
        let flag1: bool = world.getBlockId(x + b0, y, z + b1) == block.m_blockID
            || world.getBlockId(x + b0, y + 1, z + b1) == block.m_blockID;
        let mut flag2: bool = false;
        if flag && !flag1
        {
            flag2 = true;
        }
        else if k > j
        {
            flag2 = true;
        }
        if flag2
        {
            i = (i - 1) & 3;
            i += 4;
        }
        world.m_editingBlocks.set(true);
        world.setBlockAndMetadataWithNotify(x, y, z, block.m_blockID, i);
        world.setBlockAndMetadataWithNotify(x, y + 1, z, block.m_blockID, i + 8);
        world.m_editingBlocks.set(false);
        world.notifyBlocksOfNeighborChange(x, y, z, block.m_blockID);
        world.notifyBlocksOfNeighborChange(x, y + 1, z, block.m_blockID);
        stack.m_stackSize.set(stack.m_stackSize.get() - 1);
        true
    },
    ..ITEM
};

pub fn create(id: i32, material: Rc<Material>) -> Rc<Item>
{
    let item: Rc<Item> = Item::construct(&CLASS, Box::new(DoorExt { m_doorMaterial: material }), id);
    item.m_maxStackSize.set(1);
    item
}
