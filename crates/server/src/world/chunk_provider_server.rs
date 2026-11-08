use crate::world::world_server;
use common::world::{ChunkCoordIntPair, IProgressUpdate};
use game::world::{Chunk, ChunkCoordinates, IChunkLoader, IChunkProvider, World};
use jvm::array_list::JArrayList;
use jvm::field::Field;
use jvm::hash_map::{HashCursor, JHashMap, JHashSet};
use jvm::io::IoResult;
use jvm::throwable::{self, OrNpe, Throwable};
use std::cell::Cell;
use std::rc::{Rc, Weak};

pub struct ChunkProviderServer
{
    m_self: Weak<ChunkProviderServer>,
    m_chunksToUnload: Field<JHashSet<i32>>,
    m_dummyChunk: Rc<Chunk>,
    m_serverChunkGenerator: Option<Rc<dyn IChunkProvider>>,
    m_storage: Option<Rc<dyn IChunkLoader>>,
    pub m_chunkLoadOverride: Cell<bool>,
    m_id2ChunkMap: Field<JHashMap<i32, Rc<Chunk>>>,
    m_chunks: JArrayList<Rc<Chunk>>,
    m_world: Rc<World>,
}

impl ChunkProviderServer
{
    pub fn new(
        world: &Rc<World>,
        storage: Rc<dyn IChunkLoader>,
        generator: Rc<dyn IChunkProvider>,
    ) -> Rc<ChunkProviderServer>
    {
        Rc::new_cyclic(|this| ChunkProviderServer {
            m_self: this.clone(),
            m_chunksToUnload: Field::new(JHashSet::new()),
            m_chunkLoadOverride: Cell::new(false),
            m_id2ChunkMap: Field::new(JHashMap::new()),
            m_chunks: JArrayList::new(),
            m_dummyChunk: Chunk::newEmptyWithBlocks(world, vec![0; 32768], 0, 0),
            m_world: Rc::clone(world),
            m_storage: Some(storage),
            m_serverChunkGenerator: Some(generator),
        })
    }

    fn asDyn(&self) -> Rc<dyn IChunkProvider> { self.m_self.upgrade().expect("chunk provider is alive while it runs") }

    pub fn unloadChunk(&self, chunkX: i32, chunkZ: i32)
    {
        let chunkCoordinates: ChunkCoordinates = self.m_world.getSpawnPoint();
        let n: i32 = chunkX * 16 + 8 - chunkCoordinates.m_x;
        let n2: i32 = chunkZ * 16 + 8 - chunkCoordinates.m_z;
        let n3: i32 = 128;
        if n < -n3 || n > n3 || n2 < -n3 || n2 > n3
        {
            self.m_chunksToUnload.update(|set| set.add(ChunkCoordIntPair::chunkXZ2Int(chunkX, chunkZ)));
        }
    }

    pub fn loadChunk(&self, chunkX: i32, chunkZ: i32) -> Rc<Chunk> { self.prepareChunk(chunkX, chunkZ) }

    fn populated(&self, chunkX: i32, chunkZ: i32) -> bool
    {
        self.provideChunk(chunkX, chunkZ).m_isTerrainPopulated.get()
    }

    fn loadChunkFromStorage(&self, chunkX: i32, chunkZ: i32) -> Option<Rc<Chunk>>
    {
        let storage: &Rc<dyn IChunkLoader> = self.m_storage.as_ref()?;
        let result: Result<IoResult<Option<Rc<Chunk>>>, Throwable> =
            throwable::catchThrowable(&throwable::EXCEPTION, || storage.loadChunk(&self.m_world, chunkX, chunkZ));
        match result.and_then(|inner| inner)
        {
            Ok(loadChunk) =>
            {
                if let Some(loadChunk) = &loadChunk
                {
                    loadChunk.m_lastSaveTime.set(self.m_world.getWorldTime());
                }
                loadChunk
            }
            Err(ex) =>
            {
                ex.printStackTrace();
                None
            }
        }
    }

    fn saveEntities(&self, chunk: &Rc<Chunk>)
    {
        let Some(storage) = &self.m_storage
        else
        {
            return;
        };
        let result: Result<IoResult<()>, Throwable> =
            throwable::catchThrowable(&throwable::EXCEPTION, || storage.saveExtraChunkData(&self.m_world, chunk));
        if let Err(ex) = result.and_then(|inner| inner)
        {
            ex.printStackTrace();
        }
    }

    fn saveChunk(&self, chunk: &Rc<Chunk>)
    {
        let Some(storage) = &self.m_storage
        else
        {
            return;
        };
        chunk.m_lastSaveTime.set(self.m_world.getWorldTime());
        if let Err(ex) = storage.saveChunk(&self.m_world, chunk)
        {
            if !ex.instanceOf(&throwable::IO)
            {
                throwable::rethrow(ex);
            }
            ex.printStackTrace();
        }
    }
}

impl IChunkProvider for ChunkProviderServer
{
    fn chunkExists(&self, chunkX: i32, chunkZ: i32) -> bool
    {
        self.m_id2ChunkMap.with(|map| map.containsKey(&ChunkCoordIntPair::chunkXZ2Int(chunkX, chunkZ)))
    }

    fn prepareChunk(&self, chunkX: i32, chunkZ: i32) -> Rc<Chunk>
    {
        let chunkXZ2Int: i32 = ChunkCoordIntPair::chunkXZ2Int(chunkX, chunkZ);
        self.m_chunksToUnload.update(|set| set.remove(&chunkXZ2Int));
        if let Some(chunk) = self.m_id2ChunkMap.with(|map| map.get(&chunkXZ2Int))
        {
            return chunk;
        }
        let chunk: Rc<Chunk> = match self.loadChunkFromStorage(chunkX, chunkZ)
        {
            Some(chunk) => chunk,
            None => match &self.m_serverChunkGenerator
            {
                None => Rc::clone(&self.m_dummyChunk),
                Some(generator) => generator.provideChunk(chunkX, chunkZ),
            },
        };
        self.m_id2ChunkMap.update(|map| map.put(chunkXZ2Int, Rc::clone(&chunk)));
        self.m_chunks.add(Rc::clone(&chunk));
        chunk.populateBlockLight();
        chunk.onChunkLoad();
        let this: Rc<dyn IChunkProvider> = self.asDyn();
        if !chunk.m_isTerrainPopulated.get()
            && self.chunkExists(chunkX + 1, chunkZ + 1)
            && self.chunkExists(chunkX, chunkZ + 1)
            && self.chunkExists(chunkX + 1, chunkZ)
        {
            self.populate(&this, chunkX, chunkZ);
        }
        if self.chunkExists(chunkX - 1, chunkZ)
            && !self.populated(chunkX - 1, chunkZ)
            && self.chunkExists(chunkX - 1, chunkZ + 1)
            && self.chunkExists(chunkX, chunkZ + 1)
            && self.chunkExists(chunkX - 1, chunkZ)
        {
            self.populate(&this, chunkX - 1, chunkZ);
        }
        if self.chunkExists(chunkX, chunkZ - 1)
            && !self.populated(chunkX, chunkZ - 1)
            && self.chunkExists(chunkX + 1, chunkZ - 1)
            && self.chunkExists(chunkX, chunkZ - 1)
            && self.chunkExists(chunkX + 1, chunkZ)
        {
            self.populate(&this, chunkX, chunkZ - 1);
        }
        if self.chunkExists(chunkX - 1, chunkZ - 1)
            && !self.populated(chunkX - 1, chunkZ - 1)
            && self.chunkExists(chunkX - 1, chunkZ - 1)
            && self.chunkExists(chunkX, chunkZ - 1)
            && self.chunkExists(chunkX - 1, chunkZ)
        {
            self.populate(&this, chunkX - 1, chunkZ - 1);
        }
        chunk
    }

    fn provideChunk(&self, chunkX: i32, chunkZ: i32) -> Rc<Chunk>
    {
        match self.m_id2ChunkMap.with(|map| map.get(&ChunkCoordIntPair::chunkXZ2Int(chunkX, chunkZ)))
        {
            Some(chunk) => chunk,
            None =>
            {
                if self.m_world.m_findingSpawnPoint.get() || self.m_chunkLoadOverride.get()
                {
                    return self.prepareChunk(chunkX, chunkZ);
                }
                Rc::clone(&self.m_dummyChunk)
            }
        }
    }

    fn populate(&self, source: &Rc<dyn IChunkProvider>, chunkX: i32, chunkZ: i32)
    {
        let provideChunk: Rc<Chunk> = self.provideChunk(chunkX, chunkZ);
        if !provideChunk.m_isTerrainPopulated.get()
        {
            provideChunk.m_isTerrainPopulated.set(true);
            if let Some(generator) = &self.m_serverChunkGenerator
            {
                generator.populate(source, chunkX, chunkZ);
                provideChunk.setChunkModified();
            }
        }
    }

    fn saveChunks(&self, saveEntities: bool, _listener: Option<&dyn IProgressUpdate>) -> bool
    {
        let mut n: i32 = 0;
        let mut i: i32 = 0;
        while i < self.m_chunks.size()
        {
            let chunk: Rc<Chunk> = self.m_chunks.get(i);
            if saveEntities && !chunk.m_neverSave.get()
            {
                self.saveEntities(&chunk);
            }
            if chunk.needsSaving(saveEntities)
            {
                self.saveChunk(&chunk);
                chunk.m_isModified.set(false);
                n += 1;
                if n == 24 && !saveEntities
                {
                    return false;
                }
            }
            i += 1;
        }
        if saveEntities
        {
            let Some(storage) = &self.m_storage
            else
            {
                return true;
            };
            storage.saveExtraData();
        }
        true
    }

    fn unload100OldestChunks(&self) -> bool
    {
        if !world_server::fields(&self.m_world).m_levelSaving.get()
        {
            for _ in 0..100
            {
                if self.m_chunksToUnload.with(|set| !set.isEmpty())
                {
                    let n: i32 = self.m_chunksToUnload.with(|set| {
                        let mut cursor: HashCursor = set.cursor();
                        set.nextKey(&mut cursor)
                    });
                    let chunk: Rc<Chunk> = self.m_id2ChunkMap.with(|map| map.get(&n)).orNpe();
                    chunk.onChunkUnload();
                    self.saveChunk(&chunk);
                    self.saveEntities(&chunk);
                    self.m_chunksToUnload.update(|set| set.remove(&n));
                    self.m_id2ChunkMap.update(|map| map.remove(&n));
                    self.m_chunks.removeIdentity(&chunk);
                }
            }
            if let Some(storage) = &self.m_storage
            {
                storage.tick();
            }
        }
        self.m_serverChunkGenerator.as_ref().orNpe().unload100OldestChunks()
    }

    fn canSave(&self) -> bool { !world_server::fields(&self.m_world).m_levelSaving.get() }

    fn makeString(&self) -> String { game::port::abstractMethod("makeString") }
}
