use crate::network::net_handler::NetHandler;
use crate::network::packet::{Packet, PacketBase, PacketInput, PacketOutput};
use common::world::ChunkPosition;
use jvm::hash_map::JHashSet;
use jvm::io::IoResult;
use jvm::throwable::OrNpe;
use std::cell::RefCell;

#[derive(Clone, Default)]
pub struct Packet60Explosion
{
    m_base: PacketBase,
    pub m_explosionX: f64,
    pub m_explosionY: f64,
    pub m_explosionZ: f64,
    pub m_explosionSize: f32,
    pub m_destroyedBlockPositions: RefCell<Option<JHashSet<ChunkPosition>>>,
}

impl Packet60Explosion
{
    pub fn new(x: f64, y: f64, z: f64, power: f32, damagedBlocks: &JHashSet<ChunkPosition>) -> Self
    {
        Self {
            m_explosionX: x,
            m_explosionY: y,
            m_explosionZ: z,
            m_explosionSize: power,
            m_destroyedBlockPositions: RefCell::new(Some(JHashSet::copyOf(damagedBlocks))),
            ..Self::default()
        }
    }
}

impl Packet for Packet60Explosion
{
    fn base(&self) -> &PacketBase { &self.m_base }

    fn readPacketData(&mut self, input: &mut PacketInput) -> IoResult<()>
    {
        self.m_explosionX = input.readDouble()?;
        self.m_explosionY = input.readDouble()?;
        self.m_explosionZ = input.readDouble()?;
        self.m_explosionSize = input.readFloat()?;
        let n: i32 = input.readInt()?;
        let mut positions: JHashSet<ChunkPosition> = JHashSet::new();
        let n2: i32 = self.m_explosionX as i32;
        let n3: i32 = self.m_explosionY as i32;
        let n4: i32 = self.m_explosionZ as i32;
        let mut i: i32 = 0;
        let read: IoResult<()> = (|| {
            while i < n
            {
                let n5: i32 = (input.readByte()? as i32).wrapping_add(n2);
                let n6: i32 = (input.readByte()? as i32).wrapping_add(n3);
                let n7: i32 = (input.readByte()? as i32).wrapping_add(n4);
                positions.add(ChunkPosition::new(n5, n6, n7));
                i += 1;
            }
            Ok(())
        })();
        *self.m_destroyedBlockPositions.get_mut() = Some(positions);
        read
    }

    fn writePacketData(&self, output: &mut PacketOutput) -> IoResult<()>
    {
        let guard: std::cell::Ref<'_, Option<JHashSet<ChunkPosition>>> = self.m_destroyedBlockPositions.borrow();
        let positions: &JHashSet<ChunkPosition> = guard.as_ref().orNpe();
        output.writeDouble(self.m_explosionX)?;
        output.writeDouble(self.m_explosionY)?;
        output.writeDouble(self.m_explosionZ)?;
        output.writeFloat(self.m_explosionSize)?;
        output.writeInt(positions.size())?;
        let n: i32 = self.m_explosionX as i32;
        let n2: i32 = self.m_explosionY as i32;
        let n3: i32 = self.m_explosionZ as i32;
        for chunkPosition in positions.items()
        {
            let n4: i32 = chunkPosition.m_x.wrapping_sub(n);
            let n5: i32 = chunkPosition.m_y.wrapping_sub(n2);
            let n6: i32 = chunkPosition.m_z.wrapping_sub(n3);
            output.writeByte(n4)?;
            output.writeByte(n5)?;
            output.writeByte(n6)?;
        }
        Ok(())
    }

    fn processPacket(&self, handler: &dyn NetHandler) { handler.handleExplosion(self); }

    fn getPacketSize(&self) -> i32 { 32 + self.m_destroyedBlockPositions.borrow().as_ref().orNpe().size() * 3 }
}
