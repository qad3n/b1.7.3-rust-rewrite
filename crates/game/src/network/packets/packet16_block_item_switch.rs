use crate::network::net_handler::NetHandler;
use crate::network::packet::{Packet, PacketBase, PacketInput, PacketOutput};
use jvm::io::IoResult;

#[derive(Clone, Default)]
pub struct Packet16BlockItemSwitch
{
    m_base: PacketBase,
    pub m_id: i32,
}

impl Packet16BlockItemSwitch
{
    pub fn new(slot: i32) -> Self { Self { m_id: slot, ..Self::default() } }
}

impl Packet for Packet16BlockItemSwitch
{
    fn base(&self) -> &PacketBase { &self.m_base }

    fn readPacketData(&mut self, input: &mut PacketInput) -> IoResult<()>
    {
        self.m_id = input.readShort()? as i32;
        Ok(())
    }

    fn writePacketData(&self, output: &mut PacketOutput) -> IoResult<()> { output.writeShort(self.m_id) }

    fn processPacket(&self, handler: &dyn NetHandler) { handler.handleBlockItemSwitch(self); }

    fn getPacketSize(&self) -> i32 { 2 }
}
