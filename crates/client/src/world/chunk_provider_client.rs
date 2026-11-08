use common::world::{ChunkCoordIntPair, IProgressUpdate};
use game::world::{Chunk, IChunkProvider, World};
use jvm::array_list::JArrayList;
use jvm::field::Field;
use jvm::hash_map::JHashMap;
use std::rc::Rc;

pub struct ChunkProviderClient
{
    m_blankChunk: Rc<Chunk>,
    m_chunkMapping: Field<JHashMap<ChunkCoordIntPair, Rc<Chunk>>>,
    m_chunks: JArrayList<Rc<Chunk>>,
    m_worldObj: Rc<World>,
}

impl ChunkProviderClient
{
    fn world(&self) -> Rc<World> { Rc::clone(&self.m_worldObj) }

    pub fn new(world: &Rc<World>) -> ChunkProviderClient
    {
        ChunkProviderClient {
            m_blankChunk: Chunk::newEmptyWithBlocks(world, vec![0; 32768], 0, 0),
            m_chunkMapping: Field::new(JHashMap::new()),
            m_chunks: JArrayList::new(),
            m_worldObj: Rc::clone(world),
        }
    }

    pub fn unloadChunk(&self, chunkX: i32, chunkZ: i32)
    {
        let chunk: Rc<Chunk> = self.provideChunk(chunkX, chunkZ);
        if !chunk.isEmpty()
        {
            chunk.onChunkUnload();
        }
        self.m_chunkMapping.update(|map| map.remove(&ChunkCoordIntPair::new(chunkX, chunkZ)));
        self.m_chunks.removeIdentity(&chunk);
    }
}

impl IChunkProvider for ChunkProviderClient
{
    fn chunkExists(&self, _chunkX: i32, _chunkZ: i32) -> bool { true }

    fn provideChunk(&self, chunkX: i32, chunkZ: i32) -> Rc<Chunk>
    {
        let chunkCoordIntPair: ChunkCoordIntPair = ChunkCoordIntPair::new(chunkX, chunkZ);
        match self.m_chunkMapping.with(|map| map.get(&chunkCoordIntPair))
        {
            Some(chunk) => chunk,
            None => Rc::clone(&self.m_blankChunk),
        }
    }

    fn prepareChunk(&self, chunkX: i32, chunkZ: i32) -> Rc<Chunk>
    {
        let chunkCoordIntPair: ChunkCoordIntPair = ChunkCoordIntPair::new(chunkX, chunkZ);
        let byArray: Vec<i8> = vec![0; 32768];
        let chunk: Rc<Chunk> = Chunk::withBlocks(&self.world(), byArray, chunkX, chunkZ);
        chunk.skylight().m_data.update(|data| data.fill(-1));
        self.m_chunkMapping.update(|map| map.put(chunkCoordIntPair, Rc::clone(&chunk)));
        chunk.m_isChunkLoaded.set(true);
        chunk
    }

    fn populate(&self, _source: &Rc<dyn IChunkProvider>, _chunkX: i32, _chunkZ: i32) {}

    fn saveChunks(&self, _saveEntities: bool, _listener: Option<&dyn IProgressUpdate>) -> bool { true }

    fn unload100OldestChunks(&self) -> bool { false }

    fn canSave(&self) -> bool { false }

    fn releaseChunks(&self) -> Vec<Rc<Chunk>>
    {
        let mut chunks: Vec<Rc<Chunk>> = self.m_chunkMapping.with(JHashMap::values);
        chunks.extend(self.m_chunks.toVec());
        self.m_chunkMapping.update(JHashMap::clear);
        self.m_chunks.clear();
        chunks
    }

    fn makeString(&self) -> String { format!("MultiplayerChunkCache: {}", self.m_chunkMapping.with(JHashMap::size)) }
}
