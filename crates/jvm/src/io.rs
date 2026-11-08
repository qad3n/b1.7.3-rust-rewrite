use crate::string::JString;
use crate::throwable::{self, Throwable, ThrowableClass};

pub type IoResult<T> = Result<T, Throwable>;

pub static FILE_NOT_FOUND: ThrowableClass =
    ThrowableClass { m_name: "java.io.FileNotFoundException", m_parent: Some(&throwable::IO) };
pub static UTF_DATA_FORMAT: ThrowableClass =
    ThrowableClass { m_name: "java.io.UTFDataFormatException", m_parent: Some(&throwable::IO) };

pub fn ioError(class: &'static ThrowableClass, message: &str) -> Throwable
{
    Throwable::new(class, Some(message.to_owned()))
}

pub fn eof() -> Throwable { Throwable::new(&throwable::EOF, None) }

pub fn fromStd(error: &std::io::Error) -> Throwable
{
    match error.kind()
    {
        std::io::ErrorKind::UnexpectedEof => eof(),
        _ => ioError(&throwable::IO, &error.to_string()),
    }
}

fn checkBounds(length: usize, off: i32, len: i32)
{
    if off < 0 || len < 0 || len as i64 > length as i64 - off as i64
    {
        throwable::throwBare(&throwable::INDEX_OUT_OF_BOUNDS);
    }
}

pub trait InputStream
{
    fn read(&mut self) -> IoResult<i32>;

    // InputStream.read(byte[], int, int) of jdk 6 reads byte by byte and stops early on an IOException after the first
    fn readInto(&mut self, buf: &mut [u8], off: i32, len: i32) -> IoResult<i32>
    {
        checkBounds(buf.len(), off, len);
        if len == 0
        {
            return Ok(0);
        }

        let first: i32 = self.read()?;
        if first == -1
        {
            return Ok(-1);
        }
        buf[off as usize] = first as u8;

        let mut i: i32 = 1;
        while i < len
        {
            let value: i32 = match self.read()
            {
                Ok(value) => value,
                Err(_) => break,
            };
            if value == -1
            {
                break;
            }
            buf[(off + i) as usize] = value as u8;
            i += 1;
        }
        Ok(i)
    }

    fn skip(&mut self, n: i64) -> IoResult<i64>
    {
        const SKIP_BUFFER_SIZE: i64 = 2048;
        let mut remaining: i64 = n;
        if n <= 0
        {
            return Ok(0);
        }

        let size: i64 = SKIP_BUFFER_SIZE.min(remaining);
        let mut buffer: Vec<u8> = vec![0; size as usize];
        while remaining > 0
        {
            let count: i32 = self.readInto(&mut buffer, 0, size.min(remaining) as i32)?;
            if count < 0
            {
                break;
            }
            remaining -= count as i64;
        }
        Ok(n - remaining)
    }

    fn available(&mut self) -> IoResult<i32> { Ok(0) }

    fn close(&mut self) -> IoResult<()> { Ok(()) }
}

pub trait OutputStream
{
    fn write(&mut self, value: i32) -> IoResult<()>;

    fn writeFrom(&mut self, buf: &[u8], off: i32, len: i32) -> IoResult<()>
    {
        checkBounds(buf.len(), off, len);
        for i in 0..len
        {
            self.write(buf[(off + i) as usize] as i32)?;
        }
        Ok(())
    }

    fn flush(&mut self) -> IoResult<()> { Ok(()) }

    fn close(&mut self) -> IoResult<()> { self.flush() }
}

impl<T: InputStream + ?Sized> InputStream for Box<T>
{
    fn read(&mut self) -> IoResult<i32> { (**self).read() }

    fn readInto(&mut self, buf: &mut [u8], off: i32, len: i32) -> IoResult<i32> { (**self).readInto(buf, off, len) }

    fn skip(&mut self, n: i64) -> IoResult<i64> { (**self).skip(n) }

    fn available(&mut self) -> IoResult<i32> { (**self).available() }

    fn close(&mut self) -> IoResult<()> { (**self).close() }
}

impl<T: OutputStream + ?Sized> OutputStream for Box<T>
{
    fn write(&mut self, value: i32) -> IoResult<()> { (**self).write(value) }

    fn writeFrom(&mut self, buf: &[u8], off: i32, len: i32) -> IoResult<()> { (**self).writeFrom(buf, off, len) }

    fn flush(&mut self) -> IoResult<()> { (**self).flush() }

    fn close(&mut self) -> IoResult<()> { (**self).close() }
}

pub struct ByteArrayInputStream
{
    m_buf: Vec<u8>,
    m_pos: i32,
    m_mark: i32,
    m_count: i32,
}

impl ByteArrayInputStream
{
    pub fn new(buf: Vec<u8>) -> Self
    {
        let count: i32 = buf.len() as i32;
        Self { m_buf: buf, m_pos: 0, m_mark: 0, m_count: count }
    }

    pub fn reset(&mut self) { self.m_pos = self.m_mark; }
}

impl InputStream for ByteArrayInputStream
{
    fn read(&mut self) -> IoResult<i32>
    {
        if self.m_pos < self.m_count
        {
            let value: i32 = self.m_buf[self.m_pos as usize] as i32;
            self.m_pos += 1;
            return Ok(value);
        }
        Ok(-1)
    }

    fn readInto(&mut self, buf: &mut [u8], off: i32, len: i32) -> IoResult<i32>
    {
        checkBounds(buf.len(), off, len);
        if self.m_pos >= self.m_count
        {
            return Ok(-1);
        }

        let mut len: i32 = len;
        if self.m_pos + len > self.m_count
        {
            len = self.m_count - self.m_pos;
        }
        if len <= 0
        {
            return Ok(0);
        }

        buf[off as usize..(off + len) as usize]
            .copy_from_slice(&self.m_buf[self.m_pos as usize..(self.m_pos + len) as usize]);
        self.m_pos += len;
        Ok(len)
    }

    fn skip(&mut self, n: i64) -> IoResult<i64>
    {
        let mut n: i64 = n;
        if self.m_pos as i64 + n > self.m_count as i64
        {
            n = (self.m_count - self.m_pos) as i64;
        }
        if n < 0
        {
            return Ok(0);
        }
        self.m_pos += n as i32;
        Ok(n)
    }

    fn available(&mut self) -> IoResult<i32> { Ok(self.m_count - self.m_pos) }
}

#[derive(Default)]
pub struct ByteArrayOutputStream
{
    m_buf: Vec<u8>,
}

impl ByteArrayOutputStream
{
    pub fn new() -> Self { Self { m_buf: Vec::new() } }

    pub fn toByteArray(&self) -> Vec<u8> { self.m_buf.clone() }

    pub fn size(&self) -> i32 { self.m_buf.len() as i32 }

    pub fn reset(&mut self) { self.m_buf.clear(); }
}

impl OutputStream for ByteArrayOutputStream
{
    fn write(&mut self, value: i32) -> IoResult<()>
    {
        self.m_buf.push(value as u8);
        Ok(())
    }

    fn writeFrom(&mut self, buf: &[u8], off: i32, len: i32) -> IoResult<()>
    {
        checkBounds(buf.len(), off, len);
        self.m_buf.extend_from_slice(&buf[off as usize..(off + len) as usize]);
        Ok(())
    }
}

pub struct DataInputStream<S: InputStream>
{
    pub m_in: S,
}

impl<S: InputStream> DataInputStream<S>
{
    pub fn new(input: S) -> Self { Self { m_in: input } }

    pub fn read(&mut self, buf: &mut [u8]) -> IoResult<i32>
    {
        let len: i32 = buf.len() as i32;
        self.m_in.readInto(buf, 0, len)
    }

    pub fn readFully(&mut self, buf: &mut [u8]) -> IoResult<()>
    {
        let len: i32 = buf.len() as i32;
        self.readFullyAt(buf, 0, len)
    }

    pub fn readFullyAt(&mut self, buf: &mut [u8], off: i32, len: i32) -> IoResult<()>
    {
        if len < 0
        {
            throwable::throwBare(&throwable::INDEX_OUT_OF_BOUNDS);
        }

        let mut n: i32 = 0;
        while n < len
        {
            let count: i32 = self.m_in.readInto(buf, off + n, len - n)?;
            if count < 0
            {
                return Err(eof());
            }
            n += count;
        }
        Ok(())
    }

    pub fn skipBytes(&mut self, n: i32) -> IoResult<i32>
    {
        let mut total: i32 = 0;
        while total < n
        {
            let cur: i32 = self.m_in.skip((n - total) as i64)? as i32;
            if cur <= 0
            {
                break;
            }
            total += cur;
        }
        Ok(total)
    }

    fn readByteValue(&mut self) -> IoResult<i32>
    {
        let ch: i32 = self.m_in.read()?;
        if ch < 0
        {
            return Err(eof());
        }
        Ok(ch)
    }

    pub fn readBoolean(&mut self) -> IoResult<bool> { Ok(self.readByteValue()? != 0) }

    pub fn readByte(&mut self) -> IoResult<i8> { Ok(self.readByteValue()? as i8) }

    pub fn readUnsignedByte(&mut self) -> IoResult<i32> { self.readByteValue() }

    pub fn readShort(&mut self) -> IoResult<i16>
    {
        let ch1: i32 = self.m_in.read()?;
        let ch2: i32 = self.m_in.read()?;
        if (ch1 | ch2) < 0
        {
            return Err(eof());
        }
        Ok(((ch1 << 8) + ch2) as i16)
    }

    pub fn readUnsignedShort(&mut self) -> IoResult<i32>
    {
        let ch1: i32 = self.m_in.read()?;
        let ch2: i32 = self.m_in.read()?;
        if (ch1 | ch2) < 0
        {
            return Err(eof());
        }
        Ok((ch1 << 8) + ch2)
    }

    pub fn readChar(&mut self) -> IoResult<u16> { Ok(self.readUnsignedShort()? as u16) }

    pub fn readInt(&mut self) -> IoResult<i32>
    {
        let ch1: i32 = self.m_in.read()?;
        let ch2: i32 = self.m_in.read()?;
        let ch3: i32 = self.m_in.read()?;
        let ch4: i32 = self.m_in.read()?;
        if (ch1 | ch2 | ch3 | ch4) < 0
        {
            return Err(eof());
        }
        Ok((ch1 << 24).wrapping_add(ch2 << 16).wrapping_add(ch3 << 8).wrapping_add(ch4))
    }

    pub fn readLong(&mut self) -> IoResult<i64>
    {
        let mut buffer: [u8; 8] = [0; 8];
        self.readFully(&mut buffer)?;
        Ok(i64::from_be_bytes(buffer))
    }

    pub fn readFloat(&mut self) -> IoResult<f32> { Ok(f32::from_bits(self.readInt()? as u32)) }

    pub fn readDouble(&mut self) -> IoResult<f64> { Ok(f64::from_bits(self.readLong()? as u64)) }

    pub fn readUTF(&mut self) -> IoResult<JString>
    {
        let utflen: i32 = self.readUnsignedShort()?;
        let mut bytes: Vec<u8> = vec![0; utflen as usize];
        self.readFully(&mut bytes)?;
        decodeModifiedUtf8(&bytes)
    }
}

pub fn decodeModifiedUtf8(bytes: &[u8]) -> IoResult<JString>
{
    let utflen: i32 = bytes.len() as i32;
    let mut chars: Vec<u16> = Vec::with_capacity(bytes.len());
    let mut count: i32 = 0;
    while count < utflen
    {
        let c: i32 = bytes[count as usize] as i32;
        match c >> 4
        {
            0..=7 =>
            {
                count += 1;
                chars.push(c as u16);
            }
            12 | 13 =>
            {
                count += 2;
                if count > utflen
                {
                    return Err(ioError(&UTF_DATA_FORMAT, "malformed input: partial character at end"));
                }
                let char2: i32 = bytes[(count - 1) as usize] as i8 as i32;
                if (char2 & 0xC0) != 0x80
                {
                    return Err(ioError(&UTF_DATA_FORMAT, &format!("malformed input around byte {count}")));
                }
                chars.push((((c & 0x1F) << 6) | (char2 & 0x3F)) as u16);
            }
            14 =>
            {
                count += 3;
                if count > utflen
                {
                    return Err(ioError(&UTF_DATA_FORMAT, "malformed input: partial character at end"));
                }
                let char2: i32 = bytes[(count - 2) as usize] as i8 as i32;
                let char3: i32 = bytes[(count - 1) as usize] as i8 as i32;
                if (char2 & 0xC0) != 0x80 || (char3 & 0xC0) != 0x80
                {
                    return Err(ioError(&UTF_DATA_FORMAT, &format!("malformed input around byte {}", count - 1)));
                }
                chars.push((((c & 0x0F) << 12) | ((char2 & 0x3F) << 6) | (char3 & 0x3F)) as u16);
            }
            _ => return Err(ioError(&UTF_DATA_FORMAT, &format!("malformed input around byte {count}"))),
        }
    }
    Ok(JString::fromUtf16(chars))
}

pub fn encodeModifiedUtf8(text: &JString) -> IoResult<Vec<u8>>
{
    let mut utflen: i32 = 0;
    for &c in text.chars()
    {
        if (0x0001..=0x007F).contains(&c)
        {
            utflen += 1;
        }
        else if c > 0x07FF
        {
            utflen += 3;
        }
        else
        {
            utflen += 2;
        }
    }

    if utflen > 65535
    {
        return Err(ioError(&UTF_DATA_FORMAT, &format!("encoded string too long: {utflen} bytes")));
    }

    let mut bytes: Vec<u8> = Vec::with_capacity(utflen as usize + 2);
    bytes.push(((utflen >> 8) & 0xFF) as u8);
    bytes.push((utflen & 0xFF) as u8);
    for &c in text.chars()
    {
        let c: i32 = c as i32;
        if (0x0001..=0x007F).contains(&c)
        {
            bytes.push(c as u8);
        }
        else if c > 0x07FF
        {
            bytes.push((0xE0 | ((c >> 12) & 0x0F)) as u8);
            bytes.push((0x80 | ((c >> 6) & 0x3F)) as u8);
            bytes.push((0x80 | (c & 0x3F)) as u8);
        }
        else
        {
            bytes.push((0xC0 | ((c >> 6) & 0x1F)) as u8);
            bytes.push((0x80 | (c & 0x3F)) as u8);
        }
    }
    Ok(bytes)
}

pub struct DataOutputStream<S: OutputStream>
{
    pub m_out: S,
    m_written: i32,
}

impl<S: OutputStream> DataOutputStream<S>
{
    pub fn new(output: S) -> Self { Self { m_out: output, m_written: 0 } }

    fn incCount(&mut self, value: i32)
    {
        let mut temp: i32 = self.m_written.wrapping_add(value);
        if temp < 0
        {
            temp = i32::MAX;
        }
        self.m_written = temp;
    }

    pub fn size(&self) -> i32 { self.m_written }

    pub fn write(&mut self, value: i32) -> IoResult<()>
    {
        self.m_out.write(value)?;
        self.incCount(1);
        Ok(())
    }

    pub fn writeBuf(&mut self, buf: &[u8]) -> IoResult<()>
    {
        let len: i32 = buf.len() as i32;
        self.writeFrom(buf, 0, len)
    }

    pub fn writeFrom(&mut self, buf: &[u8], off: i32, len: i32) -> IoResult<()>
    {
        self.m_out.writeFrom(buf, off, len)?;
        self.incCount(len);
        Ok(())
    }

    pub fn flush(&mut self) -> IoResult<()> { self.m_out.flush() }

    pub fn close(&mut self) -> IoResult<()> { self.m_out.close() }

    pub fn writeBoolean(&mut self, value: bool) -> IoResult<()> { self.write(if value { 1 } else { 0 }) }

    pub fn writeByte(&mut self, value: i32) -> IoResult<()> { self.write(value) }

    pub fn writeShort(&mut self, value: i32) -> IoResult<()>
    {
        self.m_out.write(((value as u32) >> 8) as i32 & 0xFF)?;
        self.m_out.write(value & 0xFF)?;
        self.incCount(2);
        Ok(())
    }

    pub fn writeChar(&mut self, value: i32) -> IoResult<()> { self.writeShort(value) }

    pub fn writeInt(&mut self, value: i32) -> IoResult<()>
    {
        self.m_out.write(((value as u32) >> 24) as i32 & 0xFF)?;
        self.m_out.write(((value as u32) >> 16) as i32 & 0xFF)?;
        self.m_out.write(((value as u32) >> 8) as i32 & 0xFF)?;
        self.m_out.write(value & 0xFF)?;
        self.incCount(4);
        Ok(())
    }

    pub fn writeLong(&mut self, value: i64) -> IoResult<()>
    {
        let buffer: [u8; 8] = value.to_be_bytes();
        self.m_out.writeFrom(&buffer, 0, 8)?;
        self.incCount(8);
        Ok(())
    }

    pub fn writeFloat(&mut self, value: f32) -> IoResult<()> { self.writeInt(floatToIntBits(value)) }

    pub fn writeDouble(&mut self, value: f64) -> IoResult<()> { self.writeLong(doubleToLongBits(value)) }

    pub fn writeBytes(&mut self, text: &JString) -> IoResult<()>
    {
        for &c in text.chars()
        {
            self.m_out.write(c as u8 as i32)?;
        }
        self.incCount(text.length());
        Ok(())
    }

    pub fn writeChars(&mut self, text: &JString) -> IoResult<()>
    {
        for &c in text.chars()
        {
            let v: i32 = c as i32;
            self.m_out.write((v >> 8) & 0xFF)?;
            self.m_out.write(v & 0xFF)?;
        }
        self.incCount(text.length() * 2);
        Ok(())
    }

    pub fn writeUTF(&mut self, text: &JString) -> IoResult<()>
    {
        let bytes: Vec<u8> = encodeModifiedUtf8(text)?;
        let len: i32 = bytes.len() as i32;
        self.m_out.writeFrom(&bytes, 0, len)?;
        self.incCount(len);
        Ok(())
    }
}

// Float.floatToIntBits folds every NaN into the canonical one
pub fn floatToIntBits(value: f32) -> i32
{
    if value.is_nan()
    {
        return 0x7FC00000;
    }
    value.to_bits() as i32
}

pub fn doubleToLongBits(value: f64) -> i64
{
    if value.is_nan()
    {
        return 0x7FF8000000000000;
    }
    value.to_bits() as i64
}

pub struct FileInputStream
{
    m_file: std::fs::File,
}

impl FileInputStream
{
    pub fn open(path: &std::path::Path) -> IoResult<Self>
    {
        match std::fs::File::open(path)
        {
            Ok(file) => Ok(Self { m_file: file }),
            Err(error) => Err(fileNotFound(path, &error)),
        }
    }
}

// FileNotFoundException carries the path and the strerror text in parentheses
pub fn fileNotFound(path: &std::path::Path, error: &std::io::Error) -> Throwable
{
    let reason: String = match error.kind()
    {
        std::io::ErrorKind::NotFound => "No such file or directory".to_owned(),
        std::io::ErrorKind::PermissionDenied => "Permission denied".to_owned(),
        std::io::ErrorKind::IsADirectory => "Is a directory".to_owned(),
        _ => error.to_string(),
    };
    ioError(&FILE_NOT_FOUND, &format!("{} ({reason})", path.display()))
}

impl InputStream for FileInputStream
{
    fn read(&mut self) -> IoResult<i32>
    {
        let mut buffer: [u8; 1] = [0];
        let count: i32 = self.readInto(&mut buffer, 0, 1)?;
        if count <= 0
        {
            return Ok(-1);
        }
        Ok(buffer[0] as i32)
    }

    fn readInto(&mut self, buf: &mut [u8], off: i32, len: i32) -> IoResult<i32>
    {
        use std::io::Read;
        checkBounds(buf.len(), off, len);
        if len == 0
        {
            return Ok(0);
        }
        match self.m_file.read(&mut buf[off as usize..(off + len) as usize])
        {
            Ok(0) => Ok(-1),
            Ok(count) => Ok(count as i32),
            Err(error) => Err(fromStd(&error)),
        }
    }

    fn skip(&mut self, n: i64) -> IoResult<i64>
    {
        use std::io::Seek;
        match self.m_file.seek(std::io::SeekFrom::Current(n))
        {
            Ok(_) => Ok(n),
            Err(error) => Err(fromStd(&error)),
        }
    }
}

pub struct FileOutputStream
{
    m_file: std::fs::File,
}

impl FileOutputStream
{
    pub fn create(path: &std::path::Path, append: bool) -> IoResult<Self>
    {
        let result: std::io::Result<std::fs::File> =
            std::fs::OpenOptions::new().write(true).create(true).append(append).truncate(!append).open(path);
        match result
        {
            Ok(file) => Ok(Self { m_file: file }),
            Err(error) => Err(fileNotFound(path, &error)),
        }
    }
}

impl OutputStream for FileOutputStream
{
    fn write(&mut self, value: i32) -> IoResult<()>
    {
        let buffer: [u8; 1] = [value as u8];
        self.writeFrom(&buffer, 0, 1)
    }

    fn writeFrom(&mut self, buf: &[u8], off: i32, len: i32) -> IoResult<()>
    {
        use std::io::Write;
        checkBounds(buf.len(), off, len);
        self.m_file.write_all(&buf[off as usize..(off + len) as usize]).map_err(|error| fromStd(&error))
    }
}

// BufferedOutputStream: bytes reach the target only when the buffer fills or on flush, which shapes packet timing
pub struct BufferedOutputStream<S: OutputStream>
{
    m_out: S,
    m_buf: Vec<u8>,
    m_capacity: usize,
}

impl<S: OutputStream> BufferedOutputStream<S>
{
    pub fn new(output: S, size: i32) -> Self
    {
        if size <= 0
        {
            throwable::throw(&throwable::ILLEGAL_ARGUMENT, "Buffer size <= 0");
        }
        Self { m_out: output, m_buf: Vec::with_capacity(size as usize), m_capacity: size as usize }
    }

    fn flushBuffer(&mut self) -> IoResult<()>
    {
        if !self.m_buf.is_empty()
        {
            let pending: Vec<u8> = std::mem::take(&mut self.m_buf);
            let len: i32 = pending.len() as i32;
            self.m_out.writeFrom(&pending, 0, len)?;
        }
        Ok(())
    }
}

impl<S: OutputStream> OutputStream for BufferedOutputStream<S>
{
    fn write(&mut self, value: i32) -> IoResult<()>
    {
        if self.m_buf.len() >= self.m_capacity
        {
            self.flushBuffer()?;
        }
        self.m_buf.push(value as u8);
        Ok(())
    }

    fn writeFrom(&mut self, buf: &[u8], off: i32, len: i32) -> IoResult<()>
    {
        if len as usize >= self.m_capacity
        {
            self.flushBuffer()?;
            return self.m_out.writeFrom(buf, off, len);
        }

        if len as usize > self.m_capacity - self.m_buf.len()
        {
            self.flushBuffer()?;
        }
        self.m_buf.extend_from_slice(&buf[off as usize..(off + len) as usize]);
        Ok(())
    }

    fn flush(&mut self) -> IoResult<()>
    {
        self.flushBuffer()?;
        self.m_out.flush()
    }

    fn close(&mut self) -> IoResult<()>
    {
        let flushed: IoResult<()> = self.flush();
        self.m_out.close()?;
        flushed
    }
}

pub struct BufferedInputStream<S: InputStream>
{
    m_in: S,
    m_buf: Vec<u8>,
    m_pos: usize,
}

impl<S: InputStream> BufferedInputStream<S>
{
    pub fn new(input: S, size: i32) -> Self
    {
        if size <= 0
        {
            throwable::throw(&throwable::ILLEGAL_ARGUMENT, "Buffer size <= 0");
        }
        Self { m_in: input, m_buf: Vec::with_capacity(size as usize), m_pos: 0 }
    }

    fn fill(&mut self) -> IoResult<()>
    {
        let capacity: usize = self.m_buf.capacity();
        self.m_buf.resize(capacity, 0);
        self.m_pos = 0;
        let count: i32 = self.m_in.readInto(&mut self.m_buf, 0, capacity as i32)?;
        self.m_buf.truncate(count.max(0) as usize);
        Ok(())
    }
}

impl<S: InputStream> InputStream for BufferedInputStream<S>
{
    fn read(&mut self) -> IoResult<i32>
    {
        if self.m_pos >= self.m_buf.len()
        {
            self.fill()?;
            if self.m_pos >= self.m_buf.len()
            {
                return Ok(-1);
            }
        }
        let value: i32 = self.m_buf[self.m_pos] as i32;
        self.m_pos += 1;
        Ok(value)
    }

    fn readInto(&mut self, buf: &mut [u8], off: i32, len: i32) -> IoResult<i32>
    {
        checkBounds(buf.len(), off, len);
        if len == 0
        {
            return Ok(0);
        }

        if self.m_pos >= self.m_buf.len()
        {
            if len as usize >= self.m_buf.capacity()
            {
                return self.m_in.readInto(buf, off, len);
            }
            self.fill()?;
            if self.m_pos >= self.m_buf.len()
            {
                return Ok(-1);
            }
        }

        let count: usize = (len as usize).min(self.m_buf.len() - self.m_pos);
        buf[off as usize..off as usize + count].copy_from_slice(&self.m_buf[self.m_pos..self.m_pos + count]);
        self.m_pos += count;
        Ok(count as i32)
    }

    fn available(&mut self) -> IoResult<i32> { Ok((self.m_buf.len() - self.m_pos) as i32 + self.m_in.available()?) }

    fn close(&mut self) -> IoResult<()> { self.m_in.close() }
}
