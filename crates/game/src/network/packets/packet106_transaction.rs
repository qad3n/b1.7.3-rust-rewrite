use crate::network::net_handler::NetHandler;
use crate::network::packet::{Packet, PacketBase, PacketInput, PacketOutput};
use jvm::io::IoResult;

#[derive(Clone, Default)]
pub struct Packet106Transaction
{
    m_base: PacketBase,
    pub m_windowId: i32,
    pub m_interactionId: i16,
    pub m_accepted: bool,
}

impl Packet106Transaction
{
    pub fn new(menuId: i32, interactionId: i16, accepted: bool) -> Self
    {
        Self { m_windowId: menuId, m_interactionId: interactionId, m_accepted: accepted, ..Self::default() }
    }
}

impl Packet for Packet106Transaction
{
    fn base(&self) -> &PacketBase { &self.m_base }

    fn readPacketData(&mut self, input: &mut PacketInput) -> IoResult<()>
    {
        self.m_windowId = input.readByte()? as i32;
        self.m_interactionId = input.readShort()?;
        self.m_accepted = input.readByte()? != 0;
        Ok(())
    }

    fn writePacketData(&self, output: &mut PacketOutput) -> IoResult<()>
    {
        output.writeByte(self.m_windowId)?;
        output.writeShort(self.m_interactionId as i32)?;
        output.writeByte(if self.m_accepted { 1 } else { 0 })
    }

    fn processPacket(&self, handler: &dyn NetHandler) { handler.handleInventoryMenuConfirm(self); }

    fn getPacketSize(&self) -> i32 { 4 }
}
