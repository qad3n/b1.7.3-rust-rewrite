use crate::item::ItemStack;
use crate::network::packet::{PacketInput, PacketOutput};
use jvm::io::IoResult;
use std::rc::Rc;

#[derive(Clone, Copy, Debug)]
pub struct PacketItemStack
{
    pub m_itemID: i32,
    pub m_stackSize: i32,
    pub m_itemDamage: i32,
}

impl PacketItemStack
{
    pub fn from(item: &ItemStack) -> PacketItemStack
    {
        PacketItemStack {
            m_itemID: item.m_itemID.get(),
            m_stackSize: item.m_stackSize.get(),
            m_itemDamage: item.getItemDamage(),
        }
    }

    pub fn fromOption(item: Option<&Rc<ItemStack>>) -> Option<PacketItemStack> { item.map(|item| Self::from(item)) }

    pub fn toItemStack(&self) -> Rc<ItemStack>
    {
        ItemStack::withDamage(self.m_itemID, self.m_stackSize, self.m_itemDamage)
    }

    pub fn toItemStackOption(item: Option<PacketItemStack>) -> Option<Rc<ItemStack>>
    {
        item.map(|item| item.toItemStack())
    }

    pub fn read(input: &mut PacketInput) -> IoResult<Option<PacketItemStack>>
    {
        let s: i16 = input.readShort()?;
        if s >= 0
        {
            let by: i8 = input.readByte()?;
            let s2: i16 = input.readShort()?;
            return Ok(Some(PacketItemStack { m_itemID: s as i32, m_stackSize: by as i32, m_itemDamage: s2 as i32 }));
        }
        Ok(None)
    }

    pub fn write(item: Option<&PacketItemStack>, output: &mut PacketOutput) -> IoResult<()>
    {
        match item
        {
            None => output.writeShort(-1),
            Some(item) =>
            {
                output.writeShort(item.m_itemID)?;
                output.writeByte(item.m_stackSize)?;
                output.writeShort(item.m_itemDamage)
            }
        }
    }
}
