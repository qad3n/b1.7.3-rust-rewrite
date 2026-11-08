use crate::network::net_handler::NetHandler;
use crate::network::packet::{Packet, PacketBase, PacketInput, PacketOutput};
use jvm::io::IoResult;

#[derive(Clone, Default)]
pub struct Packet101CloseWindow
{
    m_base: PacketBase,
    pub m_windowId: i32,
}

impl Packet101CloseWindow
{
    pub fn new(id: i32) -> Self { Self { m_windowId: id, ..Self::default() } }
}

impl Packet for Packet101CloseWindow
{
    fn base(&self) -> &PacketBase { &self.m_base }

    fn readPacketData(&mut self, input: &mut PacketInput) -> IoResult<()>
    {
        self.m_windowId = input.readByte()? as i32;
        Ok(())
    }

    fn writePacketData(&self, output: &mut PacketOutput) -> IoResult<()> { output.writeByte(self.m_windowId) }

    fn processPacket(&self, handler: &dyn NetHandler) { handler.handleCloseInventoryMenu(self); }

    fn getPacketSize(&self) -> i32 { 1 }
}
