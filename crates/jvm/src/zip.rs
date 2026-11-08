// java.util.zip on flate2 for now, the byte exact zlib 1.2.3 port replaces the encoder later
use crate::io::{self, InputStream, IoResult, OutputStream};
use crate::throwable::{self, Throwable, ThrowableClass};
use flate2::{Compression, Crc};
use std::io::{Read, Write};

pub static ZIP: ThrowableClass =
    ThrowableClass { m_name: "java.util.zip.ZipException", m_parent: Some(&throwable::IO) };
pub static DATA_FORMAT: ThrowableClass =
    ThrowableClass { m_name: "java.util.zip.DataFormatException", m_parent: Some(&throwable::EXCEPTION) };

const DEFAULT_LEVEL: u32 = 6;
const GZIP_HEADER: [u8; 10] = [0x1F, 0x8B, 8, 0, 0, 0, 0, 0, 0, 0];

struct StdRead<S: InputStream>
{
    m_in: S,
}

impl<S: InputStream> Read for StdRead<S>
{
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize>
    {
        let len: i32 = buf.len() as i32;
        match self.m_in.readInto(buf, 0, len)
        {
            Ok(count) if count < 0 => Ok(0),
            Ok(count) => Ok(count as usize),
            Err(error) => Err(std::io::Error::other(error.to_string())),
        }
    }
}

struct StdWrite<S: OutputStream>
{
    m_out: S,
}

impl<S: OutputStream> Write for StdWrite<S>
{
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize>
    {
        let len: i32 = buf.len() as i32;
        match self.m_out.writeFrom(buf, 0, len)
        {
            Ok(()) => Ok(buf.len()),
            Err(error) => Err(std::io::Error::other(error.to_string())),
        }
    }

    fn flush(&mut self) -> std::io::Result<()>
    {
        self.m_out.flush().map_err(|error| std::io::Error::other(error.to_string()))
    }
}

fn zipError(error: &std::io::Error) -> Throwable
{
    match error.kind()
    {
        std::io::ErrorKind::UnexpectedEof =>
        {
            Throwable::new(&throwable::EOF, Some("Unexpected end of ZLIB input stream".to_owned()))
        }
        std::io::ErrorKind::InvalidInput | std::io::ErrorKind::InvalidData =>
        {
            Throwable::new(&ZIP, Some(error.to_string()))
        }
        _ => io::ioError(&throwable::IO, &error.to_string()),
    }
}

fn readByte(reader: &mut impl Read) -> IoResult<i32>
{
    let mut buffer: [u8; 1] = [0];
    match reader.read(&mut buffer)
    {
        Ok(0) => Ok(-1),
        Ok(_) => Ok(buffer[0] as i32),
        Err(error) => Err(zipError(&error)),
    }
}

fn readSlice(reader: &mut impl Read, buf: &mut [u8], off: i32, len: i32) -> IoResult<i32>
{
    if len == 0
    {
        return Ok(0);
    }
    match reader.read(&mut buf[off as usize..(off + len) as usize])
    {
        Ok(0) => Ok(-1),
        Ok(count) => Ok(count as i32),
        Err(error) => Err(zipError(&error)),
    }
}

pub struct GZIPInputStream<S: InputStream>
{
    m_decoder: flate2::read::GzDecoder<StdRead<S>>,
}

impl<S: InputStream> GZIPInputStream<S>
{
    pub fn new(input: S) -> IoResult<Self>
    {
        let mut decoder: flate2::read::GzDecoder<StdRead<S>> = flate2::read::GzDecoder::new(StdRead { m_in: input });
        if decoder.header().is_none()
        {
            let mut probe: [u8; 0] = [];
            if let Err(error) = decoder.read(&mut probe)
            {
                return Err(zipError(&error));
            }
            if decoder.header().is_none()
            {
                return Err(io::ioError(&ZIP, "Not in GZIP format"));
            }
        }
        Ok(Self { m_decoder: decoder })
    }
}

impl<S: InputStream> InputStream for GZIPInputStream<S>
{
    fn read(&mut self) -> IoResult<i32> { readByte(&mut self.m_decoder) }

    fn readInto(&mut self, buf: &mut [u8], off: i32, len: i32) -> IoResult<i32>
    {
        readSlice(&mut self.m_decoder, buf, off, len)
    }

    fn close(&mut self) -> IoResult<()> { self.m_decoder.get_mut().m_in.close() }
}

pub struct InflaterInputStream<S: InputStream>
{
    m_decoder: flate2::read::ZlibDecoder<StdRead<S>>,
}

impl<S: InputStream> InflaterInputStream<S>
{
    pub fn new(input: S) -> Self { Self { m_decoder: flate2::read::ZlibDecoder::new(StdRead { m_in: input }) } }
}

impl<S: InputStream> InputStream for InflaterInputStream<S>
{
    fn read(&mut self) -> IoResult<i32> { readByte(&mut self.m_decoder) }

    fn readInto(&mut self, buf: &mut [u8], off: i32, len: i32) -> IoResult<i32>
    {
        readSlice(&mut self.m_decoder, buf, off, len)
    }

    fn close(&mut self) -> IoResult<()> { self.m_decoder.get_mut().m_in.close() }
}

// java writes a fixed ten byte header with mtime 0 and os 0, then raw deflate, crc32 and size
pub struct GZIPOutputStream<S: OutputStream>
{
    m_encoder: Option<flate2::write::DeflateEncoder<StdWrite<S>>>,
    m_crc: Crc,
}

impl<S: OutputStream> GZIPOutputStream<S>
{
    pub fn new(output: S) -> IoResult<Self>
    {
        let mut output: S = output;
        output.writeFrom(&GZIP_HEADER, 0, GZIP_HEADER.len() as i32)?;
        let encoder: flate2::write::DeflateEncoder<StdWrite<S>> =
            flate2::write::DeflateEncoder::new(StdWrite { m_out: output }, Compression::new(DEFAULT_LEVEL));
        Ok(Self { m_encoder: Some(encoder), m_crc: Crc::new() })
    }

    fn encoder(&mut self) -> IoResult<&mut flate2::write::DeflateEncoder<StdWrite<S>>>
    {
        match self.m_encoder.as_mut()
        {
            Some(encoder) => Ok(encoder),
            None => Err(io::ioError(&throwable::IO, "write beyond end of stream")),
        }
    }
}

impl<S: OutputStream> OutputStream for GZIPOutputStream<S>
{
    fn write(&mut self, value: i32) -> IoResult<()>
    {
        let buffer: [u8; 1] = [value as u8];
        self.writeFrom(&buffer, 0, 1)
    }

    fn writeFrom(&mut self, buf: &[u8], off: i32, len: i32) -> IoResult<()>
    {
        let slice: &[u8] = &buf[off as usize..(off + len) as usize];
        self.m_crc.update(slice);
        self.encoder()?.write_all(slice).map_err(|error| zipError(&error))
    }

    fn flush(&mut self) -> IoResult<()> { self.encoder()?.flush().map_err(|error| zipError(&error)) }

    fn close(&mut self) -> IoResult<()>
    {
        let Some(encoder) = self.m_encoder.take()
        else
        {
            return Ok(());
        };
        let mut sink: StdWrite<S> = encoder.finish().map_err(|error| zipError(&error))?;
        let mut trailer: Vec<u8> = self.m_crc.sum().to_le_bytes().to_vec();
        trailer.extend_from_slice(&self.m_crc.amount().to_le_bytes());
        sink.m_out.writeFrom(&trailer, 0, trailer.len() as i32)?;
        sink.m_out.close()
    }
}

pub struct DeflaterOutputStream<S: OutputStream>
{
    m_encoder: Option<flate2::write::ZlibEncoder<StdWrite<S>>>,
}

impl<S: OutputStream> DeflaterOutputStream<S>
{
    pub fn new(output: S) -> Self
    {
        Self {
            m_encoder: Some(flate2::write::ZlibEncoder::new(
                StdWrite { m_out: output },
                Compression::new(DEFAULT_LEVEL),
            )),
        }
    }
}

impl<S: OutputStream> OutputStream for DeflaterOutputStream<S>
{
    fn write(&mut self, value: i32) -> IoResult<()>
    {
        let buffer: [u8; 1] = [value as u8];
        self.writeFrom(&buffer, 0, 1)
    }

    fn writeFrom(&mut self, buf: &[u8], off: i32, len: i32) -> IoResult<()>
    {
        match self.m_encoder.as_mut()
        {
            Some(encoder) =>
            {
                encoder.write_all(&buf[off as usize..(off + len) as usize]).map_err(|error| zipError(&error))
            }
            None => Err(io::ioError(&throwable::IO, "write beyond end of stream")),
        }
    }

    fn close(&mut self) -> IoResult<()>
    {
        let Some(encoder) = self.m_encoder.take()
        else
        {
            return Ok(());
        };
        let mut sink: StdWrite<S> = encoder.finish().map_err(|error| zipError(&error))?;
        sink.m_out.close()
    }
}

// java.util.zip.Deflater and Inflater over whole buffers, as Packet51MapChunk uses them
pub struct Deflater
{
    m_compress: flate2::Compress,
    m_input: Vec<u8>,
    m_finish: bool,
}

impl Deflater
{
    pub fn new(level: i32) -> Self
    {
        let level: u32 = if level < 0
        {
            DEFAULT_LEVEL
        }
        else
        {
            level as u32
        };
        Self { m_compress: flate2::Compress::new(Compression::new(level), true), m_input: Vec::new(), m_finish: false }
    }

    pub fn setInput(&mut self, input: &[u8]) { self.m_input = input.to_vec(); }

    pub fn finish(&mut self) { self.m_finish = true; }

    pub fn deflate(&mut self, output: &mut [u8]) -> i32
    {
        let consumed: usize = self.m_compress.total_in() as usize;
        let before: u64 = self.m_compress.total_out();
        let flush: flate2::FlushCompress = if self.m_finish
        {
            flate2::FlushCompress::Finish
        }
        else
        {
            flate2::FlushCompress::None
        };
        let pending: Vec<u8> = self.m_input[consumed.min(self.m_input.len())..].to_vec();
        let _status: Result<flate2::Status, flate2::CompressError> = self.m_compress.compress(&pending, output, flush);
        (self.m_compress.total_out() - before) as i32
    }

    pub fn end(&mut self) {}
}

pub struct Inflater
{
    m_decompress: flate2::Decompress,
    m_input: Vec<u8>,
}

impl Inflater
{
    pub fn new() -> Self { Self { m_decompress: flate2::Decompress::new(true), m_input: Vec::new() } }

    pub fn setInput(&mut self, input: &[u8]) { self.m_input = input.to_vec(); }

    pub fn inflate(&mut self, output: &mut [u8]) -> Result<i32, Throwable>
    {
        let consumed: usize = self.m_decompress.total_in() as usize;
        let before: u64 = self.m_decompress.total_out();
        let pending: Vec<u8> = self.m_input[consumed.min(self.m_input.len())..].to_vec();
        match self.m_decompress.decompress(&pending, output, flate2::FlushDecompress::None)
        {
            Ok(_) => Ok((self.m_decompress.total_out() - before) as i32),
            Err(error) => Err(io::ioError(&DATA_FORMAT, &error.to_string())),
        }
    }

    pub fn end(&mut self) {}
}

impl Default for Inflater
{
    fn default() -> Self { Self::new() }
}
