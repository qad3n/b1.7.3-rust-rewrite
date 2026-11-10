mod class;
mod empty_chunk;
mod entities;
mod light;

use crate::tileentity::TileEntity;
use crate::world::World;
pub use class::{CHUNK, ChunkClass};
use common::world::{ChunkPosition, NibbleArray};
pub use empty_chunk::EMPTY_CHUNK;
use jvm::array_list::JArrayList;
use jvm::field::Field;
use jvm::hash_map::JHashMap;
use jvm::random::Random;
use jvm::throwable::OrNpe;
use std::cell::Cell;
use std::rc::Rc;

// block columns in a chunk
pub const COLUMNS: i32 = 256;

thread_local! {
    pub static IS_LIT: Cell<bool> = const { Cell::new(false) };
}

pub struct Chunk
{
    pub m_class: &'static ChunkClass,
    pub m_blocks: Field<Vec<i8>>,
    // high nibbles of 12-bit block ids, allocated once a mod block above 255 is placed
    pub m_addBlocks: Field<Option<Vec<u8>>>,
    pub m_isChunkLoaded: Cell<bool>,
    pub m_worldObj: Field<Option<Rc<World>>>,
    pub m_data: Field<Option<Rc<NibbleArray>>>,
    pub m_skylightMap: Field<Option<Rc<NibbleArray>>>,
    pub m_blocklightMap: Field<Option<Rc<NibbleArray>>>,
    // rewrite: wider than a byte so a column of a taller dimension can reach its top
    pub m_heightMap: Field<Vec<i16>>,
    pub m_lowestBlockHeight: Cell<i32>,
    pub m_xPosition: i32,
    pub m_zPosition: i32,
    pub m_chunkTileEntityMap: Field<JHashMap<ChunkPosition, Rc<TileEntity>>>,
    pub m_entities: [JArrayList<Rc<crate::entity::Entity>>; 8],
    pub m_isTerrainPopulated: Cell<bool>,
    pub m_isModified: Cell<bool>,
    pub m_neverSave: Cell<bool>,
    pub m_hasEntities: Cell<bool>,
    pub m_lastSaveTime: Cell<i64>,
    // rewrite: per mod chunk data, saved under loader:attachments
    pub m_attachments: std::cell::RefCell<common::nbt::attachments::Attachments>,
    // rewrite: blocks per column, the world height when the chunk was made
    pub m_height: i32,
}

impl Chunk
{
    fn alloc(class: &'static ChunkClass, world: &Rc<World>, chunkX: i32, chunkZ: i32) -> Self
    {
        Self {
            m_class: class,
            m_blocks: Field::new(Vec::new()),
            m_addBlocks: Field::new(None),
            m_isChunkLoaded: Cell::new(false),
            m_worldObj: Field::new(Some(Rc::clone(world))),
            m_data: Field::new(None),
            m_skylightMap: Field::new(None),
            m_blocklightMap: Field::new(None),
            m_heightMap: Field::new(vec![0; COLUMNS as usize]),
            m_lowestBlockHeight: Cell::new(0),
            m_xPosition: chunkX,
            m_zPosition: chunkZ,
            m_chunkTileEntityMap: Field::new(JHashMap::new()),
            m_entities: std::array::from_fn(|_| JArrayList::new()),
            m_isTerrainPopulated: Cell::new(false),
            m_isModified: Cell::new(false),
            m_neverSave: Cell::new(false),
            m_hasEntities: Cell::new(false),
            m_lastSaveTime: Cell::new(0),
            m_attachments: std::cell::RefCell::new(common::nbt::attachments::Attachments::new()),
            m_height: world.height(),
        }
    }

    pub fn new(world: &Rc<World>, chunkX: i32, chunkZ: i32) -> Rc<Self>
    {
        Rc::new(Self::alloc(&CHUNK, world, chunkX, chunkZ))
    }

    pub fn withBlocks(world: &Rc<World>, blocks: Vec<i8>, chunkX: i32, chunkZ: i32) -> Rc<Self>
    {
        Rc::new(Self::allocWithBlocks(&CHUNK, world, blocks, chunkX, chunkZ))
    }

    fn allocWithBlocks(class: &'static ChunkClass, world: &Rc<World>, blocks: Vec<i8>, chunkX: i32, chunkZ: i32)
    -> Self
    {
        let chunk: Self = Self::alloc(class, world, chunkX, chunkZ);
        let len: i32 = blocks.len() as i32;
        chunk.m_blocks.set(blocks);
        chunk.m_data.set(Some(Rc::new(NibbleArray::new(len))));
        chunk.m_skylightMap.set(Some(Rc::new(NibbleArray::new(len))));
        chunk.m_blocklightMap.set(Some(Rc::new(NibbleArray::new(len))));
        chunk
    }

    pub fn newEmpty(world: &Rc<World>, chunkX: i32, chunkZ: i32) -> Rc<Self>
    {
        let chunk: Self = Self::alloc(&EMPTY_CHUNK, world, chunkX, chunkZ);
        chunk.m_neverSave.set(true);
        Rc::new(chunk)
    }

    pub fn newEmptyWithBlocks(world: &Rc<World>, blocks: Vec<i8>, chunkX: i32, chunkZ: i32) -> Rc<Self>
    {
        let chunk: Self = Self::allocWithBlocks(&EMPTY_CHUNK, world, blocks, chunkX, chunkZ);
        chunk.m_neverSave.set(true);
        Rc::new(chunk)
    }

    pub fn world(&self) -> Rc<World> { self.m_worldObj.get().orNpe() }

    pub fn data(&self) -> Rc<NibbleArray> { self.m_data.get().orNpe() }

    pub fn skylight(&self) -> Rc<NibbleArray> { self.m_skylightMap.get().orNpe() }

    pub fn blocklight(&self) -> Rc<NibbleArray> { self.m_blocklightMap.get().orNpe() }

    pub fn metaAt(&self, x: i32, y: i32, z: i32) -> i32 { nibble(&self.m_data, x, y, z) }

    pub fn skyAt(&self, x: i32, y: i32, z: i32) -> i32 { nibble(&self.m_skylightMap, x, y, z) }

    pub fn blockLightAt(&self, x: i32, y: i32, z: i32) -> i32 { nibble(&self.m_blocklightMap, x, y, z) }

    pub fn rawBlock(&self, index: i32) -> i8 { self.m_blocks.with(|blocks| blocks[index as usize]) }

    pub fn blockAt(&self, index: i32) -> i32
    {
        let low: i32 = self.rawBlock(index) as i32 & 0xFF;
        let high: i32 = self
            .m_addBlocks
            .with(|add| add.as_ref().map_or(0, |add| (add[(index >> 1) as usize] >> ((index & 1) * 4)) as i32 & 0xF));
        low | high << 8
    }

    // chunk packets carry only low id bytes so a resent region drops high nibbles a chunk extension then restores
    pub fn clearHigh(&self, start: i32, len: i32)
    {
        if self.m_addBlocks.with(Option::is_none)
        {
            return;
        }

        for index in start..start + len
        {
            self.setBlockHigh(index, 0);
        }
    }

    pub fn setBlockHigh(&self, index: i32, block: i32)
    {
        let high: u8 = (block >> 8) as u8 & 0xF;
        if high == 0 && self.m_addBlocks.with(Option::is_none)
        {
            return;
        }
        let size: usize = self.m_blocks.with(Vec::len) / 2;
        self.m_addBlocks.update(|add| {
            let add: &mut Vec<u8> = add.get_or_insert_with(|| vec![0; size]);
            let slot: &mut u8 = &mut add[(index >> 1) as usize];
            let shift: i32 = (index & 1) * 4;
            *slot = (*slot & !(0xF << shift)) | (high << shift);
        });
    }

    fn heightAt(&self, index: i32) -> i32 { self.m_heightMap.with(|map| map[index as usize] as i32) }

    pub fn height(&self) -> i32 { self.m_height }

    pub fn volume(&self) -> i32 { COLUMNS * self.m_height }

    // McRegion order, x << 11 | z << 7 | y at the vanilla height
    pub fn index(&self, x: i32, y: i32, z: i32) -> i32 { (x << 4 | z) * self.m_height + y }

    pub fn fill(&self) { self.m_blocks.update(|blocks| crate::world::ChunkBlockMap::fill(blocks)); }

    pub fn getRandomForSlimeImpl(&self, seed: i64) -> Random
    {
        let x: i32 = self.m_xPosition;
        let z: i32 = self.m_zPosition;
        let sum: i64 = self
            .world()
            .getRandomSeed()
            .wrapping_add(x.wrapping_mul(x).wrapping_mul(4987142) as i64)
            .wrapping_add(x.wrapping_mul(5947611) as i64)
            .wrapping_add((z.wrapping_mul(z) as i64).wrapping_mul(4392871))
            .wrapping_add(z.wrapping_mul(389711) as i64);
        Random::withSeed(sum ^ seed)
    }
}

fn nibble(array: &Field<Option<Rc<NibbleArray>>>, x: i32, y: i32, z: i32) -> i32
{
    array.with(|array| array.as_ref().orNpe().getNibble(x, y, z))
}
