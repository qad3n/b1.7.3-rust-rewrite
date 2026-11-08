use crate::entity::Entity;
use crate::network::net_handler::NetHandler;
use crate::network::packet::{Packet, PacketBase, PacketInput, PacketOutput};
use jvm::io::IoResult;

#[derive(Clone, Default)]
pub struct Packet19EntityAction
{
    m_base: PacketBase,
    pub m_entityId: i32,
    pub m_state: i32,
}

impl Packet19EntityAction
{
    pub fn new(entity: &Entity, action: i32) -> Self
    {
        Self { m_entityId: entity.m_entityId.get(), m_state: action, ..Self::default() }
    }
}

impl Packet for Packet19EntityAction
{
    fn base(&self) -> &PacketBase { &self.m_base }

    fn readPacketData(&mut self, input: &mut PacketInput) -> IoResult<()>
    {
        self.m_entityId = input.readInt()?;
        self.m_state = input.readByte()? as i32;
        Ok(())
    }

    fn writePacketData(&self, output: &mut PacketOutput) -> IoResult<()>
    {
        output.writeInt(self.m_entityId)?;
        output.writeByte(self.m_state)
    }

    fn processPacket(&self, handler: &dyn NetHandler) { handler.handlePlayerMovementAction(self); }

    fn getPacketSize(&self) -> i32 { 5 }
}
