use jvm::object::JavaObject;

#[derive(Clone, Copy, Debug, Default)]
pub struct ChunkCoordinates
{
    pub m_x: i32,
    pub m_y: i32,
    pub m_z: i32,
}

impl ChunkCoordinates
{
    pub fn new(x: i32, y: i32, z: i32) -> Self { Self { m_x: x, m_y: y, m_z: z } }

    pub fn withCopy(pos: &ChunkCoordinates) -> Self { Self { m_x: pos.m_x, m_y: pos.m_y, m_z: pos.m_z } }

    pub fn equals(&self, other: &Self) -> bool
    {
        self.m_x == other.m_x && self.m_y == other.m_y && self.m_z == other.m_z
    }

    pub fn hashCode(&self) -> i32 { (self.m_x.wrapping_add(self.m_z) << (8i32.wrapping_add(self.m_y) & 31)) << 16 }

    pub fn compareTo(&self, other: &Self) -> i32
    {
        if self.m_y == other.m_y
        {
            if self.m_z == other.m_z
            {
                return self.m_x.wrapping_sub(other.m_x);
            }
            return self.m_z.wrapping_sub(other.m_z);
        }
        self.m_y.wrapping_sub(other.m_y)
    }

    pub fn getSqDistanceTo(&self, x: i32, y: i32, z: i32) -> f64
    {
        let n: i32 = self.m_x - x;
        let n2: i32 = self.m_y - y;
        let n3: i32 = self.m_z - z;
        (n.wrapping_mul(n).wrapping_add(n2.wrapping_mul(n2)).wrapping_add(n3.wrapping_mul(n3)) as f64).sqrt()
    }
}

impl JavaObject for ChunkCoordinates
{
    fn javaEquals(&self, other: &Self) -> bool { self.equals(other) }

    fn javaHash(&self) -> i32 { self.hashCode() }
}
