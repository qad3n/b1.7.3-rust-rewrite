use crate::storage::chunk_loader::{ChunkLoader, catchException};
use crate::storage::region_file::{ChunkInputStream, ChunkOutputStream};
use crate::storage::region_file_cache::RegionFileCache;
use crate::world::{Chunk, IChunkLoader, World, WorldInfo};
use common::nbt::{CompressedStreamTools, NBTBase, NBTTagCompound};
use jvm::io::IoResult;
use jvm::throwable::OrNpe;
use std::path::{Path, PathBuf};
use std::rc::Rc;

pub struct McRegionChunkLoader
{
    m_worldDir: PathBuf,
}

impl McRegionChunkLoader
{
    pub fn new(dir: &Path) -> McRegionChunkLoader { McRegionChunkLoader { m_worldDir: dir.to_path_buf() } }
}

impl IChunkLoader for McRegionChunkLoader
{
    fn loadChunk(&self, world: &Rc<World>, chunkX: i32, chunkZ: i32) -> IoResult<Option<Rc<Chunk>>>
    {
        let Some(mut dataInputStream): Option<ChunkInputStream> =
            RegionFileCache::getChunkInputStream(&self.m_worldDir, chunkX, chunkZ)
        else
        {
            return Ok(None);
        };
        let nbtTagCompound: Rc<NBTBase> = CompressedStreamTools::read(&mut dataInputStream)?;
        Ok(ChunkLoader::checkChunk(world, &nbtTagCompound, chunkX, chunkZ))
    }

    fn saveChunk(&self, world: &Rc<World>, chunk: &Rc<Chunk>) -> IoResult<()>
    {
        world.checkSessionLock();
        catchException(|| {
            let mut dataOutputStream: ChunkOutputStream =
                RegionFileCache::getChunkOutputStream(&self.m_worldDir, chunk.m_xPosition, chunk.m_zPosition).orNpe();
            let nbtTagCompound: Rc<NBTBase> = NBTTagCompound::create();
            let nbtTagCompound2: Rc<NBTBase> = NBTTagCompound::create();
            nbtTagCompound.compound().setTag("Level", &nbtTagCompound2);
            ChunkLoader::storeChunkInCompound(chunk, world, nbtTagCompound2.compound());
            CompressedStreamTools::write(&nbtTagCompound, &mut dataOutputStream)?;
            dataOutputStream.close()?;
            let worldInfo: Rc<WorldInfo> = world.getWorldInfo();
            worldInfo.setSizeOnDisk(
                worldInfo.getSizeOnDisk()
                    + RegionFileCache::getSizeDelta(&self.m_worldDir, chunk.m_xPosition, chunk.m_zPosition) as i64,
            );
            Ok(())
        });
        Ok(())
    }

    fn saveExtraChunkData(&self, _world: &Rc<World>, _chunk: &Rc<Chunk>) -> IoResult<()> { Ok(()) }

    fn tick(&self) {}

    fn saveExtraData(&self) {}
}
