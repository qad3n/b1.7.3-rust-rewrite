use crate::network::net_handler::NetHandler;
use crate::network::packet::{Packet, PacketBase, PacketInput, PacketOutput};
use jvm::io::IoResult;

#[derive(Clone, Default)]
pub struct Packet27Position
{
    m_base: PacketBase,
    m_sidewaysSpeed: f32,
    m_forwardSpeed: f32,
    m_jumping: bool,
    m_sneaking: bool,
    m_pitch: f32,
    m_yaw: f32,
}

impl Packet for Packet27Position
{
    fn base(&self) -> &PacketBase { &self.m_base }

    fn readPacketData(&mut self, input: &mut PacketInput) -> IoResult<()>
    {
        self.m_sidewaysSpeed = input.readFloat()?;
        self.m_forwardSpeed = input.readFloat()?;
        self.m_pitch = input.readFloat()?;
        self.m_yaw = input.readFloat()?;
        self.m_jumping = input.readBoolean()?;
        self.m_sneaking = input.readBoolean()?;
        Ok(())
    }

    fn writePacketData(&self, output: &mut PacketOutput) -> IoResult<()>
    {
        output.writeFloat(self.m_sidewaysSpeed)?;
        output.writeFloat(self.m_forwardSpeed)?;
        output.writeFloat(self.m_pitch)?;
        output.writeFloat(self.m_yaw)?;
        output.writeBoolean(self.m_jumping)?;
        output.writeBoolean(self.m_sneaking)
    }

    fn processPacket(&self, handler: &dyn NetHandler) { handler.handlePlayerInput(self); }

    fn getPacketSize(&self) -> i32 { 18 }
}

impl Packet27Position
{
    pub fn getSidewaysSpeed(&self) -> f32 { self.m_sidewaysSpeed }

    pub fn getPitch(&self) -> f32 { self.m_pitch }

    pub fn getForwardSpeed(&self) -> f32 { self.m_forwardSpeed }

    pub fn getYaw(&self) -> f32 { self.m_yaw }

    pub fn getJumping(&self) -> bool { self.m_jumping }

    pub fn getSneaking(&self) -> bool { self.m_sneaking }
}
