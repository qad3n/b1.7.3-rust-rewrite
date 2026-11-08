use crate::entity::Entity;
use crate::network::net_handler::NetHandler;
use crate::network::packet::{Packet, PacketBase, PacketInput, PacketOutput};
use jvm::io::IoResult;

#[derive(Clone, Default)]
pub struct Packet17Sleep
{
    m_base: PacketBase,
    pub m_id: i32,
    pub m_x: i32,
    pub m_y: i32,
    pub m_z: i32,
    pub m_action: i32,
}

impl Packet17Sleep
{
    pub fn new(player: &Entity, action: i32, x: i32, y: i32, z: i32) -> Self
    {
        Self { m_action: action, m_x: x, m_y: y, m_z: z, m_id: player.m_entityId.get(), ..Self::default() }
    }
}

impl Packet for Packet17Sleep
{
    fn base(&self) -> &PacketBase { &self.m_base }

    fn readPacketData(&mut self, input: &mut PacketInput) -> IoResult<()>
    {
        self.m_id = input.readInt()?;
        self.m_action = input.readByte()? as i32;
        self.m_x = input.readInt()?;
        self.m_y = input.readByte()? as i32;
        self.m_z = input.readInt()?;
        Ok(())
    }

    fn writePacketData(&self, output: &mut PacketOutput) -> IoResult<()>
    {
        output.writeInt(self.m_id)?;
        output.writeByte(self.m_action)?;
        output.writeInt(self.m_x)?;
        output.writeByte(self.m_y)?;
        output.writeInt(self.m_z)
    }

    fn processPacket(&self, handler: &dyn NetHandler) { handler.handlePlayerSleep(self); }

    fn getPacketSize(&self) -> i32 { 14 }
}
