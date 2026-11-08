use crate::network::net_handler::NetHandler;
use crate::network::packet::{self, Packet, PacketBase, PacketInput, PacketOutput};
use jvm::io::IoResult;

#[derive(Clone, Default)]
pub struct Packet14BlockDig
{
    m_base: PacketBase,
    pub m_xPosition: i32,
    pub m_yPosition: i32,
    pub m_zPosition: i32,
    pub m_face: i32,
    pub m_status: i32,
}

impl Packet14BlockDig
{
    pub fn new(action: i32, x: i32, y: i32, z: i32, face: i32) -> Self
    {
        Self { m_status: action, m_xPosition: x, m_yPosition: y, m_zPosition: z, m_face: face, ..Self::default() }
    }
}

impl Packet for Packet14BlockDig
{
    fn base(&self) -> &PacketBase { &self.m_base }

    fn readPacketData(&mut self, input: &mut PacketInput) -> IoResult<()>
    {
        self.m_status = packet::read(input)?;
        self.m_xPosition = input.readInt()?;
        self.m_yPosition = packet::read(input)?;
        self.m_zPosition = input.readInt()?;
        self.m_face = packet::read(input)?;
        Ok(())
    }

    fn writePacketData(&self, output: &mut PacketOutput) -> IoResult<()>
    {
        output.write(self.m_status)?;
        output.writeInt(self.m_xPosition)?;
        output.write(self.m_yPosition)?;
        output.writeInt(self.m_zPosition)?;
        output.write(self.m_face)
    }

    fn processPacket(&self, handler: &dyn NetHandler) { handler.handleBlockDig(self); }

    fn getPacketSize(&self) -> i32 { 11 }
}
