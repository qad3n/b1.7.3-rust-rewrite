use crate::network::net_handler::NetHandler;
use crate::network::packet::{self, Packet, PacketBase, PacketInput, PacketOutput};
// string length of a field java dereferences without a null check comes from port
use crate::port::network;
use jvm::io::IoResult;
use jvm::string::JString;
use jvm::throwable::OrNpe;

#[derive(Clone, Default)]
pub struct Packet255KickDisconnect
{
    m_base: PacketBase,
    pub m_reason: Option<JString>,
}

impl Packet255KickDisconnect
{
    pub fn new(reason: JString) -> Self { Self { m_reason: Some(reason), ..Self::default() } }
}

impl Packet for Packet255KickDisconnect
{
    fn base(&self) -> &PacketBase { &self.m_base }

    fn readPacketData(&mut self, input: &mut PacketInput) -> IoResult<()>
    {
        self.m_reason = Some(packet::readString(input, 100)?);
        Ok(())
    }

    fn writePacketData(&self, output: &mut PacketOutput) -> IoResult<()>
    {
        packet::writeString(self.m_reason.as_ref().orNpe(), output)
    }

    fn processPacket(&self, handler: &dyn NetHandler) { handler.handleKickDisconnect(self); }

    fn getPacketSize(&self) -> i32 { network::length(&self.m_reason) }
}
