use crate::network::net_handler::NetHandler;
use crate::network::packet::{self, Packet, PacketBase, PacketInput, PacketOutput};
// string length of a field java dereferences without a null check comes from port
use crate::port::network;
use jvm::io::IoResult;
use jvm::string::JString;
use jvm::throwable::OrNpe;

#[derive(Clone, Default)]
pub struct Packet2Handshake
{
    m_base: PacketBase,
    pub m_username: Option<JString>,
}

impl Packet2Handshake
{
    pub fn new(key: JString) -> Self { Self { m_username: Some(key), ..Self::default() } }
}

impl Packet for Packet2Handshake
{
    fn base(&self) -> &PacketBase { &self.m_base }

    fn readPacketData(&mut self, input: &mut PacketInput) -> IoResult<()>
    {
        self.m_username = Some(packet::readString(input, 32)?);
        Ok(())
    }

    fn writePacketData(&self, output: &mut PacketOutput) -> IoResult<()>
    {
        packet::writeString(self.m_username.as_ref().orNpe(), output)
    }

    fn processPacket(&self, handler: &dyn NetHandler) { handler.handleHandshake(self); }

    fn getPacketSize(&self) -> i32 { 4 + network::length(&self.m_username) + 4 }
}
