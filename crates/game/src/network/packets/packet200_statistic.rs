use crate::network::net_handler::NetHandler;
use crate::network::packet::{Packet, PacketBase, PacketInput, PacketOutput};
use jvm::io::IoResult;

#[derive(Clone, Default)]
pub struct Packet200Statistic
{
    m_base: PacketBase,
    pub m_statId: i32,
    pub m_value: i32,
}

impl Packet200Statistic
{
    pub fn new(statId: i32, value: i32) -> Self { Self { m_statId: statId, m_value: value, ..Self::default() } }
}

impl Packet for Packet200Statistic
{
    fn base(&self) -> &PacketBase { &self.m_base }

    fn readPacketData(&mut self, input: &mut PacketInput) -> IoResult<()>
    {
        self.m_statId = input.readInt()?;
        self.m_value = input.readByte()? as i32;
        Ok(())
    }

    fn writePacketData(&self, output: &mut PacketOutput) -> IoResult<()>
    {
        output.writeInt(self.m_statId)?;
        output.writeByte(self.m_value)
    }

    fn processPacket(&self, handler: &dyn NetHandler) { handler.handleStat(self); }

    fn getPacketSize(&self) -> i32 { 6 }
}
