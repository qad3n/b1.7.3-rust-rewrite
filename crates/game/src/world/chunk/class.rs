use super::{entities, light};
use crate::entity::Entity;
use crate::math::AxisAlignedBB;
use crate::tileentity::TileEntity;
use crate::world::{Chunk, EnumSkyBlock};
use jvm::array_list::JArrayList;
use jvm::random::Random;
use std::rc::Rc;

crate::vtable! {
    ChunkClass for Chunk {
        fn addEntity(entity: &Rc<Entity>);
        fn addTileEntity(blockEntity: &Rc<TileEntity>);
        fn canBlockSeeTheSky(localX: i32, y: i32, localZ: i32) -> bool;
        fn generateHeightMap();
        fn getBlockID(localX: i32, y: i32, localZ: i32) -> i32;
        fn getBlockLightValue(localX: i32, y: i32, localZ: i32, ambientDarkness: i32) -> i32;
        fn getBlockMetadata(localX: i32, y: i32, localZ: i32) -> i32;
        fn getChunkBlockTileEntity(localX: i32, y: i32, localZ: i32) -> Option<Rc<TileEntity>>;
        fn getEntitiesOfTypeWithinAAAB(kind: &str, bounds: &AxisAlignedBB, entities: &mut Vec<Rc<Entity>>);
        fn getEntitiesWithinAABBForEntity(exclude: Option<&Rc<Entity>>, bounds: &AxisAlignedBB, entities: &JArrayList<Rc<Entity>>);
        fn getHeightValue(localX: i32, localZ: i32) -> i32;
        fn getRandomForSlime(seed: i64) -> Random;
        fn getSavedLightValue(kind: EnumSkyBlock, localX: i32, y: i32, localZ: i32) -> i32;
        fn isAtLocation(chunkX: i32, chunkZ: i32) -> bool;
        fn isEmpty() -> bool;
        fn needsSaving(saveEntities: bool) -> bool;
        fn onChunkLoad();
        fn onChunkUnload();
        fn populateBlockLight();
        fn populateHeightMap();
        fn populateLight();
        fn removeChunkBlockTileEntity(localX: i32, y: i32, localZ: i32);
        fn removeEntity(entity: &Rc<Entity>);
        fn removeEntityAtIndex(entity: &Rc<Entity>, chunkY: i32);
        fn setBlockID(localX: i32, y: i32, localZ: i32, block: i32) -> bool;
        fn setBlockIDWithMetadata(localX: i32, y: i32, localZ: i32, block: i32, metadata: i32) -> bool;
        fn setBlockMetadata(localX: i32, y: i32, localZ: i32, metadata: i32);
        fn setChunkBlockTileEntity(localX: i32, y: i32, localZ: i32, blockEntity: &Rc<TileEntity>);
        fn setChunkData(data: &[i8], minX: i32, minY: i32, minZ: i32, maxX: i32, maxY: i32, maxZ: i32, size: i32) -> i32;
        fn getChunkData(data: &mut [i8], minX: i32, minY: i32, minZ: i32, maxX: i32, maxY: i32, maxZ: i32, size: i32) -> i32;
        fn setChunkModified();
        fn setLightValue(kind: EnumSkyBlock, localX: i32, y: i32, localZ: i32, light: i32);
    }
}

pub const CHUNK: ChunkClass = ChunkClass {
    m_name: "Chunk",
    m_parent: None,
    m_interfaces: &[],
    addEntity: entities::addEntity,
    addTileEntity: entities::addTileEntity,
    canBlockSeeTheSky: |this, x, y, z| y >= this.height(z << 4 | x),
    generateHeightMap: light::generateHeightMap,
    getBlockID: |this, x, y, z| this.rawBlock(x << 11 | z << 7 | y) as i32 & 0xFF,
    getBlockLightValue: light::getBlockLightValue,
    getBlockMetadata: |this, x, y, z| this.data().getNibble(x, y, z),
    getChunkBlockTileEntity: entities::getChunkBlockTileEntity,
    getEntitiesOfTypeWithinAAAB: entities::getEntitiesOfTypeWithinAAAB,
    getEntitiesWithinAABBForEntity: entities::getEntitiesWithinAABBForEntity,
    getHeightValue: |this, x, z| this.height(z << 4 | x),
    getRandomForSlime: |this, seed| this.getRandomForSlimeImpl(seed),
    getSavedLightValue: light::getSavedLightValue,
    isAtLocation: |this, x, z| x == this.m_xPosition && z == this.m_zPosition,
    isEmpty: |_| false,
    needsSaving: entities::needsSaving,
    onChunkLoad: entities::onChunkLoad,
    onChunkUnload: entities::onChunkUnload,
    populateBlockLight: |_| {},
    populateHeightMap: light::populateHeightMap,
    populateLight: |_| {},
    removeChunkBlockTileEntity: entities::removeChunkBlockTileEntity,
    removeEntity: |this, entity| this.removeEntityAtIndex(entity, entity.m_chunkCoordY.get()),
    removeEntityAtIndex: entities::removeEntityAtIndex,
    setBlockID: light::setBlockID,
    setBlockIDWithMetadata: light::setBlockIDWithMetadata,
    setBlockMetadata: |this, x, y, z, metadata| {
        this.m_isModified.set(true);
        this.data().setNibble(x, y, z, metadata);
    },
    setChunkBlockTileEntity: entities::setChunkBlockTileEntity,
    setChunkData: light::setChunkData,
    getChunkData: light::getChunkData,
    setChunkModified: |this| this.m_isModified.set(true),
    setLightValue: light::setLightValue,
};
