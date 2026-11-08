use crate::block::Block;
use crate::entity::Entity;
use crate::math::AxisAlignedBB;
use crate::math::math_helper::floor_double;
use crate::tileentity::TileEntity;
use crate::world::{Chunk, World};
use common::world::ChunkPosition;
use jvm::array_list::JArrayList;
use jvm::hash_map::JHashMap;
use jvm::throwable::OrNpe;
use std::rc::Rc;

pub fn addEntity(this: &Rc<Chunk>, entity: &Rc<Entity>)
{
    this.m_hasEntities.set(true);
    let i: i32 = floor_double(entity.posX() / 16.0);
    let j: i32 = floor_double(entity.posZ() / 16.0);
    if i != this.m_xPosition || j != this.m_zPosition
    {
        jvm::system::println(&format!("Wrong location! {}", crate::entity::EntityList::objectString(entity)));
        jvm::system::dumpStack();
    }

    let mut k: i32 = floor_double(entity.posY() / 16.0);
    if k < 0
    {
        k = 0;
    }
    if k >= this.m_entities.len() as i32
    {
        k = this.m_entities.len() as i32 - 1;
    }
    entity.m_addedToChunk.set(true);
    entity.m_chunkCoordX.set(this.m_xPosition);
    entity.m_chunkCoordY.set(k);
    entity.m_chunkCoordZ.set(this.m_zPosition);
    this.m_entities[k as usize].add(Rc::clone(entity));
}

pub fn removeEntityAtIndex(this: &Rc<Chunk>, entity: &Rc<Entity>, chunkY: i32)
{
    let mut chunkY: i32 = chunkY;
    if chunkY < 0
    {
        chunkY = 0;
    }
    if chunkY >= this.m_entities.len() as i32
    {
        chunkY = this.m_entities.len() as i32 - 1;
    }
    this.m_entities[chunkY as usize].remove(entity);
}

fn tileAt(this: &Chunk, position: &ChunkPosition) -> Option<Rc<TileEntity>>
{
    this.m_chunkTileEntityMap.with(|map| map.get(position))
}

pub fn getChunkBlockTileEntity(this: &Rc<Chunk>, localX: i32, y: i32, localZ: i32) -> Option<Rc<TileEntity>>
{
    let chunkposition: ChunkPosition = ChunkPosition::new(localX, y, localZ);
    let mut tileentity: Option<Rc<TileEntity>> = tileAt(this, &chunkposition);
    if tileentity.is_none()
    {
        let i: i32 = this.getBlockID(localX, y, localZ);
        if !Block::isBlockContainer(i)
        {
            return None;
        }
        let blockcontainer: Rc<Block> = Block::byId(i).orNpe();
        blockcontainer.onBlockAdded(&this.world(), this.m_xPosition * 16 + localX, y, this.m_zPosition * 16 + localZ);
        tileentity = tileAt(this, &chunkposition);
    }

    if let Some(tile) = &tileentity
        && tile.isRemoved()
    {
        this.m_chunkTileEntityMap.update(|map| map.remove(&chunkposition));
        return None;
    }
    tileentity
}

pub fn addTileEntity(this: &Rc<Chunk>, blockEntity: &Rc<TileEntity>)
{
    let i: i32 = blockEntity.m_xCoord.get() - this.m_xPosition * 16;
    let j: i32 = blockEntity.m_yCoord.get();
    let k: i32 = blockEntity.m_zCoord.get() - this.m_zPosition * 16;
    this.setChunkBlockTileEntity(i, j, k, blockEntity);
    if this.m_isChunkLoaded.get()
    {
        this.world().m_loadedTileEntityList.add(Rc::clone(blockEntity));
    }
}

pub fn setChunkBlockTileEntity(this: &Rc<Chunk>, localX: i32, y: i32, localZ: i32, blockEntity: &Rc<TileEntity>)
{
    let chunkPosition: ChunkPosition = ChunkPosition::new(localX, y, localZ);
    blockEntity.m_worldObj.set(this.m_worldObj.get());
    blockEntity.m_xCoord.set(this.m_xPosition * 16 + localX);
    blockEntity.m_yCoord.set(y);
    blockEntity.m_zCoord.set(this.m_zPosition * 16 + localZ);
    let id: i32 = this.getBlockID(localX, y, localZ);
    if id == 0 || !Block::byId(id).is_some_and(|block| block.instanceOf("BlockContainer"))
    {
        jvm::system::println("Attempted to place a tile entity where there was no entity tile!");
        return;
    }
    blockEntity.cancelRemoval();
    this.m_chunkTileEntityMap.update(|map| map.put(chunkPosition, Rc::clone(blockEntity)));
}

pub fn removeChunkBlockTileEntity(this: &Rc<Chunk>, localX: i32, y: i32, localZ: i32)
{
    let chunkposition: ChunkPosition = ChunkPosition::new(localX, y, localZ);
    if this.m_isChunkLoaded.get()
        && let Some(tileentity) = this.m_chunkTileEntityMap.update(|map| map.remove(&chunkposition))
    {
        tileentity.markRemoved();
    }
}

pub fn onChunkLoad(this: &Rc<Chunk>)
{
    this.m_isChunkLoaded.set(true);
    let world: Rc<World> = this.world();
    world.loadBlockEntities(&this.m_chunkTileEntityMap.with(JHashMap::values));
    for list in &this.m_entities
    {
        world.loadEntities(&list.toVec());
    }
}

pub fn onChunkUnload(this: &Rc<Chunk>)
{
    this.m_isChunkLoaded.set(false);
    for tileentity in this.m_chunkTileEntityMap.with(JHashMap::values)
    {
        tileentity.markRemoved();
    }
    let world: Rc<World> = this.world();
    for list in &this.m_entities
    {
        world.unloadEntities(&list.toVec());
    }
}

fn chunkRange(this: &Chunk, bounds: &AxisAlignedBB) -> (i32, i32)
{
    let mut i: i32 = floor_double((bounds.minY() - 2.0) / 16.0);
    let mut j: i32 = floor_double((bounds.maxY() + 2.0) / 16.0);
    if i < 0
    {
        i = 0;
    }
    if j >= this.m_entities.len() as i32
    {
        j = this.m_entities.len() as i32 - 1;
    }
    (i, j)
}

pub fn getEntitiesWithinAABBForEntity(
    this: &Rc<Chunk>,
    exclude: Option<&Rc<Entity>>,
    bounds: &AxisAlignedBB,
    entities: &JArrayList<Rc<Entity>>,
)
{
    let (i, j): (i32, i32) = chunkRange(this, bounds);
    for k in i..=j
    {
        let list: &JArrayList<Rc<Entity>> = &this.m_entities[k as usize];
        let mut l: i32 = 0;
        while l < list.size()
        {
            let entity: Rc<Entity> = list.get(l);
            let excluded: bool = exclude.is_some_and(|exclude| Rc::ptr_eq(exclude, &entity));
            if !excluded && entity.m_boundingBox.intersectsWith(bounds)
            {
                entities.add(entity);
            }
            l += 1;
        }
    }
}

pub fn getEntitiesOfTypeWithinAAAB(this: &Rc<Chunk>, kind: &str, bounds: &AxisAlignedBB, entities: &mut Vec<Rc<Entity>>)
{
    let (i, j): (i32, i32) = chunkRange(this, bounds);
    for k in i..=j
    {
        let list: &JArrayList<Rc<Entity>> = &this.m_entities[k as usize];
        let mut l: i32 = 0;
        while l < list.size()
        {
            let entity: Rc<Entity> = list.get(l);
            if entity.instanceOf(kind) && entity.m_boundingBox.intersectsWith(bounds)
            {
                entities.push(entity);
            }
            l += 1;
        }
    }
}

pub fn needsSaving(this: &Rc<Chunk>, saveEntities: bool) -> bool
{
    if this.m_neverSave.get()
    {
        return false;
    }
    let time: i64 = this.world().getWorldTime();
    if saveEntities
    {
        if this.m_hasEntities.get() && time != this.m_lastSaveTime.get()
        {
            return true;
        }
    }
    else if this.m_hasEntities.get() && time >= this.m_lastSaveTime.get() + 600
    {
        return true;
    }
    this.m_isModified.get()
}
