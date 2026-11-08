use crate::network::net_handler::NetHandler;
use crate::network::packet::{Packet, PacketBase, PacketInput, PacketOutput};
use jvm::io::IoResult;

#[derive(Clone, Default)]
pub struct Packet0KeepAlive
{
    m_base: PacketBase,
}

impl Packet for Packet0KeepAlive
{
    fn base(&self) -> &PacketBase { &self.m_base }

    fn readPacketData(&mut self, _input: &mut PacketInput) -> IoResult<()> { Ok(()) }

    fn writePacketData(&self, _output: &mut PacketOutput) -> IoResult<()> { Ok(()) }

    fn processPacket(&self, _handler: &dyn NetHandler) {}

    fn getPacketSize(&self) -> i32 { 0 }
}
