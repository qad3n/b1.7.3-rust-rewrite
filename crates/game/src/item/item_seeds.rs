use crate::block::Block;
use crate::item::{ITEM, Item, ItemClass};
use std::rc::Rc;

pub struct SeedsExt
{
    m_plantBlock: i32,
}

pub const CLASS: ItemClass = ItemClass {
    m_name: "ItemSeeds",
    m_parent: Some(&ITEM),
    onItemUse: |this, stack, _, world, x, y, z, face| {
        if face != 1
        {
            return false;
        }
        let n: i32 = world.getBlockId(x, y, z);
        if n == Block::tilledField().m_blockID && world.isAirBlock(x, y + 1, z)
        {
            world.setBlockWithNotify(x, y + 1, z, this.ext::<SeedsExt>().m_plantBlock);
            stack.m_stackSize.set(stack.m_stackSize.get() - 1);
            return true;
        }
        false
    },
    ..ITEM
};

pub fn create(item: i32, plantBlock: i32) -> Rc<Item>
{
    Item::construct(&CLASS, Box::new(SeedsExt { m_plantBlock: plantBlock }), item)
}
