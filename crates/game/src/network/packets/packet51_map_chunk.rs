use crate::network::net_handler::NetHandler;
use crate::network::packet::{self, Packet, PacketBase, PacketInput, PacketOutput};
use crate::world::World;
use jvm::io::{IoResult, ioError};
use jvm::throwable::{IO, OrNpe};
use jvm::zip::{DATA_FORMAT, Deflater, Inflater};
use std::rc::Rc;

#[derive(Clone)]
pub struct Packet51MapChunk
{
    m_base: PacketBase,
    pub m_xPosition: i32,
    pub m_yPosition: i32,
    pub m_zPosition: i32,
    pub m_xSize: i32,
    pub m_ySize: i32,
    pub m_zSize: i32,
    pub m_chunk: Option<Vec<u8>>,
    m_chunkSize: i32,
}

impl Default for Packet51MapChunk
{
    fn default() -> Self
    {
        Self {
            m_base: PacketBase::chunkData(),
            m_xPosition: 0,
            m_yPosition: 0,
            m_zPosition: 0,
            m_xSize: 0,
            m_ySize: 0,
            m_zSize: 0,
            m_chunk: None,
            m_chunkSize: 0,
        }
    }
}

fn newByteArray(length: i32) -> Vec<u8>
{
    if length < 0
    {
        jvm::throwable::throwBare(&jvm::throwable::NEGATIVE_ARRAY_SIZE);
    }
    vec![0; length as usize]
}

impl Packet51MapChunk
{
    pub fn new(minX: i32, minY: i32, minZ: i32, widthX: i32, height: i32, widthZ: i32, world: &Rc<World>) -> Self
    {
        let mut packet: Self = Self {
            m_xPosition: minX,
            m_yPosition: minY,
            m_zPosition: minZ,
            m_xSize: widthX,
            m_ySize: height,
            m_zSize: widthZ,
            ..Self::default()
        };
        let byArray: Vec<u8> =
            world.getChunkData(minX, minY, minZ, widthX, height, widthZ).into_iter().map(|value| value as u8).collect();
        let mut deflater: Deflater = Deflater::new(-1);
        deflater.setInput(&byArray);
        deflater.finish();
        let mut chunk: Vec<u8> = newByteArray(widthX * height * widthZ * 5 / 2);
        packet.m_chunkSize = deflater.deflate(&mut chunk);
        packet.m_chunk = Some(chunk);
        deflater.end();
        packet
    }
}

impl Packet for Packet51MapChunk
{
    fn base(&self) -> &PacketBase { &self.m_base }

    fn readPacketData(&mut self, input: &mut PacketInput) -> IoResult<()>
    {
        self.m_xPosition = input.readInt()?;
        self.m_yPosition = input.readShort()? as i32;
        self.m_zPosition = input.readInt()?;
        self.m_xSize = packet::read(input)? + 1;
        self.m_ySize = packet::read(input)? + 1;
        self.m_zSize = packet::read(input)? + 1;
        self.m_chunkSize = input.readInt()?;
        let mut byArray: Vec<u8> = newByteArray(self.m_chunkSize);
        input.readFully(&mut byArray)?;
        let mut chunk: Vec<u8> =
            newByteArray(self.m_xSize.wrapping_mul(self.m_ySize).wrapping_mul(self.m_zSize).wrapping_mul(5) / 2);
        let mut inflater: Inflater = Inflater::new();
        inflater.setInput(&byArray);
        let inflated: IoResult<i32> = inflater.inflate(&mut chunk);
        inflater.end();
        self.m_chunk = Some(chunk);
        match inflated
        {
            Err(error) if error.instanceOf(&DATA_FORMAT) => Err(ioError(&IO, "Bad compressed data format")),
            Err(error) => Err(error),
            Ok(_) => Ok(()),
        }
    }

    fn writePacketData(&self, output: &mut PacketOutput) -> IoResult<()>
    {
        output.writeInt(self.m_xPosition)?;
        output.writeShort(self.m_yPosition)?;
        output.writeInt(self.m_zPosition)?;
        output.write(self.m_xSize - 1)?;
        output.write(self.m_ySize - 1)?;
        output.write(self.m_zSize - 1)?;
        output.writeInt(self.m_chunkSize)?;
        output.writeFrom(self.m_chunk.as_ref().orNpe(), 0, self.m_chunkSize)
    }

    fn processPacket(&self, handler: &dyn NetHandler) { handler.handleMapChunk(self); }

    fn getPacketSize(&self) -> i32 { 17 + self.m_chunkSize }
}
