use crate::block::{Block, block_juke_box};
use crate::item::{ITEM, Item, ItemClass};
use std::rc::Rc;

pub struct RecordExt
{
    pub m_recordName: String,
}

pub fn recordName(this: &Item) -> &str { &this.ext::<RecordExt>().m_recordName }

pub const CLASS: ItemClass = ItemClass {
    m_name: "ItemRecord",
    m_parent: Some(&ITEM),
    onItemUse: |this, stack, _, world, x, y, z, _| {
        if world.getBlockId(x, y, z) == Block::jukebox().m_blockID && world.getBlockMetadata(x, y, z) == 0
        {
            if world.m_multiplayerWorld.get()
            {
                return true;
            }
            block_juke_box::ejectRecord(world, x, y, z, this.m_shiftedIndex);
            world.doEventFrom(None, 1005, x, y, z, this.m_shiftedIndex);
            stack.m_stackSize.set(stack.m_stackSize.get() - 1);
            return true;
        }
        false
    },
    ..ITEM
};

pub fn create(id: i32, record: &str) -> Rc<Item>
{
    let item: Rc<Item> = Item::construct(&CLASS, Box::new(RecordExt { m_recordName: record.to_string() }), id);
    item.m_maxStackSize.set(1);
    item
}
