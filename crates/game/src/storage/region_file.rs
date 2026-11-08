// java.io.RandomAccessFile is a port stand-in
use crate::port::random_access_file::RandomAccessFile;
use jvm::array_list::JArrayList;
use jvm::io::{ByteArrayInputStream, DataInputStream, DataOutputStream, InputStream, IoResult, OutputStream};
use jvm::throwable::{self, OrNpe};
use jvm::zip::{DeflaterOutputStream, GZIPInputStream, InflaterInputStream};
use std::cell::{Cell, RefCell};
use std::path::Path;
use std::rc::Rc;

mod layout
{
    pub const SECTOR_BYTES: i32 = 4096;
    pub const SECTOR_INTS: i32 = 1024;
    pub const REGION_CHUNKS: i32 = 32;
    pub const COUNT_MASK: i32 = 0xFF;
    pub const MAX_SECTORS: i32 = 256;
    pub const INT_BYTES: i32 = 4;
    pub const VERSION_GZIP: i8 = 1;
    pub const VERSION_DEFLATE: i8 = 2;
}

pub type ChunkInputStream = DataInputStream<Box<dyn InputStream>>;
pub type ChunkOutputStream = DataOutputStream<DeflaterOutputStream<RegionFileChunkBuffer>>;

pub struct RegionFile
{
    m_dataFile: RefCell<Option<RandomAccessFile>>,
    m_offsets: RefCell<Vec<i32>>,
    m_chunkSaveTimes: RefCell<Vec<i32>>,
    m_sectorFree: Option<JArrayList<bool>>,
    m_sizeDelta: Cell<i32>,
}

impl RegionFile
{
    pub fn new(file: &Path) -> Rc<RegionFile>
    {
        let mut region: RegionFile = RegionFile {
            m_dataFile: RefCell::new(None),
            m_offsets: RefCell::new(vec![0; layout::SECTOR_INTS as usize]),
            m_chunkSaveTimes: RefCell::new(vec![0; layout::SECTOR_INTS as usize]),
            m_sectorFree: None,
            m_sizeDelta: Cell::new(0),
        };
        if let Err(error) = region.load(file)
        {
            error.printStackTrace();
        }
        Rc::new(region)
    }

    fn load(&mut self, file: &Path) -> IoResult<()>
    {
        *self.m_dataFile.get_mut() = Some(RandomAccessFile::open(file)?);
        let dataFile: &mut RandomAccessFile = self.m_dataFile.get_mut().as_mut().orNpe();
        let sectorMask: i64 = layout::SECTOR_BYTES as i64 - 1;
        if dataFile.length()? < layout::SECTOR_BYTES as i64
        {
            for _ in 0..layout::SECTOR_INTS * 2
            {
                dataFile.writeInt(0)?;
            }
            self.m_sizeDelta.set(self.m_sizeDelta.get() + layout::SECTOR_BYTES * 2);
        }
        if (dataFile.length()? & sectorMask) != 0
        {
            let mut j1: i64 = 0;
            while j1 < (dataFile.length()? & sectorMask)
            {
                dataFile.write(0)?;
                j1 += 1;
            }
        }
        let k1: i32 = dataFile.length()? as i32 / layout::SECTOR_BYTES;
        let sectorFree: &JArrayList<bool> = self.m_sectorFree.insert(JArrayList::new());
        for _ in 0..k1
        {
            sectorFree.add(true);
        }
        sectorFree.set(0, false);
        sectorFree.set(1, false);
        dataFile.seek(0)?;
        let offsets: &mut Vec<i32> = self.m_offsets.get_mut();
        for offset in offsets.iter_mut()
        {
            let k: i32 = dataFile.readInt()?;
            *offset = k;
            if k != 0 && (k >> 8) + (k & layout::COUNT_MASK) <= sectorFree.size()
            {
                for l in 0..(k & layout::COUNT_MASK)
                {
                    sectorFree.set((k >> 8) + l, false);
                }
            }
        }
        for saveTime in self.m_chunkSaveTimes.get_mut().iter_mut()
        {
            *saveTime = dataFile.readInt()?;
        }
        Ok(())
    }

    fn sectorFree(&self) -> &JArrayList<bool> { self.m_sectorFree.as_ref().orNpe() }

    fn withDataFile<R>(&self, body: impl FnOnce(&mut RandomAccessFile) -> IoResult<R>) -> IoResult<R>
    {
        let mut dataFile = self.m_dataFile.borrow_mut();
        body(dataFile.as_mut().orNpe())
    }

    pub fn resetBytesWritten(&self) -> i32 { self.m_sizeDelta.replace(0) }

    pub fn getChunkDataInputStream(&self, chunkX: i32, chunkZ: i32) -> Option<ChunkInputStream>
    {
        if self.outOfBounds(chunkX, chunkZ)
        {
            return None;
        }
        self.readChunk(chunkX, chunkZ).unwrap_or_default()
    }

    fn readChunk(&self, chunkX: i32, chunkZ: i32) -> IoResult<Option<ChunkInputStream>>
    {
        let n: i32 = self.getOffset(chunkX, chunkZ);
        if n == 0
        {
            return Ok(None);
        }
        let n2: i32 = n >> 8;
        let n3: i32 = n & layout::COUNT_MASK;
        if n2 + n3 > self.sectorFree().size()
        {
            return Ok(None);
        }
        let (n4, by, byArray): (i32, i8, Vec<u8>) = self.withDataFile(|dataFile| {
            dataFile.seek((n2 * layout::SECTOR_BYTES) as i64)?;
            let n4: i32 = dataFile.readInt()?;
            if n4 > layout::SECTOR_BYTES * n3
            {
                return Ok((n4, 0, Vec::new()));
            }
            let by: i8 = dataFile.readByte()?;
            if by != layout::VERSION_GZIP && by != layout::VERSION_DEFLATE
            {
                return Ok((n4, by, Vec::new()));
            }
            if n4 - 1 < 0
            {
                throwable::throw(&throwable::NEGATIVE_ARRAY_SIZE, &(n4 - 1).to_string());
            }
            let mut byArray: Vec<u8> = vec![0; (n4 - 1) as usize];
            dataFile.read(&mut byArray)?;
            Ok((n4, by, byArray))
        })?;
        if n4 > layout::SECTOR_BYTES * n3
        {
            return Ok(None);
        }
        let input: Box<dyn InputStream> = match by
        {
            layout::VERSION_GZIP => Box::new(GZIPInputStream::new(ByteArrayInputStream::new(byArray))?),
            layout::VERSION_DEFLATE => Box::new(InflaterInputStream::new(ByteArrayInputStream::new(byArray))),
            _ => return Ok(None),
        };
        Ok(Some(DataInputStream::new(input)))
    }

    pub fn getChunkDataOutputStream(self: &Rc<Self>, chunkX: i32, chunkZ: i32) -> Option<ChunkOutputStream>
    {
        if self.outOfBounds(chunkX, chunkZ)
        {
            return None;
        }
        let buffer: RegionFileChunkBuffer =
            RegionFileChunkBuffer { m_region: Rc::clone(self), m_chunkX: chunkX, m_chunkZ: chunkZ, m_buf: Vec::new() };
        Some(DataOutputStream::new(DeflaterOutputStream::new(buffer)))
    }

    fn write(&self, chunkX: i32, chunkZ: i32, data: &[u8], size: i32)
    {
        if let Err(error) = self.writeChunk(chunkX, chunkZ, data, size)
        {
            error.printStackTrace();
        }
    }

    fn writeChunk(&self, chunkX: i32, chunkZ: i32, data: &[u8], size: i32) -> IoResult<()>
    {
        let sectorFree: &JArrayList<bool> = self.sectorFree();
        let i: i32 = self.getOffset(chunkX, chunkZ);
        let mut j: i32 = i >> 8;
        let k: i32 = i & layout::COUNT_MASK;
        let l: i32 = (size + 5) / layout::SECTOR_BYTES + 1;
        if l >= layout::MAX_SECTORS
        {
            return Ok(());
        }
        if j != 0 && k == l
        {
            self.writeSectors(j, data, size)?;
        }
        else
        {
            for i1 in 0..k
            {
                sectorFree.set(j + i1, true);
            }
            let mut l1: i32 = sectorFree.indexOf(&true);
            let mut j1: i32 = 0;
            if l1 != -1
            {
                let mut k1: i32 = l1;
                while k1 < sectorFree.size()
                {
                    if j1 != 0
                    {
                        if sectorFree.get(k1)
                        {
                            j1 += 1;
                        }
                        else
                        {
                            j1 = 0;
                        }
                    }
                    else if sectorFree.get(k1)
                    {
                        l1 = k1;
                        j1 = 1;
                    }
                    if j1 >= l
                    {
                        break;
                    }
                    k1 += 1;
                }
            }
            if j1 >= l
            {
                j = l1;
                self.setOffset(chunkX, chunkZ, j << 8 | l)?;
                for j2 in 0..l
                {
                    sectorFree.set(j + j2, false);
                }
                self.writeSectors(j, data, size)?;
            }
            else
            {
                j = sectorFree.size();
                self.withDataFile(|dataFile| {
                    let end: i64 = dataFile.length()?;
                    dataFile.seek(end)
                })?;
                let emptySector: Vec<u8> = vec![0; layout::SECTOR_BYTES as usize];
                for _ in 0..l
                {
                    self.withDataFile(|dataFile| dataFile.writeFrom(&emptySector, 0, layout::SECTOR_BYTES))?;
                    sectorFree.add(false);
                }
                self.m_sizeDelta.set(self.m_sizeDelta.get() + layout::SECTOR_BYTES * l);
                self.writeSectors(j, data, size)?;
                self.setOffset(chunkX, chunkZ, j << 8 | l)?;
            }
        }
        self.writeChunkSaveTime(chunkX, chunkZ, (jvm::system::currentTimeMillis() / 1000) as i32)
    }

    fn writeSectors(&self, blockOffset: i32, data: &[u8], size: i32) -> IoResult<()>
    {
        self.withDataFile(|dataFile| {
            dataFile.seek((blockOffset * layout::SECTOR_BYTES) as i64)?;
            dataFile.writeInt(size + 1)?;
            dataFile.writeByte(layout::VERSION_DEFLATE as i32)?;
            dataFile.writeFrom(data, 0, size)
        })
    }

    fn outOfBounds(&self, chunkX: i32, chunkZ: i32) -> bool
    {
        chunkX < 0 || chunkX >= layout::REGION_CHUNKS || chunkZ < 0 || chunkZ >= layout::REGION_CHUNKS
    }

    fn index(chunkX: i32, chunkZ: i32) -> i32 { chunkX + chunkZ * layout::REGION_CHUNKS }

    fn getOffset(&self, chunkX: i32, chunkZ: i32) -> i32
    {
        self.m_offsets.borrow()[Self::index(chunkX, chunkZ) as usize]
    }

    pub fn hasChunkData(&self, chunkX: i32, chunkZ: i32) -> bool { self.getOffset(chunkX, chunkZ) != 0 }

    fn setOffset(&self, chunkX: i32, chunkZ: i32, blockInfo: i32) -> IoResult<()>
    {
        let index: i32 = Self::index(chunkX, chunkZ);
        self.m_offsets.borrow_mut()[index as usize] = blockInfo;
        self.withDataFile(|dataFile| {
            dataFile.seek((index * layout::INT_BYTES) as i64)?;
            dataFile.writeInt(blockInfo)
        })
    }

    fn writeChunkSaveTime(&self, chunkX: i32, chunkZ: i32, timeSeconds: i32) -> IoResult<()>
    {
        let index: i32 = Self::index(chunkX, chunkZ);
        self.m_chunkSaveTimes.borrow_mut()[index as usize] = timeSeconds;
        self.withDataFile(|dataFile| {
            dataFile.seek((layout::SECTOR_BYTES + index * layout::INT_BYTES) as i64)?;
            dataFile.writeInt(timeSeconds)
        })
    }

    pub fn close(&self) -> IoResult<()> { self.withDataFile(RandomAccessFile::close) }
}

pub struct RegionFileChunkBuffer
{
    m_region: Rc<RegionFile>,
    m_chunkX: i32,
    m_chunkZ: i32,
    m_buf: Vec<u8>,
}

impl OutputStream for RegionFileChunkBuffer
{
    fn write(&mut self, value: i32) -> IoResult<()>
    {
        self.m_buf.push(value as u8);
        Ok(())
    }

    fn writeFrom(&mut self, buf: &[u8], off: i32, len: i32) -> IoResult<()>
    {
        self.m_buf.extend_from_slice(&buf[off as usize..(off + len) as usize]);
        Ok(())
    }

    fn close(&mut self) -> IoResult<()>
    {
        self.m_region.write(self.m_chunkX, self.m_chunkZ, &self.m_buf, self.m_buf.len() as i32);
        Ok(())
    }
}
