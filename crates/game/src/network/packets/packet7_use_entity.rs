use crate::network::net_handler::NetHandler;
use crate::network::packet::{Packet, PacketBase, PacketInput, PacketOutput};
use jvm::io::IoResult;

#[derive(Clone, Default)]
pub struct Packet7UseEntity
{
    m_base: PacketBase,
    pub m_playerEntityId: i32,
    pub m_targetEntity: i32,
    pub m_isLeftClick: i32,
}

impl Packet7UseEntity
{
    pub fn new(playerId: i32, targetId: i32, action: i32) -> Self
    {
        Self { m_playerEntityId: playerId, m_targetEntity: targetId, m_isLeftClick: action, ..Self::default() }
    }
}

impl Packet for Packet7UseEntity
{
    fn base(&self) -> &PacketBase { &self.m_base }

    fn readPacketData(&mut self, input: &mut PacketInput) -> IoResult<()>
    {
        self.m_playerEntityId = input.readInt()?;
        self.m_targetEntity = input.readInt()?;
        self.m_isLeftClick = input.readByte()? as i32;
        Ok(())
    }

    fn writePacketData(&self, output: &mut PacketOutput) -> IoResult<()>
    {
        output.writeInt(self.m_playerEntityId)?;
        output.writeInt(self.m_targetEntity)?;
        output.writeByte(self.m_isLeftClick)
    }

    fn processPacket(&self, handler: &dyn NetHandler) { handler.handleUseEntity(self); }

    fn getPacketSize(&self) -> i32 { 9 }
}
