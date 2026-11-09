use crate::network::net_handler::NetHandler;
use crate::network::packet::{Packet, PacketBase, PacketInput, PacketOutput};
use jvm::io::{IoResult, ioError};
use jvm::throwable::IO;

// mod loader traffic, only ever sent between two modded peers, protocol 14 has no custom payload packet
#[derive(Clone, Default)]
pub struct Packet250ModPayload
{
    m_base: PacketBase,
    pub m_channel: u32,
    pub m_data: Vec<u8>,
}

const MAX_BYTES: u32 = 1 << 20;

fn readVarint(input: &mut PacketInput) -> IoResult<u32>
{
    let mut value: u32 = 0;
    for shift in 0..5
    {
        let byte: u8 = input.readByte()? as u8;
        value |= ((byte & 0x7F) as u32) << (shift * 7);
        if byte & 0x80 == 0
        {
            return Ok(value);
        }
    }
    Err(ioError(&IO, "varint longer than 5 bytes"))
}

fn writeVarint(output: &mut PacketOutput, mut value: u32) -> IoResult<()>
{
    loop
    {
        let byte: u8 = (value & 0x7F) as u8;
        value >>= 7;
        if value == 0
        {
            return output.writeByte(byte as i32);
        }
        output.writeByte((byte | 0x80) as i32)?;
    }
}

impl Packet250ModPayload
{
    pub fn new(channel: u32, data: Vec<u8>) -> Self { Self { m_channel: channel, m_data: data, ..Self::default() } }

    // a payload completing a chunk packet waits in the chunk queue behind it
    pub fn withChunkData(channel: u32, data: Vec<u8>) -> Self
    {
        Self { m_base: PacketBase::chunkData(), m_channel: channel, m_data: data }
    }
}

impl Packet for Packet250ModPayload
{
    fn base(&self) -> &PacketBase { &self.m_base }

    fn readPacketData(&mut self, input: &mut PacketInput) -> IoResult<()>
    {
        self.m_channel = readVarint(input)?;
        let length: u32 = readVarint(input)?;
        if length > MAX_BYTES
        {
            return Err(ioError(&IO, &format!("mod payload of {length} bytes is too large")));
        }
        let mut data: Vec<u8> = vec![0; length as usize];
        input.readFully(&mut data)?;
        self.m_data = data;
        Ok(())
    }

    fn writePacketData(&self, output: &mut PacketOutput) -> IoResult<()>
    {
        writeVarint(output, self.m_channel)?;
        writeVarint(output, self.m_data.len() as u32)?;
        output.writeBuf(&self.m_data)
    }

    fn processPacket(&self, handler: &dyn NetHandler) { handler.handleModPayload(self); }

    fn getPacketSize(&self) -> i32 { 10 + self.m_data.len() as i32 }
}
