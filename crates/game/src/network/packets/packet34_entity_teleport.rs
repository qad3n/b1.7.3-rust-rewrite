use crate::entity::Entity;
use crate::math::math_helper;
use crate::network::net_handler::NetHandler;
use crate::network::packet::{self, Packet, PacketBase, PacketInput, PacketOutput};
use jvm::io::IoResult;

#[derive(Clone, Default)]
pub struct Packet34EntityTeleport
{
    m_base: PacketBase,
    pub m_entityId: i32,
    pub m_xPosition: i32,
    pub m_yPosition: i32,
    pub m_zPosition: i32,
    pub m_yaw: i8,
    pub m_pitch: i8,
}

impl Packet34EntityTeleport
{
    pub fn withPosition(id: i32, x: i32, y: i32, z: i32, yaw: i8, pitch: i8) -> Self
    {
        Self {
            m_entityId: id,
            m_xPosition: x,
            m_yPosition: y,
            m_zPosition: z,
            m_yaw: yaw,
            m_pitch: pitch,
            ..Self::default()
        }
    }

    pub fn new(entity: &Entity) -> Self
    {
        Self {
            m_entityId: entity.m_entityId.get(),
            m_xPosition: math_helper::floor_double(entity.m_posX.get() * 32.0),
            m_yPosition: math_helper::floor_double(entity.m_posY.get() * 32.0),
            m_zPosition: math_helper::floor_double(entity.m_posZ.get() * 32.0),
            m_yaw: (entity.m_rotationYaw.get() * 256.0 / 360.0) as i32 as i8,
            m_pitch: (entity.m_rotationPitch.get() * 256.0 / 360.0) as i32 as i8,
            ..Self::default()
        }
    }
}

impl Packet for Packet34EntityTeleport
{
    fn base(&self) -> &PacketBase { &self.m_base }

    fn readPacketData(&mut self, input: &mut PacketInput) -> IoResult<()>
    {
        self.m_entityId = input.readInt()?;
        self.m_xPosition = input.readInt()?;
        self.m_yPosition = input.readInt()?;
        self.m_zPosition = input.readInt()?;
        self.m_yaw = packet::read(input)? as i8;
        self.m_pitch = packet::read(input)? as i8;
        Ok(())
    }

    fn writePacketData(&self, output: &mut PacketOutput) -> IoResult<()>
    {
        output.writeInt(self.m_entityId)?;
        output.writeInt(self.m_xPosition)?;
        output.writeInt(self.m_yPosition)?;
        output.writeInt(self.m_zPosition)?;
        output.write(self.m_yaw as i32)?;
        output.write(self.m_pitch as i32)
    }

    fn processPacket(&self, handler: &dyn NetHandler) { handler.handleEntityTeleport(self); }

    fn getPacketSize(&self) -> i32 { 34 }
}
