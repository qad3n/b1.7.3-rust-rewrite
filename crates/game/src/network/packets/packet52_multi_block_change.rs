use crate::network::net_handler::NetHandler;
use crate::network::packet::{Packet, PacketBase, PacketInput, PacketOutput};
use crate::world::{Chunk, World};
use jvm::io::IoResult;
use jvm::throwable::OrNpe;
use std::rc::Rc;

#[derive(Clone)]
pub struct Packet52MultiBlockChange
{
    m_base: PacketBase,
    pub m_xPosition: i32,
    pub m_zPosition: i32,
    pub m_coordinateArray: Option<Vec<i16>>,
    pub m_typeArray: Option<Vec<u8>>,
    pub m_metadataArray: Option<Vec<u8>>,
    pub m_size: i32,
}

impl Default for Packet52MultiBlockChange
{
    fn default() -> Self
    {
        Self {
            m_base: PacketBase::chunkData(),
            m_xPosition: 0,
            m_zPosition: 0,
            m_coordinateArray: None,
            m_typeArray: None,
            m_metadataArray: None,
            m_size: 0,
        }
    }
}

impl Packet52MultiBlockChange
{
    pub fn new(chunkX: i32, chunkZ: i32, positions: &[i16], blockChangeCount: i32, world: &Rc<World>) -> Self
    {
        let mut coordinateArray: Vec<i16> = vec![0; blockChangeCount as usize];
        let mut typeArray: Vec<u8> = vec![0; blockChangeCount as usize];
        let mut metadataArray: Vec<u8> = vec![0; blockChangeCount as usize];
        let chunk: Rc<Chunk> = world.getChunkFromChunkCoords(chunkX, chunkZ);
        for i in 0..blockChangeCount as usize
        {
            let n: i32 = positions[i] as i32 >> 12 & 0xF;
            let n2: i32 = positions[i] as i32 >> 8 & 0xF;
            let n3: i32 = positions[i] as i32 & 0xFF;
            coordinateArray[i] = positions[i];
            typeArray[i] = chunk.getBlockID(n, n3, n2) as u8;
            metadataArray[i] = chunk.getBlockMetadata(n, n3, n2) as u8;
        }
        Self {
            m_xPosition: chunkX,
            m_zPosition: chunkZ,
            m_size: blockChangeCount,
            m_coordinateArray: Some(coordinateArray),
            m_typeArray: Some(typeArray),
            m_metadataArray: Some(metadataArray),
            ..Self::default()
        }
    }
}

impl Packet for Packet52MultiBlockChange
{
    fn base(&self) -> &PacketBase { &self.m_base }

    fn readPacketData(&mut self, input: &mut PacketInput) -> IoResult<()>
    {
        self.m_xPosition = input.readInt()?;
        self.m_zPosition = input.readInt()?;
        self.m_size = input.readShort()? as i32 & 0xFFFF;
        let size: usize = self.m_size as usize;
        let mut coordinateArray: Vec<i16> = vec![0; size];
        self.m_typeArray = Some(vec![0; size]);
        self.m_metadataArray = Some(vec![0; size]);
        for coordinate in &mut coordinateArray
        {
            *coordinate = input.readShort()?;
        }
        self.m_coordinateArray = Some(coordinateArray);
        input.readFully(self.m_typeArray.as_mut().orNpe())?;
        input.readFully(self.m_metadataArray.as_mut().orNpe())
    }

    fn writePacketData(&self, output: &mut PacketOutput) -> IoResult<()>
    {
        output.writeInt(self.m_xPosition)?;
        output.writeInt(self.m_zPosition)?;
        output.writeShort(self.m_size as i16 as i32)?;
        let coordinateArray: &Vec<i16> = self.m_coordinateArray.as_ref().orNpe();
        for i in 0..self.m_size
        {
            output.writeShort(coordinateArray[i as usize] as i32)?;
        }
        output.writeBuf(self.m_typeArray.as_ref().orNpe())?;
        output.writeBuf(self.m_metadataArray.as_ref().orNpe())
    }

    fn processPacket(&self, handler: &dyn NetHandler) { handler.handleMultiBlockChange(self); }

    fn getPacketSize(&self) -> i32 { 10 + self.m_size * 4 }
}
