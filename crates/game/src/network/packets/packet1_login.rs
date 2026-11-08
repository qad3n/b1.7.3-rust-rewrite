use crate::network::net_handler::NetHandler;
use crate::network::packet::{self, Packet, PacketBase, PacketInput, PacketOutput};
// string length of a field java dereferences without a null check comes from port
use crate::port::network;
use jvm::io::IoResult;
use jvm::string::JString;
use jvm::throwable::OrNpe;

#[derive(Clone, Default)]
pub struct Packet1Login
{
    m_base: PacketBase,
    pub m_protocolVersion: i32,
    pub m_username: Option<JString>,
    pub m_mapSeed: i64,
    pub m_dimension: i8,
}

impl Packet1Login
{
    pub fn withSeed(username: Option<JString>, id: i32, l: i64, dimensionId: i8) -> Self
    {
        Self { m_username: username, m_protocolVersion: id, m_mapSeed: l, m_dimension: dimensionId, ..Self::default() }
    }

    pub fn new(username: JString, gameVersion: i32) -> Self
    {
        Self { m_username: Some(username), m_protocolVersion: gameVersion, ..Self::default() }
    }
}

impl Packet for Packet1Login
{
    fn base(&self) -> &PacketBase { &self.m_base }

    fn readPacketData(&mut self, input: &mut PacketInput) -> IoResult<()>
    {
        self.m_protocolVersion = input.readInt()?;
        self.m_username = Some(packet::readString(input, 16)?);
        self.m_mapSeed = input.readLong()?;
        self.m_dimension = input.readByte()?;
        Ok(())
    }

    fn writePacketData(&self, output: &mut PacketOutput) -> IoResult<()>
    {
        output.writeInt(self.m_protocolVersion)?;
        packet::writeString(self.m_username.as_ref().orNpe(), output)?;
        output.writeLong(self.m_mapSeed)?;
        output.writeByte(self.m_dimension as i32)
    }

    fn processPacket(&self, handler: &dyn NetHandler) { handler.handleLogin(self); }

    fn getPacketSize(&self) -> i32 { 4 + network::length(&self.m_username) + 4 + 5 }
}
