use crate::network::net_handler::NetHandler;
use crate::network::packet::{Packet, PacketBase, PacketInput, PacketOutput};
// string length of a field java dereferences without a null check comes from port
use crate::port::network;
use jvm::io::IoResult;
use jvm::string::JString;
use jvm::throwable::OrNpe;

#[derive(Clone, Default)]
pub struct Packet100OpenWindow
{
    m_base: PacketBase,
    pub m_windowId: i32,
    pub m_inventoryType: i32,
    pub m_windowTitle: Option<JString>,
    pub m_slotsCount: i32,
}

impl Packet100OpenWindow
{
    pub fn new(n: i32, n2: i32, string: Option<JString>, n3: i32) -> Self
    {
        Self { m_windowId: n, m_inventoryType: n2, m_windowTitle: string, m_slotsCount: n3, ..Self::default() }
    }
}

impl Packet for Packet100OpenWindow
{
    fn base(&self) -> &PacketBase { &self.m_base }

    fn readPacketData(&mut self, input: &mut PacketInput) -> IoResult<()>
    {
        self.m_windowId = input.readByte()? as i32;
        self.m_inventoryType = input.readByte()? as i32;
        self.m_windowTitle = Some(input.readUTF()?);
        self.m_slotsCount = input.readByte()? as i32;
        Ok(())
    }

    fn writePacketData(&self, output: &mut PacketOutput) -> IoResult<()>
    {
        output.writeByte(self.m_windowId)?;
        output.writeByte(self.m_inventoryType)?;
        output.writeUTF(self.m_windowTitle.as_ref().orNpe())?;
        output.writeByte(self.m_slotsCount)
    }

    fn processPacket(&self, handler: &dyn NetHandler) { handler.handleOpenInventoryMenu(self); }

    fn getPacketSize(&self) -> i32 { 3 + network::length(&self.m_windowTitle) }
}
