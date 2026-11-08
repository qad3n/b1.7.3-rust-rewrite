use crate::network::net_handler::NetHandler;
use crate::network::packet::{Packet, PacketBase, PacketInput, PacketOutput};
use jvm::io::IoResult;

#[derive(Clone, Default)]
pub struct Packet61DoorChange
{
    m_base: PacketBase,
    pub m_event: i32,
    pub m_data: i32,
    pub m_x: i32,
    pub m_y: i32,
    pub m_z: i32,
}

impl Packet61DoorChange
{
    pub fn new(n: i32, n2: i32, n3: i32, n4: i32, n5: i32) -> Self
    {
        Self { m_event: n, m_x: n2, m_y: n3, m_z: n4, m_data: n5, ..Self::default() }
    }
}

impl Packet for Packet61DoorChange
{
    fn base(&self) -> &PacketBase { &self.m_base }

    fn readPacketData(&mut self, input: &mut PacketInput) -> IoResult<()>
    {
        self.m_event = input.readInt()?;
        self.m_x = input.readInt()?;
        self.m_y = input.readByte()? as i32;
        self.m_z = input.readInt()?;
        self.m_data = input.readInt()?;
        Ok(())
    }

    fn writePacketData(&self, output: &mut PacketOutput) -> IoResult<()>
    {
        output.writeInt(self.m_event)?;
        output.writeInt(self.m_x)?;
        output.writeByte(self.m_y)?;
        output.writeInt(self.m_z)?;
        output.writeInt(self.m_data)
    }

    fn processPacket(&self, handler: &dyn NetHandler) { handler.handleWorldEvent(self); }

    fn getPacketSize(&self) -> i32 { 20 }
}
