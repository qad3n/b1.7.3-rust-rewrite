use jvm::object::JavaObject;

#[derive(Clone, Copy, Debug)]
pub struct ChunkPosition
{
    pub m_x: i32,
    pub m_y: i32,
    pub m_z: i32,
}

impl ChunkPosition
{
    pub fn new(x: i32, y: i32, z: i32) -> Self { Self { m_x: x, m_y: y, m_z: z } }

    pub fn equals(&self, other: &Self) -> bool
    {
        other.m_x == self.m_x && other.m_y == self.m_y && other.m_z == self.m_z
    }

    pub fn hashCode(&self) -> i32
    {
        self.m_x.wrapping_mul(8976890).wrapping_add(self.m_y.wrapping_mul(981131)).wrapping_add(self.m_z)
    }
}

impl JavaObject for ChunkPosition
{
    fn javaEquals(&self, other: &Self) -> bool { self.equals(other) }

    fn javaHash(&self) -> i32 { self.hashCode() }
}
