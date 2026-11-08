use jvm::object::JavaObject;

#[derive(Clone, Copy, Debug)]
pub struct ChunkCoordIntPair
{
    pub m_chunkXPos: i32,
    pub m_chunkZPos: i32,
}

impl ChunkCoordIntPair
{
    pub fn new(x: i32, z: i32) -> Self { Self { m_chunkXPos: x, m_chunkZPos: z } }

    pub fn chunkXZ2Int(chunkX: i32, chunkZ: i32) -> i32
    {
        (if chunkX < 0 { i32::MIN } else { 0 })
            | (chunkX & i16::MAX as i32) << 16
            | (if chunkZ < 0 { 32768 } else { 0 })
            | chunkZ & i16::MAX as i32
    }

    pub fn hashCode(&self) -> i32 { Self::chunkXZ2Int(self.m_chunkXPos, self.m_chunkZPos) }

    pub fn equals(&self, other: &Self) -> bool
    {
        other.m_chunkXPos == self.m_chunkXPos && other.m_chunkZPos == self.m_chunkZPos
    }
}

impl JavaObject for ChunkCoordIntPair
{
    fn javaEquals(&self, other: &Self) -> bool { self.equals(other) }

    fn javaHash(&self) -> i32 { self.hashCode() }
}
