use crate::network::net_handler::NetHandler;
use crate::network::packet::{Packet, PacketBase, PacketInput, PacketOutput};
use jvm::io::IoResult;

#[derive(Clone, Default)]
pub struct Packet38EntityStatus
{
    m_base: PacketBase,
    pub m_entityId: i32,
    pub m_entityStatus: i8,
}

impl Packet38EntityStatus
{
    pub fn new(id: i32, event: i8) -> Self { Self { m_entityId: id, m_entityStatus: event, ..Self::default() } }
}

impl Packet for Packet38EntityStatus
{
    fn base(&self) -> &PacketBase { &self.m_base }

    fn readPacketData(&mut self, input: &mut PacketInput) -> IoResult<()>
    {
        self.m_entityId = input.readInt()?;
        self.m_entityStatus = input.readByte()?;
        Ok(())
    }

    fn writePacketData(&self, output: &mut PacketOutput) -> IoResult<()>
    {
        output.writeInt(self.m_entityId)?;
        output.writeByte(self.m_entityStatus as i32)
    }

    fn processPacket(&self, handler: &dyn NetHandler) { handler.handleEntityEvent(self); }

    fn getPacketSize(&self) -> i32 { 5 }
}
