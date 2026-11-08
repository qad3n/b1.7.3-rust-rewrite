use crate::network::net_handler::NetHandler;
use crate::network::packet::{self, Packet, PacketBase, PacketInput, PacketOutput};
// string length of a field java dereferences without a null check comes from port
use crate::port::network;
use jvm::io::IoResult;
use jvm::string::JString;
use jvm::throwable::OrNpe;

#[derive(Clone)]
pub struct Packet130UpdateSign
{
    m_base: PacketBase,
    pub m_xPosition: i32,
    pub m_yPosition: i32,
    pub m_zPosition: i32,
    pub m_signLines: Option<Vec<Option<JString>>>,
}

impl Default for Packet130UpdateSign
{
    fn default() -> Self
    {
        Self { m_base: PacketBase::chunkData(), m_xPosition: 0, m_yPosition: 0, m_zPosition: 0, m_signLines: None }
    }
}

impl Packet130UpdateSign
{
    pub fn new(x: i32, y: i32, z: i32, lines: Vec<Option<JString>>) -> Self
    {
        Self { m_xPosition: x, m_yPosition: y, m_zPosition: z, m_signLines: Some(lines), ..Self::default() }
    }
}

impl Packet for Packet130UpdateSign
{
    fn base(&self) -> &PacketBase { &self.m_base }

    fn readPacketData(&mut self, input: &mut PacketInput) -> IoResult<()>
    {
        self.m_xPosition = input.readInt()?;
        self.m_yPosition = input.readShort()? as i32;
        self.m_zPosition = input.readInt()?;
        self.m_signLines = Some(vec![None; 4]);
        for i in 0..4
        {
            let line: JString = packet::readString(input, 15)?;
            self.m_signLines.as_mut().orNpe()[i] = Some(line);
        }
        Ok(())
    }

    fn writePacketData(&self, output: &mut PacketOutput) -> IoResult<()>
    {
        output.writeInt(self.m_xPosition)?;
        output.writeShort(self.m_yPosition)?;
        output.writeInt(self.m_zPosition)?;
        let signLines: &Vec<Option<JString>> = self.m_signLines.as_ref().orNpe();
        for line in &signLines[..4]
        {
            packet::writeString(line.as_ref().orNpe(), output)?;
        }
        Ok(())
    }

    fn processPacket(&self, handler: &dyn NetHandler) { handler.handleSignUpdate(self); }

    fn getPacketSize(&self) -> i32
    {
        let signLines: &Vec<Option<JString>> = self.m_signLines.as_ref().orNpe();
        let mut n: i32 = 0;
        for line in &signLines[..4]
        {
            n += network::length(line);
        }
        n
    }
}
