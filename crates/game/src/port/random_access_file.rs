use jvm::io::{self, IoResult};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

pub struct RandomAccessFile
{
    m_file: std::fs::File,
}

impl RandomAccessFile
{
    pub fn open(path: &Path) -> IoResult<RandomAccessFile>
    {
        match std::fs::OpenOptions::new().read(true).write(true).create(true).truncate(false).open(path)
        {
            Ok(file) => Ok(RandomAccessFile { m_file: file }),
            Err(error) => Err(io::fileNotFound(path, &error)),
        }
    }

    pub fn length(&self) -> IoResult<i64>
    {
        self.m_file.metadata().map(|metadata| metadata.len() as i64).map_err(|error| io::fromStd(&error))
    }

    pub fn seek(&mut self, pos: i64) -> IoResult<()>
    {
        self.m_file.seek(SeekFrom::Start(pos as u64)).map(|_| ()).map_err(|error| io::fromStd(&error))
    }

    pub fn read(&mut self, buf: &mut [u8]) -> IoResult<i32>
    {
        if buf.is_empty()
        {
            return Ok(0);
        }
        match self.m_file.read(buf)
        {
            Ok(0) => Ok(-1),
            Ok(count) => Ok(count as i32),
            Err(error) => Err(io::fromStd(&error)),
        }
    }

    fn readFully(&mut self, buf: &mut [u8]) -> IoResult<()>
    {
        self.m_file.read_exact(buf).map_err(|error| match error.kind()
        {
            std::io::ErrorKind::UnexpectedEof => io::eof(),
            _ => io::fromStd(&error),
        })
    }

    pub fn readInt(&mut self) -> IoResult<i32>
    {
        let mut buffer: [u8; 4] = [0; 4];
        self.readFully(&mut buffer)?;
        Ok(i32::from_be_bytes(buffer))
    }

    pub fn readByte(&mut self) -> IoResult<i8>
    {
        let mut buffer: [u8; 1] = [0];
        self.readFully(&mut buffer)?;
        Ok(buffer[0] as i8)
    }

    pub fn writeFrom(&mut self, buf: &[u8], off: i32, len: i32) -> IoResult<()>
    {
        self.m_file.write_all(&buf[off as usize..(off + len) as usize]).map_err(|error| io::fromStd(&error))
    }

    pub fn write(&mut self, value: i32) -> IoResult<()> { self.writeFrom(&[value as u8], 0, 1) }

    pub fn writeByte(&mut self, value: i32) -> IoResult<()> { self.write(value) }

    pub fn writeInt(&mut self, value: i32) -> IoResult<()> { self.writeFrom(&value.to_be_bytes(), 0, 4) }

    pub fn close(&mut self) -> IoResult<()> { self.m_file.flush().map_err(|error| io::fromStd(&error)) }
}
