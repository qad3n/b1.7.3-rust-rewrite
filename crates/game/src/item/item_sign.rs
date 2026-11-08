use crate::block::Block;
use crate::item::{ITEM, Item, ItemClass};
use crate::math::math_helper;
use crate::tileentity::TileEntity;
use std::rc::Rc;

pub const CLASS: ItemClass = ItemClass {
    m_name: "ItemSign",
    m_parent: Some(&ITEM),
    onItemUse: |_, stack, player, world, mut x, mut y, mut z, face| {
        if face == 0
        {
            return false;
        }
        if !world.getBlockMaterial(x, y, z).isSolid()
        {
            return false;
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
        if !Block::signPost().canPlaceBlockAt(world, x, y, z)
        {
            return false;
        }
        if face == 1
        {
            let rotation: i32 =
                math_helper::floor_double(((player.m_rotationYaw.get() + 180.0) * 16.0 / 360.0) as f64 + 0.5) & 0xF;
            world.setBlockAndMetadataWithNotify(x, y, z, Block::signPost().m_blockID, rotation);
        }
        else
        {
            world.setBlockAndMetadataWithNotify(x, y, z, Block::signWall().m_blockID, face);
        }
        stack.m_stackSize.set(stack.m_stackSize.get() - 1);
        if let Some(tileEntitySign) = TileEntity::castNullable(world.getBlockTileEntity(x, y, z), "TileEntitySign")
        {
            player.displayGUIEditSign(&tileEntitySign);
        }
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
