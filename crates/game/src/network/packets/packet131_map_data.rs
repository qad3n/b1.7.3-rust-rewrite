use crate::network::net_handler::NetHandler;
use crate::network::packet::{Packet, PacketBase, PacketInput, PacketOutput};
use jvm::io::IoResult;
use jvm::throwable::OrNpe;

#[derive(Clone)]
pub struct Packet131MapData
{
    m_base: PacketBase,
    pub m_type: i16,
    pub m_id: i16,
    pub m_data: Option<Vec<u8>>,
}

impl Default for Packet131MapData
{
    fn default() -> Self { Self { m_base: PacketBase::chunkData(), m_type: 0, m_id: 0, m_data: None } }
}

impl Packet131MapData
{
    pub fn new(r#type: i16, id: i16, data: Vec<u8>) -> Self
    {
        Self { m_type: r#type, m_id: id, m_data: Some(data), ..Self::default() }
    }
}

impl Packet for Packet131MapData
{
    fn base(&self) -> &PacketBase { &self.m_base }

    fn readPacketData(&mut self, input: &mut PacketInput) -> IoResult<()>
    {
        self.m_type = input.readShort()?;
        self.m_id = input.readShort()?;
        self.m_data = Some(vec![0; (input.readByte()? as i32 & 0xFF) as usize]);
        input.readFully(self.m_data.as_mut().orNpe())
    }

    fn writePacketData(&self, output: &mut PacketOutput) -> IoResult<()>
    {
        let data: &Vec<u8> = self.m_data.as_ref().orNpe();
        output.writeShort(self.m_type as i32)?;
        output.writeShort(self.m_id as i32)?;
        output.writeByte(data.len() as i32)?;
        output.writeBuf(data)
    }

    fn processPacket(&self, handler: &dyn NetHandler) { handler.handleMapData(self); }

    fn getPacketSize(&self) -> i32 { 4 + self.m_data.as_ref().orNpe().len() as i32 }
}
