use crate::entity::Entity;
use crate::network::net_handler::NetHandler;
use crate::network::packet::{Packet, PacketBase, PacketInput, PacketOutput};
use jvm::io::IoResult;

#[derive(Clone, Default)]
pub struct Packet28EntityVelocity
{
    m_base: PacketBase,
    pub m_entityId: i32,
    pub m_motionX: i32,
    pub m_motionY: i32,
    pub m_motionZ: i32,
}

impl Packet28EntityVelocity
{
    pub fn fromEntity(entity: &Entity) -> Self
    {
        Self::new(entity.m_entityId.get(), entity.m_motionX.get(), entity.m_motionY.get(), entity.m_motionZ.get())
    }

    pub fn new(id: i32, mut velocityX: f64, mut velocityY: f64, mut velocityZ: f64) -> Self
    {
        let d: f64 = 3.9;
        if velocityX < -d
        {
            velocityX = -d;
        }
        if velocityY < -d
        {
            velocityY = -d;
        }
        if velocityZ < -d
        {
            velocityZ = -d;
        }
        if velocityX > d
        {
            velocityX = d;
        }
        if velocityY > d
        {
            velocityY = d;
        }
        if velocityZ > d
        {
            velocityZ = d;
        }
        Self {
            m_entityId: id,
            m_motionX: (velocityX * 8000.0) as i32,
            m_motionY: (velocityY * 8000.0) as i32,
            m_motionZ: (velocityZ * 8000.0) as i32,
            ..Self::default()
        }
    }
}

impl Packet for Packet28EntityVelocity
{
    fn base(&self) -> &PacketBase { &self.m_base }

    fn readPacketData(&mut self, input: &mut PacketInput) -> IoResult<()>
    {
        self.m_entityId = input.readInt()?;
        self.m_motionX = input.readShort()? as i32;
        self.m_motionY = input.readShort()? as i32;
        self.m_motionZ = input.readShort()? as i32;
        Ok(())
    }

    fn writePacketData(&self, output: &mut PacketOutput) -> IoResult<()>
    {
        output.writeInt(self.m_entityId)?;
        output.writeShort(self.m_motionX)?;
        output.writeShort(self.m_motionY)?;
        output.writeShort(self.m_motionZ)
    }

    fn processPacket(&self, handler: &dyn NetHandler) { handler.handleEntityVelocity(self); }

    fn getPacketSize(&self) -> i32 { 10 }
}
