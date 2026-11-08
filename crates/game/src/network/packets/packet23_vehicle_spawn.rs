use crate::entity::Entity;
use crate::math::math_helper;
use crate::network::net_handler::NetHandler;
use crate::network::packet::{Packet, PacketBase, PacketInput, PacketOutput};
use jvm::io::IoResult;

#[derive(Clone, Default)]
pub struct Packet23VehicleSpawn
{
    m_base: PacketBase,
    pub m_entityId: i32,
    pub m_xPosition: i32,
    pub m_yPosition: i32,
    pub m_zPosition: i32,
    pub m_velocityX: i32,
    pub m_velocityY: i32,
    pub m_velocityZ: i32,
    pub m_type: i32,
    pub m_data: i32,
}

impl Packet23VehicleSpawn
{
    pub fn new(entity: &Entity, r#type: i32) -> Self { Self::withData(entity, r#type, 0) }

    pub fn withData(entity: &Entity, r#type: i32, data: i32) -> Self
    {
        let mut packet: Self = Self {
            m_entityId: entity.m_entityId.get(),
            m_xPosition: math_helper::floor_double(entity.m_posX.get() * 32.0),
            m_yPosition: math_helper::floor_double(entity.m_posY.get() * 32.0),
            m_zPosition: math_helper::floor_double(entity.m_posZ.get() * 32.0),
            m_type: r#type,
            m_data: data,
            ..Self::default()
        };
        if data > 0
        {
            let mut d: f64 = entity.m_motionX.get();
            let mut d2: f64 = entity.m_motionY.get();
            let mut d3: f64 = entity.m_motionZ.get();
            let d4: f64 = 3.9;
            if d < -d4
            {
                d = -d4;
            }
            if d2 < -d4
            {
                d2 = -d4;
            }
            if d3 < -d4
            {
                d3 = -d4;
            }
            if d > d4
            {
                d = d4;
            }
            if d2 > d4
            {
                d2 = d4;
            }
            if d3 > d4
            {
                d3 = d4;
            }
            packet.m_velocityX = (d * 8000.0) as i32;
            packet.m_velocityY = (d2 * 8000.0) as i32;
            packet.m_velocityZ = (d3 * 8000.0) as i32;
        }
        packet
    }
}

impl Packet for Packet23VehicleSpawn
{
    fn base(&self) -> &PacketBase { &self.m_base }

    fn readPacketData(&mut self, input: &mut PacketInput) -> IoResult<()>
    {
        self.m_entityId = input.readInt()?;
        self.m_type = input.readByte()? as i32;
        self.m_xPosition = input.readInt()?;
        self.m_yPosition = input.readInt()?;
        self.m_zPosition = input.readInt()?;
        self.m_data = input.readInt()?;
        if self.m_data > 0
        {
            self.m_velocityX = input.readShort()? as i32;
            self.m_velocityY = input.readShort()? as i32;
            self.m_velocityZ = input.readShort()? as i32;
        }
        Ok(())
    }

    fn writePacketData(&self, output: &mut PacketOutput) -> IoResult<()>
    {
        output.writeInt(self.m_entityId)?;
        output.writeByte(self.m_type)?;
        output.writeInt(self.m_xPosition)?;
        output.writeInt(self.m_yPosition)?;
        output.writeInt(self.m_zPosition)?;
        output.writeInt(self.m_data)?;
        if self.m_data > 0
        {
            output.writeShort(self.m_velocityX)?;
            output.writeShort(self.m_velocityY)?;
            output.writeShort(self.m_velocityZ)?;
        }
        Ok(())
    }

    fn processPacket(&self, handler: &dyn NetHandler) { handler.handleVehicleSpawn(self); }

    fn getPacketSize(&self) -> i32
    {
        if 21i32.wrapping_add(self.m_data) > 0
        {
            6
        }
        else
        {
            0
        }
    }
}
