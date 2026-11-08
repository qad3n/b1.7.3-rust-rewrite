use crate::entity::{Entity, EntityList};
// java.io.File calls are port stand-ins
use crate::port::file;
use crate::tileentity::TileEntity;
use crate::world::{Chunk, IChunkLoader, World, WorldInfo};
use common::nbt::{CompressedStreamTools, NBTBase, NBTTagCompound, NBTTagList};
use common::world::NibbleArray;
use jvm::io::{FileInputStream, FileOutputStream, IoResult};
use jvm::throwable::{self, OrNpe, Throwable};
use std::path::{Path, PathBuf};
use std::rc::Rc;

const HEIGHT_MAP_SIZE: usize = 256;
const FOLDER_MASK: i32 = 63;

pub struct ChunkLoader
{
    m_saveDir: PathBuf,
    m_createIfNecessary: bool,
}

pub(super) fn catchException<R>(body: impl FnOnce() -> IoResult<R>) -> Option<R>
{
    let result: Result<IoResult<R>, Throwable> = throwable::catchThrowable(&throwable::EXCEPTION, body);
    match result.and_then(|inner| inner)
    {
        Ok(value) => Some(value),
        Err(error) =>
        {
            error.printStackTrace();
            None
        }
    }
}

impl ChunkLoader
{
    pub fn new(dir: &Path, make: bool) -> ChunkLoader
    {
        ChunkLoader { m_saveDir: dir.to_path_buf(), m_createIfNecessary: make }
    }

    fn chunkFileForXZ(&self, chunkX: i32, chunkZ: i32) -> Option<PathBuf>
    {
        let s: String = format!("c.{}.{}.dat", file::toStringRadix36(chunkX), file::toStringRadix36(chunkZ));
        let s1: String = file::toStringRadix36(chunkX & FOLDER_MASK);
        let s2: String = file::toStringRadix36(chunkZ & FOLDER_MASK);
        let mut file1: PathBuf = self.m_saveDir.join(s1);
        if !file::exists(&file1)
        {
            if !self.m_createIfNecessary
            {
                return None;
            }
            file::mkdir(&file1);
        }
        file1 = file1.join(s2);
        if !file::exists(&file1)
        {
            if !self.m_createIfNecessary
            {
                return None;
            }
            file::mkdir(&file1);
        }
        file1 = file1.join(s);
        if !file::exists(&file1) && !self.m_createIfNecessary
        {
            None
        }
        else
        {
            Some(file1)
        }
    }

    pub fn storeChunkInCompound(chunk: &Rc<Chunk>, world: &Rc<World>, nbt: &NBTTagCompound)
    {
        world.checkSessionLock();
        nbt.setInteger("xPos", chunk.m_xPosition);
        nbt.setInteger("zPos", chunk.m_zPosition);
        nbt.setLong("LastUpdate", world.getWorldTime());
        nbt.setByteArray("Blocks", Rc::new(chunk.m_blocks.get()));
        nbt.setByteArray("Data", Rc::new(chunk.data().m_data.get()));
        nbt.setByteArray("SkyLight", Rc::new(chunk.skylight().m_data.get()));
        nbt.setByteArray("BlockLight", Rc::new(chunk.blocklight().m_data.get()));
        nbt.setByteArray("HeightMap", Rc::new(chunk.m_heightMap.get()));
        nbt.setBoolean("TerrainPopulated", chunk.m_isTerrainPopulated.get());
        chunk.m_hasEntities.set(false);
        let nbttaglist: Rc<NBTBase> = NBTTagList::create();
        for slice in &chunk.m_entities
        {
            for entity in slice.toVec()
            {
                chunk.m_hasEntities.set(true);
                let nbttagcompound: Rc<NBTBase> = NBTTagCompound::create();
                if entity.addEntityID(nbttagcompound.compound())
                {
                    nbttaglist.list().setTag(nbttagcompound);
                }
            }
        }
        nbt.setTag("Entities", &nbttaglist);
        let nbttaglist1: Rc<NBTBase> = NBTTagList::create();
        let tileEntities: Vec<Rc<TileEntity>> = chunk.m_chunkTileEntityMap.with(|map| map.values());
        for tileentity in tileEntities
        {
            let nbttagcompound1: Rc<NBTBase> = NBTTagCompound::create();
            tileentity.writeToNBT(nbttagcompound1.compound());
            nbttaglist1.list().setTag(nbttagcompound1);
        }
        nbt.setTag("TileEntities", &nbttaglist1);
    }

    pub fn loadChunkIntoWorldFromCompound(world: &Rc<World>, nbt: &NBTTagCompound) -> Rc<Chunk>
    {
        let i: i32 = nbt.getInteger("xPos");
        let j: i32 = nbt.getInteger("zPos");
        let chunk: Rc<Chunk> = Chunk::new(world, i, j);
        chunk.m_blocks.set((*nbt.getByteArray("Blocks").orNpe()).clone());
        chunk.m_data.set(Some(Rc::new(NibbleArray::withData((*nbt.getByteArray("Data").orNpe()).clone()))));
        chunk.m_skylightMap.set(Some(Rc::new(NibbleArray::withData((*nbt.getByteArray("SkyLight").orNpe()).clone()))));
        chunk
            .m_blocklightMap
            .set(Some(Rc::new(NibbleArray::withData((*nbt.getByteArray("BlockLight").orNpe()).clone()))));
        let heightMap: Option<Rc<Vec<i8>>> = nbt.getByteArray("HeightMap");
        if let Some(heightMap) = &heightMap
        {
            chunk.m_heightMap.set((**heightMap).clone());
        }
        chunk.m_isTerrainPopulated.set(nbt.getBoolean("TerrainPopulated"));
        let blockCount: i32 = chunk.m_blocks.with(Vec::len) as i32;
        if !chunk.data().isValid()
        {
            chunk.m_data.set(Some(Rc::new(NibbleArray::new(blockCount))));
        }
        if heightMap.is_none() || !chunk.skylight().isValid()
        {
            chunk.m_heightMap.set(vec![0; HEIGHT_MAP_SIZE]);
            chunk.m_skylightMap.set(Some(Rc::new(NibbleArray::new(blockCount))));
            chunk.populateHeightMap();
        }
        if !chunk.blocklight().isValid()
        {
            chunk.m_blocklightMap.set(Some(Rc::new(NibbleArray::new(blockCount))));
            chunk.populateLight();
        }
        let nbttaglist: Rc<NBTBase> = nbt.getTagList("Entities");
        for k in 0..nbttaglist.list().tagCount()
        {
            let nbttagcompound: Rc<NBTBase> = nbttaglist.list().tagAt(k);
            let entity: Option<Rc<Entity>> = EntityList::createEntityFromNBT(nbttagcompound.compound(), world);
            chunk.m_hasEntities.set(true);
            if let Some(entity) = entity
            {
                chunk.addEntity(&entity);
            }
        }
        let nbttaglist1: Rc<NBTBase> = nbt.getTagList("TileEntities");
        for l in 0..nbttaglist1.list().tagCount()
        {
            let nbttagcompound1: Rc<NBTBase> = nbttaglist1.list().tagAt(l);
            if let Some(tileentity) = TileEntity::createAndLoadEntity(nbttagcompound1.compound())
            {
                chunk.addTileEntity(&tileentity);
            }
        }
        chunk
    }

    pub(super) fn checkChunk(world: &Rc<World>, nbt: &Rc<NBTBase>, chunkX: i32, chunkZ: i32) -> Option<Rc<Chunk>>
    {
        let compound: &NBTTagCompound = nbt.compound();
        if !compound.hasKey("Level")
        {
            jvm::system::println(&format!("Chunk file at {chunkX},{chunkZ} is missing level data, skipping"));
            return None;
        }
        if !compound.getCompoundTag("Level").compound().hasKey("Blocks")
        {
            jvm::system::println(&format!("Chunk file at {chunkX},{chunkZ} is missing block data, skipping"));
            return None;
        }
        let mut chunk: Rc<Chunk> =
            Self::loadChunkIntoWorldFromCompound(world, compound.getCompoundTag("Level").compound());
        if !chunk.isAtLocation(chunkX, chunkZ)
        {
            jvm::system::println(&format!(
                "Chunk file at {chunkX},{chunkZ} is in the wrong location; relocating. (Expected {chunkX}, {chunkZ}, got {}, {})",
                chunk.m_xPosition, chunk.m_zPosition
            ));
            compound.setInteger("xPos", chunkX);
            compound.setInteger("zPos", chunkZ);
            chunk = Self::loadChunkIntoWorldFromCompound(world, compound.getCompoundTag("Level").compound());
        }
        chunk.fill();
        Some(chunk)
    }
}

impl IChunkLoader for ChunkLoader
{
    fn loadChunk(&self, world: &Rc<World>, chunkX: i32, chunkZ: i32) -> IoResult<Option<Rc<Chunk>>>
    {
        let Some(file1) = self.chunkFileForXZ(chunkX, chunkZ)
        else
        {
            return Ok(None);
        };
        if !file::exists(&file1)
        {
            return Ok(None);
        }
        let chunk: Option<Option<Rc<Chunk>>> = catchException(|| {
            let nbttagcompound: Rc<NBTBase> = CompressedStreamTools::readCompressed(FileInputStream::open(&file1)?)?;
            Ok(Self::checkChunk(world, &nbttagcompound, chunkX, chunkZ))
        });
        Ok(chunk.flatten())
    }

    fn saveChunk(&self, world: &Rc<World>, chunk: &Rc<Chunk>) -> IoResult<()>
    {
        world.checkSessionLock();
        let file1: PathBuf = self.chunkFileForXZ(chunk.m_xPosition, chunk.m_zPosition).orNpe();
        if file::exists(&file1)
        {
            let worldinfo: Rc<WorldInfo> = world.getWorldInfo();
            worldinfo.setSizeOnDisk(worldinfo.getSizeOnDisk() - file::length(&file1));
        }
        catchException(|| {
            let file2: PathBuf = self.m_saveDir.join("tmp_chunk.dat");
            let fileoutputstream: FileOutputStream = FileOutputStream::create(&file2, false)?;
            let nbttagcompound: Rc<NBTBase> = NBTTagCompound::create();
            let nbttagcompound1: Rc<NBTBase> = NBTTagCompound::create();
            nbttagcompound.compound().setTag("Level", &nbttagcompound1);
            Self::storeChunkInCompound(chunk, world, nbttagcompound1.compound());
            CompressedStreamTools::writeGzippedCompoundToOutputStream(&nbttagcompound, fileoutputstream)?;
            if file::exists(&file1)
            {
                file::delete(&file1);
            }
            file::renameTo(&file2, &file1);
            let worldinfo1: Rc<WorldInfo> = world.getWorldInfo();
            worldinfo1.setSizeOnDisk(worldinfo1.getSizeOnDisk() + file::length(&file1));
            Ok(())
        });
        Ok(())
    }

    fn saveExtraChunkData(&self, _world: &Rc<World>, _chunk: &Rc<Chunk>) -> IoResult<()> { Ok(()) }

    fn tick(&self) {}

    fn saveExtraData(&self) {}
}
