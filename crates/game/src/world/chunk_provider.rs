use crate::world::{Chunk, IChunkLoader, IChunkProvider, World};
use common::world::{ChunkCoordIntPair, IProgressUpdate};
use jvm::array_list::JArrayList;
use jvm::field::Field;
use jvm::hash_map::{JHashMap, JHashSet};
use jvm::throwable::{self, OrNpe};
use std::rc::{Rc, Weak};

pub struct ChunkProvider
{
    m_self: Weak<ChunkProvider>,
    m_droppedChunksSet: Field<JHashSet<i32>>,
    m_empty: Rc<Chunk>,
    m_chunkProvider: Option<Rc<dyn IChunkProvider>>,
    m_chunkLoader: Option<Rc<dyn IChunkLoader>>,
    m_chunkMap: Field<JHashMap<i32, Rc<Chunk>>>,
    m_chunkList: JArrayList<Rc<Chunk>>,
    m_world: Rc<World>,
}

impl ChunkProvider
{
    fn world(&self) -> Rc<World> { Rc::clone(&self.m_world) }

    pub fn new(
        world: &Rc<World>,
        storage: Rc<dyn IChunkLoader>,
        generator: Option<Rc<dyn IChunkProvider>>,
    ) -> Rc<dyn IChunkProvider>
    {
        let provider: Rc<Self> = Rc::new_cyclic(|this| Self {
            m_self: this.clone(),
            m_droppedChunksSet: Field::new(JHashSet::new()),
            m_empty: Chunk::newEmptyWithBlocks(world, vec![0; 32768], 0, 0),
            m_chunkProvider: generator,
            m_chunkLoader: Some(storage),
            m_chunkMap: Field::new(JHashMap::new()),
            m_chunkList: JArrayList::new(),
            m_world: Rc::clone(world),
        });
        provider
    }

    fn asDyn(&self) -> Rc<dyn IChunkProvider> { self.m_self.upgrade().expect("chunk provider is alive while it runs") }

    fn loadChunkFromFile(&self, chunkX: i32, chunkZ: i32) -> Option<Rc<Chunk>>
    {
        let loader: &Rc<dyn IChunkLoader> = self.m_chunkLoader.as_ref()?;
        let result: Result<jvm::io::IoResult<Option<Rc<Chunk>>>, throwable::Throwable> =
            throwable::catchThrowable(&throwable::EXCEPTION, || loader.loadChunk(&self.world(), chunkX, chunkZ));
        match result
        {
            Ok(Ok(chunk)) =>
            {
                if let Some(chunk) = &chunk
                {
                    chunk.m_lastSaveTime.set(self.world().getWorldTime());
                }
                chunk
            }
            Ok(Err(exception)) | Err(exception) =>
            {
                jvm::system::printStackTrace(&exception);
                None
            }
        }
    }

    fn saveEntities(&self, chunk: &Rc<Chunk>)
    {
        let Some(loader) = &self.m_chunkLoader
        else
        {
            return;
        };
        let result: Result<jvm::io::IoResult<()>, throwable::Throwable> =
            throwable::catchThrowable(&throwable::EXCEPTION, || loader.saveExtraChunkData(&self.world(), chunk));
        if let Ok(Err(exception)) | Err(exception) = result
        {
            jvm::system::printStackTrace(&exception);
        }
    }

    fn saveChunk(&self, chunk: &Rc<Chunk>)
    {
        let Some(loader) = &self.m_chunkLoader
        else
        {
            return;
        };
        chunk.m_lastSaveTime.set(self.world().getWorldTime());
        if let Err(exception) = loader.saveChunk(&self.world(), chunk)
        {
            jvm::system::printStackTrace(&exception);
        }
    }

    fn populated(&self, chunkX: i32, chunkZ: i32) -> bool
    {
        self.provideChunk(chunkX, chunkZ).m_isTerrainPopulated.get()
    }
}

impl IChunkProvider for ChunkProvider
{
    fn chunkExists(&self, chunkX: i32, chunkZ: i32) -> bool
    {
        self.m_chunkMap.with(|map| map.containsKey(&ChunkCoordIntPair::chunkXZ2Int(chunkX, chunkZ)))
    }

    fn prepareChunk(&self, chunkX: i32, chunkZ: i32) -> Rc<Chunk>
    {
        let key: i32 = ChunkCoordIntPair::chunkXZ2Int(chunkX, chunkZ);
        self.m_droppedChunksSet.update(|set| set.remove(&key));
        if let Some(chunk) = self.m_chunkMap.with(|map| map.get(&key))
        {
            return chunk;
        }

        let chunk: Rc<Chunk> = match self.loadChunkFromFile(chunkX, chunkZ)
        {
            Some(chunk) => chunk,
            None => match &self.m_chunkProvider
            {
                None => Rc::clone(&self.m_empty),
                Some(generator) => generator.provideChunk(chunkX, chunkZ),
            },
        };
        self.m_chunkMap.update(|map| map.put(key, Rc::clone(&chunk)));
        self.m_chunkList.add(Rc::clone(&chunk));
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
        match self.m_chunkMap.with(|map| map.get(&ChunkCoordIntPair::chunkXZ2Int(chunkX, chunkZ)))
        {
            Some(chunk) => chunk,
            None => self.prepareChunk(chunkX, chunkZ),
        }
    }

    fn populate(&self, source: &Rc<dyn IChunkProvider>, chunkX: i32, chunkZ: i32)
    {
        let chunk: Rc<Chunk> = self.provideChunk(chunkX, chunkZ);
        if chunk.m_isTerrainPopulated.get()
        {
            return;
        }
        chunk.m_isTerrainPopulated.set(true);
        if let Some(generator) = &self.m_chunkProvider
        {
            generator.populate(source, chunkX, chunkZ);
            chunk.setChunkModified();
        }
    }

    fn saveChunks(&self, saveEntities: bool, _listener: Option<&dyn IProgressUpdate>) -> bool
    {
        let mut n: i32 = 0;
        let mut i: i32 = 0;
        while i < self.m_chunkList.size()
        {
            let chunk: Rc<Chunk> = self.m_chunkList.get(i);
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
            let Some(loader) = &self.m_chunkLoader
            else
            {
                return true;
            };
            loader.saveExtraData();
        }
        true
    }

    fn unload100OldestChunks(&self) -> bool
    {
        for _ in 0..100
        {
            if self.m_droppedChunksSet.with(|set| !set.isEmpty())
            {
                let n: i32 = self.m_droppedChunksSet.with(|set| {
                    let mut cursor: jvm::hash_map::HashCursor = set.cursor();
                    set.nextKey(&mut cursor)
                });
                let chunk: Rc<Chunk> = self.m_chunkMap.with(|map| map.get(&n)).orNpe();
                chunk.onChunkUnload();
                self.saveChunk(&chunk);
                self.saveEntities(&chunk);
                self.m_droppedChunksSet.update(|set| set.remove(&n));
                self.m_chunkMap.update(|map| map.remove(&n));
                self.m_chunkList.removeIdentity(&chunk);
            }
        }
        if let Some(loader) = &self.m_chunkLoader
        {
            loader.tick();
        }
        self.m_chunkProvider.as_ref().orNpe().unload100OldestChunks()
    }

    fn canSave(&self) -> bool { true }

    fn releaseChunks(&self) -> Vec<Rc<Chunk>>
    {
        let chunks: Vec<Rc<Chunk>> = self.m_chunkList.toVec();
        self.m_chunkList.clear();
        self.m_chunkMap.update(JHashMap::clear);
        self.m_droppedChunksSet.update(JHashSet::clear);
        chunks
    }

    fn makeString(&self) -> String
    {
        format!(
            "ServerChunkCache: {} Drop: {}",
            self.m_chunkMap.with(JHashMap::size),
            self.m_droppedChunksSet.with(JHashSet::size)
        )
    }
}
