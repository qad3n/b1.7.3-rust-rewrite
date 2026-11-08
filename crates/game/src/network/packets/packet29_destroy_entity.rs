use crate::network::net_handler::NetHandler;
use crate::network::packet::{Packet, PacketBase, PacketInput, PacketOutput};
use jvm::io::IoResult;

#[derive(Clone, Default)]
pub struct Packet29DestroyEntity
{
    m_base: PacketBase,
    pub m_entityId: i32,
}

impl Packet29DestroyEntity
{
    pub fn new(id: i32) -> Self { Self { m_entityId: id, ..Self::default() } }
}

impl Packet for Packet29DestroyEntity
{
    fn base(&self) -> &PacketBase { &self.m_base }

    fn readPacketData(&mut self, input: &mut PacketInput) -> IoResult<()>
    {
        self.m_entityId = input.readInt()?;
        Ok(())
    }

    fn writePacketData(&self, output: &mut PacketOutput) -> IoResult<()> { output.writeInt(self.m_entityId) }

    fn processPacket(&self, handler: &dyn NetHandler) { handler.handleDestroyEntity(self); }

    fn getPacketSize(&self) -> i32 { 4 }
}
