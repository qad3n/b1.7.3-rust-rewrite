use crate::network::net_handler::NetHandler;
use crate::network::packet::{Packet, PacketBase, PacketInput, PacketOutput};
use jvm::io::IoResult;

#[derive(Clone, Default)]
pub struct Packet22Collect
{
    m_base: PacketBase,
    pub m_collectedEntityId: i32,
    pub m_collectorEntityId: i32,
}

impl Packet22Collect
{
    pub fn new(id: i32, collectorId: i32) -> Self
    {
        Self { m_collectedEntityId: id, m_collectorEntityId: collectorId, ..Self::default() }
    }
}

impl Packet for Packet22Collect
{
    fn base(&self) -> &PacketBase { &self.m_base }

    fn readPacketData(&mut self, input: &mut PacketInput) -> IoResult<()>
    {
        self.m_collectedEntityId = input.readInt()?;
        self.m_collectorEntityId = input.readInt()?;
        Ok(())
    }

    fn writePacketData(&self, output: &mut PacketOutput) -> IoResult<()>
    {
        output.writeInt(self.m_collectedEntityId)?;
        output.writeInt(self.m_collectorEntityId)
    }

    fn processPacket(&self, handler: &dyn NetHandler) { handler.handleCollect(self); }

    fn getPacketSize(&self) -> i32 { 8 }
}
