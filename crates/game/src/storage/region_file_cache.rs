use crate::port::file;
use crate::storage::region_file::{ChunkInputStream, ChunkOutputStream, RegionFile};
use jvm::hash_map::JHashMap;
use jvm::object::JavaObject;
use jvm::string::JString;
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

const MAX_OPEN_REGIONS: i32 = 256;

#[derive(Clone)]
struct FileKey(PathBuf);

impl JavaObject for FileKey
{
    fn javaEquals(&self, other: &Self) -> bool { self.0 == other.0 }

    fn javaHash(&self) -> i32 { JString::from(self.0.to_string_lossy().as_ref()).hashCode() ^ 1234321 }
}

thread_local! {
    // java soft references become strong ones that live until the cache is cleared
    static CACHE: RefCell<JHashMap<FileKey, Rc<RegionFile>>> = RefCell::new(JHashMap::new());
}

pub struct RegionFileCache;

impl RegionFileCache
{
    pub fn getRegionFile(worldDir: &Path, chunkX: i32, chunkZ: i32) -> Rc<RegionFile>
    {
        let file1: PathBuf = worldDir.join("region");
        let file2: PathBuf = file1.join(format!("r.{}.{}.mcr", chunkX >> 5, chunkZ >> 5));
        let key: FileKey = FileKey(file2.clone());
        if let Some(regionFile) = CACHE.with(|cache| cache.borrow().get(&key))
        {
            return regionFile;
        }
        if !file::exists(&file1)
        {
            file::mkdirs(&file1);
        }
        if CACHE.with(|cache| cache.borrow().size()) >= MAX_OPEN_REGIONS
        {
            Self::flush();
        }
        let regionFile: Rc<RegionFile> = RegionFile::new(&file2);
        CACHE.with(|cache| cache.borrow_mut().put(key, Rc::clone(&regionFile)));
        regionFile
    }

    pub fn flush()
    {
        let regions: Vec<Rc<RegionFile>> = CACHE.with(|cache| cache.borrow().values());
        for regionFile in regions
        {
            if let Err(error) = regionFile.close()
            {
                error.printStackTrace();
            }
        }
        CACHE.with(|cache| cache.borrow_mut().clear());
    }

    pub fn getSizeDelta(worldDir: &Path, chunkX: i32, chunkZ: i32) -> i32
    {
        Self::getRegionFile(worldDir, chunkX, chunkZ).resetBytesWritten()
    }

    pub fn getChunkInputStream(worldDir: &Path, chunkX: i32, chunkZ: i32) -> Option<ChunkInputStream>
    {
        Self::getRegionFile(worldDir, chunkX, chunkZ).getChunkDataInputStream(chunkX & 31, chunkZ & 31)
    }

    pub fn getChunkOutputStream(worldDir: &Path, chunkX: i32, chunkZ: i32) -> Option<ChunkOutputStream>
    {
        Self::getRegionFile(worldDir, chunkX, chunkZ).getChunkDataOutputStream(chunkX & 31, chunkZ & 31)
    }
}
