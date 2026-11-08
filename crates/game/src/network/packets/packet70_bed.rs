use crate::network::net_handler::NetHandler;
use crate::network::packet::{Packet, PacketBase, PacketInput, PacketOutput};
use jvm::io::IoResult;

pub const EVENT_MESSAGES: [Option<&str>; 3] = [Some("tile.bed.notValid"), None, None];

#[derive(Clone, Default)]
pub struct Packet70Bed
{
    m_base: PacketBase,
    pub m_event: i32,
}

impl Packet70Bed
{
    pub fn new(n: i32) -> Self { Self { m_event: n, ..Self::default() } }
}

impl Packet for Packet70Bed
{
    fn base(&self) -> &PacketBase { &self.m_base }

    fn readPacketData(&mut self, input: &mut PacketInput) -> IoResult<()>
    {
        self.m_event = input.readByte()? as i32;
        Ok(())
    }

    fn writePacketData(&self, output: &mut PacketOutput) -> IoResult<()> { output.writeByte(self.m_event) }

    fn processPacket(&self, handler: &dyn NetHandler) { handler.handleGameEvent(self); }

    fn getPacketSize(&self) -> i32 { 1 }
}
