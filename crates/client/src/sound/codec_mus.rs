use jvm::io::{InputStream, IoResult};
use jvm::string::JString;

pub struct CodecMus;

impl CodecMus
{
    pub const CLASS_NAME: &'static str = "net.minecraft.src.CodecMus";

    pub fn openInputStream<S: InputStream>(url: &str, inputStream: S) -> MusInputStream<S>
    {
        MusInputStream::new(url, inputStream)
    }
}

pub struct MusInputStream<S: InputStream>
{
    m_hash: i32,
    m_inputStream: S,
    m_buffer: [u8; 1],
}

impl<S: InputStream> MusInputStream<S>
{
    pub fn new(fileUrl: &str, inputStream: S) -> MusInputStream<S>
    {
        let path: &str = fileUrl.split(['?', '#']).next().unwrap_or(fileUrl);
        let path: &str = path.strip_prefix("file:").unwrap_or(path);
        let name: &str = &path[path.rfind('/').map_or(0, |index| index + 1)..];
        MusInputStream { m_hash: JString::from(name).hashCode(), m_inputStream: inputStream, m_buffer: [0] }
    }
}

impl<S: InputStream> InputStream for MusInputStream<S>
{
    fn read(&mut self) -> IoResult<i32>
    {
        let mut buffer: [u8; 1] = self.m_buffer;
        let n: i32 = self.readInto(&mut buffer, 0, 1)?;
        self.m_buffer = buffer;
        if n < 0
        {
            return Ok(n);
        }
        Ok(self.m_buffer[0] as i8 as i32)
    }

    fn readInto(&mut self, buf: &mut [u8], off: i32, len: i32) -> IoResult<i32>
    {
        let n2: i32 = self.m_inputStream.readInto(buf, off, len)?;
        for i in 0..n2
        {
            let n3: usize = (off + i) as usize;
            let by: i8 = (buf[n3] as i8 as i32 ^ self.m_hash >> 8) as i8;
            buf[n3] = by as u8;
            self.m_hash = self.m_hash.wrapping_mul(498729871).wrapping_add(85731 * by as i32);
        }
        Ok(n2)
    }
}
