use crate::network::net_handler::NetHandler;
use crate::network::packet::{Packet, PacketBase, PacketInput, PacketOutput};
use jvm::io::IoResult;

#[derive(Clone, Default)]
pub struct Packet6SpawnPosition
{
    m_base: PacketBase,
    pub m_xPosition: i32,
    pub m_yPosition: i32,
    pub m_zPosition: i32,
}

impl Packet6SpawnPosition
{
    pub fn new(x: i32, y: i32, z: i32) -> Self
    {
        Self { m_xPosition: x, m_yPosition: y, m_zPosition: z, ..Self::default() }
    }
}

impl Packet for Packet6SpawnPosition
{
    fn base(&self) -> &PacketBase { &self.m_base }

    fn readPacketData(&mut self, input: &mut PacketInput) -> IoResult<()>
    {
        self.m_xPosition = input.readInt()?;
        self.m_yPosition = input.readInt()?;
        self.m_zPosition = input.readInt()?;
        Ok(())
    }

    fn writePacketData(&self, output: &mut PacketOutput) -> IoResult<()>
    {
        output.writeInt(self.m_xPosition)?;
        output.writeInt(self.m_yPosition)?;
        output.writeInt(self.m_zPosition)
    }

    fn processPacket(&self, handler: &dyn NetHandler) { handler.handleSpawnPosition(self); }

    fn getPacketSize(&self) -> i32 { 12 }
}
