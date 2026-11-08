use crate::network::net_handler::NetHandler;
use crate::network::packet::{self, Packet, PacketBase, PacketInput, PacketOutput};
use jvm::io::IoResult;

#[derive(Clone, Default)]
pub struct Packet50PreChunk
{
    m_base: PacketBase,
    pub m_xPosition: i32,
    pub m_yPosition: i32,
    pub m_mode: bool,
}

impl Packet50PreChunk
{
    pub fn new(chunkX: i32, chunkZ: i32, load: bool) -> Self
    {
        Self { m_xPosition: chunkX, m_yPosition: chunkZ, m_mode: load, ..Self::default() }
    }
}

impl Packet for Packet50PreChunk
{
    fn base(&self) -> &PacketBase { &self.m_base }

    fn readPacketData(&mut self, input: &mut PacketInput) -> IoResult<()>
    {
        self.m_xPosition = input.readInt()?;
        self.m_yPosition = input.readInt()?;
        self.m_mode = packet::read(input)? != 0;
        Ok(())
    }

    fn writePacketData(&self, output: &mut PacketOutput) -> IoResult<()>
    {
        output.writeInt(self.m_xPosition)?;
        output.writeInt(self.m_yPosition)?;
        output.write(if self.m_mode { 1 } else { 0 })
    }

    fn processPacket(&self, handler: &dyn NetHandler) { handler.handlePreChunk(self); }

    fn getPacketSize(&self) -> i32 { 9 }
}
