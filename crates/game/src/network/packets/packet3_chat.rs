use crate::network::net_handler::NetHandler;
use crate::network::packet::{self, Packet, PacketBase, PacketInput, PacketOutput};
// string length of a field java dereferences without a null check comes from port
use crate::port::network;
use jvm::io::IoResult;
use jvm::string::JString;
use jvm::throwable::OrNpe;

#[derive(Clone, Default)]
pub struct Packet3Chat
{
    m_base: PacketBase,
    pub m_message: Option<JString>,
}

impl Packet3Chat
{
    pub fn new(mut message: JString) -> Self
    {
        if message.length() > 119
        {
            message = message.substring(0, 119);
        }
        Self { m_message: Some(message), ..Self::default() }
    }
}

impl Packet for Packet3Chat
{
    fn base(&self) -> &PacketBase { &self.m_base }

    fn readPacketData(&mut self, input: &mut PacketInput) -> IoResult<()>
    {
        self.m_message = Some(packet::readString(input, 119)?);
        Ok(())
    }

    fn writePacketData(&self, output: &mut PacketOutput) -> IoResult<()>
    {
        packet::writeString(self.m_message.as_ref().orNpe(), output)
    }

    fn processPacket(&self, handler: &dyn NetHandler) { handler.handleChat(self); }

    fn getPacketSize(&self) -> i32 { network::length(&self.m_message) }
}
