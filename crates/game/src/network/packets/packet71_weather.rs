use crate::entity::Entity;
use crate::math::math_helper;
use crate::network::net_handler::NetHandler;
use crate::network::packet::{Packet, PacketBase, PacketInput, PacketOutput};
use jvm::io::IoResult;

#[derive(Clone, Default)]
pub struct Packet71Weather
{
    m_base: PacketBase,
    pub m_id: i32,
    pub m_x: i32,
    pub m_y: i32,
    pub m_z: i32,
    pub m_type: i32,
}

impl Packet71Weather
{
    pub fn new(entity: &Entity) -> Self
    {
        let mut packet: Self = Self {
            m_id: entity.m_entityId.get(),
            m_x: math_helper::floor_double(entity.m_posX.get() * 32.0),
            m_y: math_helper::floor_double(entity.m_posY.get() * 32.0),
            m_z: math_helper::floor_double(entity.m_posZ.get() * 32.0),
            ..Self::default()
        };
        if entity.instanceOf("EntityLightningBolt")
        {
            packet.m_type = 1;
        }
        packet
    }
}

impl Packet for Packet71Weather
{
    fn base(&self) -> &PacketBase { &self.m_base }

    fn readPacketData(&mut self, input: &mut PacketInput) -> IoResult<()>
    {
        self.m_id = input.readInt()?;
        self.m_type = input.readByte()? as i32;
        self.m_x = input.readInt()?;
        self.m_y = input.readInt()?;
        self.m_z = input.readInt()?;
        Ok(())
    }

    fn writePacketData(&self, output: &mut PacketOutput) -> IoResult<()>
    {
        output.writeInt(self.m_id)?;
        output.writeByte(self.m_type)?;
        output.writeInt(self.m_x)?;
        output.writeInt(self.m_y)?;
        output.writeInt(self.m_z)
    }

    fn processPacket(&self, handler: &dyn NetHandler) { handler.handleWeather(self); }

    fn getPacketSize(&self) -> i32 { 17 }
}
