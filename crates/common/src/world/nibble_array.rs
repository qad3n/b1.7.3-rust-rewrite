use jvm::field::Field;

// a chunk column holds 256 cells per block of height, two to a byte
const COLUMNS: i32 = 256;

pub struct NibbleArray
{
    pub m_data: Field<Vec<i8>>,
    // rewrite: blocks per column, read from the array size so taller mod dimensions index the same way
    m_height: i32,
}

impl NibbleArray
{
    pub fn new(size: i32) -> Self
    {
        Self { m_data: Field::new(vec![0; (size >> 1) as usize]), m_height: size / COLUMNS }
    }

    pub fn withData(data: Vec<i8>) -> Self
    {
        let height: i32 = data.len() as i32 * 2 / COLUMNS;
        Self { m_data: Field::new(data), m_height: height }
    }

    fn index(&self, x: i32, y: i32, z: i32) -> i32 { (x << 4 | z) * self.m_height + y }

    pub fn getNibble(&self, x: i32, y: i32, z: i32) -> i32
    {
        let n: i32 = self.index(x, y, z);
        let n2: i32 = n >> 1;
        let n3: i32 = n & 1;
        let value: i32 = self.m_data.with(|data| data[n2 as usize] as i32);
        if n3 == 0
        {
            return value & 0xF;
        }
        value >> 4 & 0xF
    }

    pub fn setNibble(&self, x: i32, y: i32, z: i32, value: i32)
    {
        let i: i32 = self.index(x, y, z);
        let j: i32 = i >> 1;
        let k: i32 = i & 1;
        self.m_data.update(|data| {
            let current: i32 = data[j as usize] as i32;
            data[j as usize] = if k == 0
            {
                (current & 240 | value & 15) as i8
            }
            else
            {
                (current & 15 | (value & 15) << 4) as i8
            };
        });
    }

    pub fn isValid(&self) -> bool { true }
}
