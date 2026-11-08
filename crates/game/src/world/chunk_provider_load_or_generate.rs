use crate::world::{Chunk, IChunkLoader, IChunkProvider, World};
use common::world::IProgressUpdate;
use jvm::field::Field;
use jvm::throwable::{self, OrNpe};
use std::cell::Cell;
use std::rc::{Rc, Weak};

#[derive(Default)]
pub struct ChunkProviderLoadOrGenerate
{
    m_self: Field<Weak<ChunkProviderLoadOrGenerate>>,
    m_blankChunk: Field<Option<Rc<Chunk>>>,
    m_chunkProvider: Field<Option<Rc<dyn IChunkProvider>>>,
    m_chunkLoader: Field<Option<Rc<dyn IChunkLoader>>>,
    m_chunks: Field<Option<Vec<Option<Rc<Chunk>>>>>,
    m_worldObj: Field<Option<Rc<World>>>,
    pub m_lastQueriedChunkXPos: Cell<i32>,
    pub m_lastQueriedChunkZPos: Cell<i32>,
    m_lastQueriedChunk: Field<Option<Rc<Chunk>>>,
    m_curChunkX: Cell<i32>,
    m_curChunkY: Cell<i32>,
}

fn same(a: &Option<Rc<Chunk>>, b: &Option<Rc<Chunk>>) -> bool
{
    match (a, b)
    {
        (Some(a), Some(b)) => Rc::ptr_eq(a, b),
        (None, None) => true,
        _ => false,
    }
}

impl ChunkProviderLoadOrGenerate
{
    pub fn new() -> Rc<Self>
    {
        let provider: Rc<Self> = Rc::new(Self::default());
        provider.m_self.set(Rc::downgrade(&provider));
        provider
    }

    pub fn setCurrentChunkOver(&self, chunkX: i32, chunkZ: i32)
    {
        self.m_curChunkX.set(chunkX);
        self.m_curChunkY.set(chunkZ);
    }

    pub fn canChunkExist(&self, chunkX: i32, chunkZ: i32) -> bool
    {
        let b0: i32 = 15;
        chunkX >= self.m_curChunkX.get() - b0
            && chunkZ >= self.m_curChunkY.get() - b0
            && chunkX <= self.m_curChunkX.get() + b0
            && chunkZ <= self.m_curChunkY.get() + b0
    }

    fn slot(&self, k: i32) -> Option<Rc<Chunk>>
    {
        self.m_chunks.with(|chunks| chunks.as_ref().orNpe()[k as usize].clone())
    }

    fn setSlot(&self, k: i32, chunk: Option<Rc<Chunk>>)
    {
        self.m_chunks.update(|chunks| chunks.as_mut().orNpe()[k as usize] = chunk);
    }

    fn world(&self) -> Rc<World> { self.m_worldObj.get().orNpe() }

    fn loadChunkFromStorage(&self, chunkX: i32, chunkZ: i32) -> Option<Rc<Chunk>>
    {
        let Some(loader) = self.m_chunkLoader.get()
        else
        {
            return self.m_blankChunk.get();
        };
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
                self.m_blankChunk.get()
            }
        }
    }

    fn saveExtraChunkData(&self, chunk: &Rc<Chunk>)
    {
        let Some(loader) = self.m_chunkLoader.get()
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
        let Some(loader) = self.m_chunkLoader.get()
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

    fn asDyn(&self) -> Rc<dyn IChunkProvider> { self.m_self.get().upgrade().expect("provider is alive while it runs") }
}

impl IChunkProvider for ChunkProviderLoadOrGenerate
{
    fn chunkExists(&self, chunkX: i32, chunkZ: i32) -> bool
    {
        if !self.canChunkExist(chunkX, chunkZ)
        {
            return false;
        }
        if chunkX == self.m_lastQueriedChunkXPos.get()
            && chunkZ == self.m_lastQueriedChunkZPos.get()
            && self.m_lastQueriedChunk.with(Option::is_some)
        {
            return true;
        }
        let k: i32 = (chunkX & 31) + (chunkZ & 31) * 32;
        match self.slot(k)
        {
            None => false,
            Some(chunk) =>
            {
                same(&Some(Rc::clone(&chunk)), &self.m_blankChunk.get()) || chunk.isAtLocation(chunkX, chunkZ)
            }
        }
    }

    fn prepareChunk(&self, chunkX: i32, chunkZ: i32) -> Rc<Chunk> { self.provideChunk(chunkX, chunkZ) }

    fn provideChunk(&self, chunkX: i32, chunkZ: i32) -> Rc<Chunk>
    {
        if chunkX == self.m_lastQueriedChunkXPos.get()
            && chunkZ == self.m_lastQueriedChunkZPos.get()
            && let Some(chunk) = self.m_lastQueriedChunk.get()
        {
            return chunk;
        }
        if !self.world().m_findingSpawnPoint.get() && !self.canChunkExist(chunkX, chunkZ)
        {
            return self.m_blankChunk.get().orNpe();
        }

        let k: i32 = (chunkX & 31) + (chunkZ & 31) * 32;
        if !self.chunkExists(chunkX, chunkZ)
        {
            if let Some(old) = self.slot(k)
            {
                old.onChunkUnload();
                self.saveChunk(&old);
                self.saveExtraChunkData(&old);
            }

            let chunk: Rc<Chunk> = match self.loadChunkFromStorage(chunkX, chunkZ)
            {
                Some(chunk) => chunk,
                None => match self.m_chunkProvider.get()
                {
                    None => self.m_blankChunk.get().orNpe(),
                    Some(generator) =>
                    {
                        let chunk: Rc<Chunk> = generator.provideChunk(chunkX, chunkZ);
                        chunk.fill();
                        chunk
                    }
                },
            };
            self.setSlot(k, Some(Rc::clone(&chunk)));
            chunk.populateBlockLight();
            chunk.onChunkLoad();

            let this: Rc<dyn IChunkProvider> = self.asDyn();
            let populated = |x: i32, z: i32| self.provideChunk(x, z).m_isTerrainPopulated.get();
            if !chunk.m_isTerrainPopulated.get()
                && self.chunkExists(chunkX + 1, chunkZ + 1)
                && self.chunkExists(chunkX, chunkZ + 1)
                && self.chunkExists(chunkX + 1, chunkZ)
            {
                self.populate(&this, chunkX, chunkZ);
            }
            if self.chunkExists(chunkX - 1, chunkZ)
                && !populated(chunkX - 1, chunkZ)
                && self.chunkExists(chunkX - 1, chunkZ + 1)
                && self.chunkExists(chunkX, chunkZ + 1)
                && self.chunkExists(chunkX - 1, chunkZ)
            {
                self.populate(&this, chunkX - 1, chunkZ);
            }
            if self.chunkExists(chunkX, chunkZ - 1)
                && !populated(chunkX, chunkZ - 1)
                && self.chunkExists(chunkX + 1, chunkZ - 1)
                && self.chunkExists(chunkX, chunkZ - 1)
                && self.chunkExists(chunkX + 1, chunkZ)
            {
                self.populate(&this, chunkX, chunkZ - 1);
            }
            if self.chunkExists(chunkX - 1, chunkZ - 1)
                && !populated(chunkX - 1, chunkZ - 1)
                && self.chunkExists(chunkX - 1, chunkZ - 1)
                && self.chunkExists(chunkX, chunkZ - 1)
                && self.chunkExists(chunkX - 1, chunkZ)
            {
                self.populate(&this, chunkX - 1, chunkZ - 1);
            }
        }

        self.m_lastQueriedChunkXPos.set(chunkX);
        self.m_lastQueriedChunkZPos.set(chunkZ);
        let chunk: Option<Rc<Chunk>> = self.slot(k);
        self.m_lastQueriedChunk.set(chunk.clone());
        chunk.orNpe()
    }

    fn populate(&self, source: &Rc<dyn IChunkProvider>, chunkX: i32, chunkZ: i32)
    {
        let chunk: Rc<Chunk> = self.provideChunk(chunkX, chunkZ);
        if chunk.m_isTerrainPopulated.get()
        {
            return;
        }
        chunk.m_isTerrainPopulated.set(true);
        if let Some(generator) = self.m_chunkProvider.get()
        {
            generator.populate(source, chunkX, chunkZ);
            chunk.setChunkModified();
        }
    }

    fn saveChunks(&self, saveEntities: bool, listener: Option<&dyn IProgressUpdate>) -> bool
    {
        let mut i: i32 = 0;
        let mut j: i32 = 0;
        let length: i32 = self.m_chunks.with(|chunks| chunks.as_ref().orNpe().len()) as i32;
        if listener.is_some()
        {
            for k in 0..length
            {
                if let Some(chunk) = self.slot(k)
                    && chunk.needsSaving(saveEntities)
                {
                    j += 1;
                }
            }
        }

        let mut i1: i32 = 0;
        for l in 0..length
        {
            let Some(chunk) = self.slot(l)
            else
            {
                continue;
            };
            if saveEntities && !chunk.m_neverSave.get()
            {
                self.saveExtraChunkData(&chunk);
            }
            if chunk.needsSaving(saveEntities)
            {
                self.saveChunk(&chunk);
                chunk.m_isModified.set(false);
                i += 1;
                if i == 2 && !saveEntities
                {
                    return false;
                }
                if let Some(listener) = listener
                {
                    i1 += 1;
                    if i1 % 10 == 0
                    {
                        listener.setLoadingProgress(i1 * 100 / j);
                    }
                }
            }
        }

        if saveEntities
        {
            let Some(loader) = self.m_chunkLoader.get()
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
        if let Some(loader) = self.m_chunkLoader.get()
        {
            loader.tick();
        }
        self.m_chunkProvider.get().orNpe().unload100OldestChunks()
    }

    fn canSave(&self) -> bool { true }

    fn releaseChunks(&self) -> Vec<Rc<Chunk>>
    {
        self.m_lastQueriedChunk.set(None);
        let chunks: Option<Vec<Option<Rc<Chunk>>>> = self.m_chunks.take();
        self.m_chunks.set(chunks.as_ref().map(|slots| vec![None; slots.len()]));
        chunks.into_iter().flatten().flatten().collect()
    }

    fn makeString(&self) -> String
    {
        format!("ChunkCache: {}", self.m_chunks.with(|chunks| chunks.as_ref().orNpe().len()))
    }

    fn asLoadOrGenerate(&self) -> Option<Rc<ChunkProviderLoadOrGenerate>> { self.m_self.get().upgrade() }
}
