use crate::math::math_helper;
// java.io.File calls are port stand-ins
use crate::port::file;
use crate::storage::region_file::{ChunkOutputStream, RegionFile};
use crate::storage::region_file_cache::RegionFileCache;
use crate::storage::save_format_old::SaveFormatOld;
use crate::storage::save_old_dir::{MC_REGION_VERSION, SaveOldDir};
use crate::storage::{ChunkFile, ISaveFormat, SaveFormatComparator};
use crate::world::{ISaveHandler, WorldInfo};
use common::world::IProgressUpdate;
use jvm::io::{DataInputStream, FileInputStream, InputStream, IoResult};
use jvm::string::JString;
use jvm::throwable::{self, OrNpe, Throwable};
use jvm::zip::GZIPInputStream;
use std::path::{Path, PathBuf};
use std::rc::Rc;

const COPY_BUFFER_SIZE: usize = 4096;
const REGION_MASK: i32 = 31;

fn isChunkNameChar(c: char) -> bool { c.is_ascii_digit() || c.is_ascii_lowercase() }

fn isChunkCoordinate(text: &str) -> bool
{
    let digits: &str = text.strip_prefix('-').unwrap_or(text);
    !digits.is_empty() && digits.chars().all(isChunkNameChar)
}

pub fn chunkFilePattern(name: &str) -> Option<(String, String)>
{
    let middle: &str = name.strip_prefix("c.")?.strip_suffix(".dat")?;
    let (x, z): (&str, &str) = middle.split_once('.')?;
    if isChunkCoordinate(x) && isChunkCoordinate(z)
    {
        Some((x.to_owned(), z.to_owned()))
    }
    else
    {
        None
    }
}

fn chunkFolderPattern(path: &Path) -> bool
{
    if !file::isDirectory(path)
    {
        return false;
    }
    let name: String = file::getName(path);
    (1..=2).contains(&name.chars().count()) && name.chars().all(isChunkNameChar)
}

fn listMatching(dir: &Path, accept: impl Fn(&Path) -> bool) -> Vec<PathBuf>
{
    file::listFiles(dir).orNpe().into_iter().filter(|path| accept(path)).collect()
}

fn progress(index: i32, total: i32) -> i32 { jvm::math::roundDouble(100.0 * index as f64 / total as f64) as i32 }

pub struct SaveConverterMcRegion
{
    m_base: SaveFormatOld,
}

impl SaveConverterMcRegion
{
    pub fn new(dir: &Path) -> SaveConverterMcRegion { SaveConverterMcRegion { m_base: SaveFormatOld::new(dir) } }

    fn collectAlphaChunkFiles(&self, dir: &Path, chunkFiles: &mut Vec<ChunkFile>, chunkDirectories: &mut Vec<PathBuf>)
    {
        for file1 in listMatching(dir, chunkFolderPattern)
        {
            chunkDirectories.push(file1.clone());
            for file2 in listMatching(&file1, chunkFolderPattern)
            {
                let afile2: Vec<PathBuf> =
                    listMatching(&file2, |path| chunkFilePattern(&file::getName(path)).is_some());
                for file3 in afile2
                {
                    chunkFiles.push(ChunkFile::new(&file3));
                }
            }
        }
    }

    fn copyChunk(regionfile: &Rc<RegionFile>, chunkfile: &ChunkFile, i: i32, j: i32, abyte: &mut [u8]) -> IoResult<()>
    {
        let mut datainputstream: DataInputStream<GZIPInputStream<FileInputStream>> =
            DataInputStream::new(GZIPInputStream::new(FileInputStream::open(chunkfile.getFile())?)?);
        let mut dataoutputstream: ChunkOutputStream =
            regionfile.getChunkDataOutputStream(i & REGION_MASK, j & REGION_MASK).orNpe();
        loop
        {
            let k: i32 = datainputstream.read(abyte)?;
            if k == -1
            {
                break;
            }
            dataoutputstream.writeFrom(abyte, 0, k)?;
        }
        dataoutputstream.close()?;
        datainputstream.m_in.close()
    }

    fn convertAlphaChunksToRegions(
        &self,
        dir: &Path,
        chunkFiles: &mut [ChunkFile],
        mut index: i32,
        total: i32,
        listener: &dyn IProgressUpdate,
    )
    {
        jvm::sort::sort(chunkFiles, &mut |a: &ChunkFile, b: &ChunkFile| a.compareTo(b));
        let mut abyte: Vec<u8> = vec![0; COPY_BUFFER_SIZE];
        for chunkfile in chunkFiles.iter()
        {
            let i: i32 = chunkfile.getChunkX();
            let j: i32 = chunkfile.getChunkZ();
            let regionfile: Rc<RegionFile> = RegionFileCache::getRegionFile(dir, i, j);
            if !regionfile.hasChunkData(i & REGION_MASK, j & REGION_MASK)
            {
                let result: Result<IoResult<()>, Throwable> = throwable::catchThrowable(&throwable::IO, || {
                    Self::copyChunk(&regionfile, chunkfile, i, j, &mut abyte)
                });
                if let Err(error) = result.and_then(|inner| inner)
                {
                    if !error.instanceOf(&throwable::IO)
                    {
                        throwable::rethrow(error);
                    }
                    error.printStackTrace();
                }
            }
            index += 1;
            listener.setLoadingProgress(progress(index, total));
        }
        RegionFileCache::flush();
    }

    fn deleteAlphaChunkFiles(
        &self,
        chunkDirectories: &[PathBuf],
        mut index: i32,
        total: i32,
        listener: &dyn IProgressUpdate,
    )
    {
        for file1 in chunkDirectories
        {
            SaveFormatOld::deleteFilesAndDirs(&file::listFiles(file1).orNpe());
            file::delete(file1);
            index += 1;
            listener.setLoadingProgress(progress(index, total));
        }
    }
}

impl ISaveFormat for SaveConverterMcRegion
{
    fn getName(&self) -> String { String::from("Scaevolus' McRegion") }

    fn getSaveLoader(&self, saveName: &str, createPlayerDataDir: bool) -> Rc<dyn ISaveHandler>
    {
        Rc::new(SaveOldDir::new(&self.m_base.m_dir, saveName, createPlayerDataDir))
    }

    fn getAll(&self) -> Vec<SaveFormatComparator>
    {
        let mut arraylist: Vec<SaveFormatComparator> = Vec::new();
        for file1 in file::listFiles(&self.m_base.m_dir).orNpe()
        {
            if !file::isDirectory(&file1)
            {
                continue;
            }
            let s: String = file::getName(&file1);
            if let Some(worldinfo) = self.getData(&s)
            {
                let flag: bool = worldinfo.getSaveVersion() != MC_REGION_VERSION;
                let name: Option<JString> = worldinfo.getWorldName();
                let s1: String = if math_helper::stringNullOrLengthZero(name.as_ref())
                {
                    s.clone()
                }
                else
                {
                    JString::display(name.as_ref())
                };
                arraylist.push(SaveFormatComparator::new(
                    s,
                    s1,
                    worldinfo.getLastTimePlayed(),
                    worldinfo.getSizeOnDisk(),
                    flag,
                ));
            }
        }
        arraylist
    }

    fn flushCache(&self) { RegionFileCache::flush(); }

    fn getData(&self, saveName: &str) -> Option<Rc<WorldInfo>> { self.m_base.getData(saveName) }

    fn delete(&self, saveName: &str) { self.m_base.delete(saveName); }

    fn rename(&self, saveName: &str, newName: &str) { self.m_base.rename(saveName, newName); }

    fn isOldMapFormat(&self, saveName: &str) -> bool
    {
        self.getData(saveName).is_some_and(|worldinfo| worldinfo.getSaveVersion() == 0)
    }

    fn convertMapFormat(&self, saveName: &str, listener: &dyn IProgressUpdate) -> bool
    {
        listener.setLoadingProgress(0);
        let mut arraylist: Vec<ChunkFile> = Vec::new();
        let mut arraylist1: Vec<PathBuf> = Vec::new();
        let mut arraylist2: Vec<ChunkFile> = Vec::new();
        let mut arraylist3: Vec<PathBuf> = Vec::new();
        let file1: PathBuf = self.m_base.m_dir.join(saveName);
        let file2: PathBuf = file1.join("DIM-1");
        jvm::system::println("Scanning folders...");
        self.collectAlphaChunkFiles(&file1, &mut arraylist, &mut arraylist1);
        if file::exists(&file2)
        {
            self.collectAlphaChunkFiles(&file2, &mut arraylist2, &mut arraylist3);
        }
        let i: i32 = (arraylist.len() + arraylist2.len() + arraylist1.len() + arraylist3.len()) as i32;
        jvm::system::println(&format!("Total conversion count is {i}"));
        let size: i32 = arraylist.len() as i32;
        let size2: i32 = arraylist2.len() as i32;
        let size1: i32 = arraylist1.len() as i32;
        self.convertAlphaChunksToRegions(&file1, &mut arraylist, 0, i, listener);
        self.convertAlphaChunksToRegions(&file2, &mut arraylist2, size, i, listener);
        let worldinfo: Rc<WorldInfo> = self.getData(saveName).orNpe();
        worldinfo.setSaveVersion(MC_REGION_VERSION);
        let isavehandler: Rc<dyn ISaveHandler> = self.getSaveLoader(saveName, false);
        isavehandler.saveWorldInfo(&worldinfo);
        self.deleteAlphaChunkFiles(&arraylist1, size + size2, i, listener);
        if file::exists(&file2)
        {
            self.deleteAlphaChunkFiles(&arraylist3, size + size2 + size1, i, listener);
        }
        true
    }
}
