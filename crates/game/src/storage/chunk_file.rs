// java.io.File calls are port stand-ins
use crate::port::file;
use crate::storage::save_converter_mc_region::chunkFilePattern;
use jvm::string::JString;
use jvm::throwable::rethrow;
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub struct ChunkFile
{
    m_file: PathBuf,
    m_chunkX: i32,
    m_chunkZ: i32,
}

impl ChunkFile
{
    pub fn new(path: &Path) -> ChunkFile
    {
        let (chunkX, chunkZ): (i32, i32) = match chunkFilePattern(&file::getName(path))
        {
            Some((x, z)) => (
                jvm::integer::parseIntRadix(&JString::from(x.as_str()), 36).unwrap_or_else(|e| rethrow(e)),
                jvm::integer::parseIntRadix(&JString::from(z.as_str()), 36).unwrap_or_else(|e| rethrow(e)),
            ),
            None => (0, 0),
        };
        ChunkFile { m_file: path.to_path_buf(), m_chunkX: chunkX, m_chunkZ: chunkZ }
    }

    pub fn compareTo(&self, chunkFile: &ChunkFile) -> i32
    {
        let n: i32 = self.m_chunkX >> 5;
        let n2: i32 = chunkFile.m_chunkX >> 5;
        if n == n2
        {
            let n3: i32 = self.m_chunkZ >> 5;
            let n4: i32 = chunkFile.m_chunkZ >> 5;
            return n3 - n4;
        }
        n - n2
    }

    pub fn getFile(&self) -> &Path { &self.m_file }

    pub fn getChunkX(&self) -> i32 { self.m_chunkX }

    pub fn getChunkZ(&self) -> i32 { self.m_chunkZ }
}
