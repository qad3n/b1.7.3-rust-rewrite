use crate::item::ItemStack;
use crate::network::net_handler::NetHandler;
use crate::network::packet::{Packet, PacketBase, PacketInput, PacketOutput};
// item stacks cross the network threads as plain port data
use crate::port::packet_item_stack::PacketItemStack;
use jvm::io::IoResult;
use std::rc::Rc;

#[derive(Clone, Default)]
pub struct Packet103SetSlot
{
    m_base: PacketBase,
    pub m_windowId: i32,
    pub m_itemSlot: i32,
    pub m_myItemStack: Option<PacketItemStack>,
}

impl Packet103SetSlot
{
    pub fn new(menuId: i32, slot: i32, item: Option<&Rc<ItemStack>>) -> Self
    {
        Self {
            m_windowId: menuId,
            m_itemSlot: slot,
            m_myItemStack: item.map(|item| PacketItemStack::from(&item.copy())),
            ..Self::default()
        }
    }
}

impl Packet for Packet103SetSlot
{
    fn base(&self) -> &PacketBase { &self.m_base }

    fn readPacketData(&mut self, input: &mut PacketInput) -> IoResult<()>
    {
        self.m_windowId = input.readByte()? as i32;
        self.m_itemSlot = input.readShort()? as i32;
        self.m_myItemStack = PacketItemStack::read(input)?;
        Ok(())
    }

    fn writePacketData(&self, output: &mut PacketOutput) -> IoResult<()>
    {
        output.writeByte(self.m_windowId)?;
        output.writeShort(self.m_itemSlot)?;
        PacketItemStack::write(self.m_myItemStack.as_ref(), output)
    }

    fn processPacket(&self, handler: &dyn NetHandler) { handler.handleInventoryMenuSlotContent(self); }

    fn getPacketSize(&self) -> i32 { 8 }
}
