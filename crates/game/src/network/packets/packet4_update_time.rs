use crate::network::net_handler::NetHandler;
use crate::network::packet::{Packet, PacketBase, PacketInput, PacketOutput};
use jvm::io::IoResult;

#[derive(Clone, Default)]
pub struct Packet4UpdateTime
{
    m_base: PacketBase,
    pub m_time: i64,
}

impl Packet4UpdateTime
{
    pub fn new(l: i64) -> Self { Self { m_time: l, ..Self::default() } }
}

impl Packet for Packet4UpdateTime
{
    fn base(&self) -> &PacketBase { &self.m_base }

    fn readPacketData(&mut self, input: &mut PacketInput) -> IoResult<()>
    {
        self.m_time = input.readLong()?;
        Ok(())
    }

    fn writePacketData(&self, output: &mut PacketOutput) -> IoResult<()> { output.writeLong(self.m_time) }

    fn processPacket(&self, handler: &dyn NetHandler) { handler.handleUpdateTime(self); }

    fn getPacketSize(&self) -> i32 { 8 }
}
