use crate::item::ItemStack;
use crate::network::net_handler::NetHandler;
use crate::network::packet::{Packet, PacketBase, PacketInput, PacketOutput};
// item stacks cross the network threads as plain port data
use crate::port::packet_item_stack::PacketItemStack;
use jvm::io::IoResult;
use std::rc::Rc;

#[derive(Clone, Default)]
pub struct Packet102WindowClick
{
    m_base: PacketBase,
    pub m_window_Id: i32,
    pub m_inventorySlot: i32,
    pub m_mouseClick: i32,
    pub m_action: i16,
    pub m_itemStack: Option<PacketItemStack>,
    pub m_quickMove: bool,
}

impl Packet102WindowClick
{
    pub fn new(
        menuId: i32,
        slotId: i32,
        clickData: i32,
        quickMove: bool,
        item: Option<&Rc<ItemStack>>,
        interactionId: i16,
    ) -> Self
    {
        Self {
            m_window_Id: menuId,
            m_inventorySlot: slotId,
            m_mouseClick: clickData,
            m_itemStack: PacketItemStack::fromOption(item),
            m_action: interactionId,
            m_quickMove: quickMove,
            ..Self::default()
        }
    }
}

impl Packet for Packet102WindowClick
{
    fn base(&self) -> &PacketBase { &self.m_base }

    fn readPacketData(&mut self, input: &mut PacketInput) -> IoResult<()>
    {
        self.m_window_Id = input.readByte()? as i32;
        self.m_inventorySlot = input.readShort()? as i32;
        self.m_mouseClick = input.readByte()? as i32;
        self.m_action = input.readShort()?;
        self.m_quickMove = input.readBoolean()?;
        self.m_itemStack = PacketItemStack::read(input)?;
        Ok(())
    }

    fn writePacketData(&self, output: &mut PacketOutput) -> IoResult<()>
    {
        output.writeByte(self.m_window_Id)?;
        output.writeShort(self.m_inventorySlot)?;
        output.writeByte(self.m_mouseClick)?;
        output.writeShort(self.m_action as i32)?;
        output.writeBoolean(self.m_quickMove)?;
        PacketItemStack::write(self.m_itemStack.as_ref(), output)
    }

    fn processPacket(&self, handler: &dyn NetHandler) { handler.handleInventoryMenuClickSlot(self); }

    fn getPacketSize(&self) -> i32 { 11 }
}
