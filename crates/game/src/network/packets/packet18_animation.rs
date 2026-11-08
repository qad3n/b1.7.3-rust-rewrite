use crate::entity::Entity;
use crate::network::net_handler::NetHandler;
use crate::network::packet::{Packet, PacketBase, PacketInput, PacketOutput};
use jvm::io::IoResult;

#[derive(Clone, Default)]
pub struct Packet18Animation
{
    m_base: PacketBase,
    pub m_entityId: i32,
    pub m_animate: i32,
}

impl Packet18Animation
{
    pub fn new(entity: &Entity, action: i32) -> Self
    {
        Self { m_entityId: entity.m_entityId.get(), m_animate: action, ..Self::default() }
    }
}

impl Packet for Packet18Animation
{
    fn base(&self) -> &PacketBase { &self.m_base }

    fn readPacketData(&mut self, input: &mut PacketInput) -> IoResult<()>
    {
        self.m_entityId = input.readInt()?;
        self.m_animate = input.readByte()? as i32;
        Ok(())
    }

    fn writePacketData(&self, output: &mut PacketOutput) -> IoResult<()>
    {
        output.writeInt(self.m_entityId)?;
        output.writeByte(self.m_animate)
    }

    fn processPacket(&self, handler: &dyn NetHandler) { handler.handleArmAnimation(self); }

    fn getPacketSize(&self) -> i32 { 5 }
}
