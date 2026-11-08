use jvm::field::Field;

pub struct NibbleArray
{
    pub m_data: Field<Vec<i8>>,
}

impl NibbleArray
{
    pub fn new(size: i32) -> Self { Self { m_data: Field::new(vec![0; (size >> 1) as usize]) } }

    pub fn withData(data: Vec<i8>) -> Self { Self { m_data: Field::new(data) } }

    pub fn getNibble(&self, x: i32, y: i32, z: i32) -> i32
    {
        let n: i32 = x << 11 | z << 7 | y;
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
        let i: i32 = x << 11 | z << 7 | y;
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
