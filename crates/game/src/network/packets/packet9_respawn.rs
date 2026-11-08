use crate::network::net_handler::NetHandler;
use crate::network::packet::{Packet, PacketBase, PacketInput, PacketOutput};
use jvm::io::IoResult;

#[derive(Clone, Default)]
pub struct Packet9Respawn
{
    m_base: PacketBase,
    pub m_dimension: i8,
}

impl Packet9Respawn
{
    pub fn new(dimensionId: i8) -> Self { Self { m_dimension: dimensionId, ..Self::default() } }
}

impl Packet for Packet9Respawn
{
    fn base(&self) -> &PacketBase { &self.m_base }

    fn readPacketData(&mut self, input: &mut PacketInput) -> IoResult<()>
    {
        self.m_dimension = input.readByte()?;
        Ok(())
    }

    fn writePacketData(&self, output: &mut PacketOutput) -> IoResult<()> { output.writeByte(self.m_dimension as i32) }

    fn processPacket(&self, handler: &dyn NetHandler) { handler.handlePlayerRespawn(self); }

    fn getPacketSize(&self) -> i32 { 1 }
}
