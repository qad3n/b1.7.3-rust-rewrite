use crate::network::net_handler::NetHandler;
use crate::network::packet::{Packet, PacketBase, PacketInput, PacketOutput};
use jvm::io::IoResult;

#[derive(Clone, Default)]
pub struct Packet8UpdateHealth
{
    m_base: PacketBase,
    pub m_healthMP: i32,
}

impl Packet8UpdateHealth
{
    pub fn new(health: i32) -> Self { Self { m_healthMP: health, ..Self::default() } }
}

impl Packet for Packet8UpdateHealth
{
    fn base(&self) -> &PacketBase { &self.m_base }

    fn readPacketData(&mut self, input: &mut PacketInput) -> IoResult<()>
    {
        self.m_healthMP = input.readShort()? as i32;
        Ok(())
    }

    fn writePacketData(&self, output: &mut PacketOutput) -> IoResult<()> { output.writeShort(self.m_healthMP) }

    fn processPacket(&self, handler: &dyn NetHandler) { handler.handleHealth(self); }

    fn getPacketSize(&self) -> i32 { 2 }
}
