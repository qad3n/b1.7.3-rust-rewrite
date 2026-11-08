use super::{CHUNK, ChunkClass};

pub const EMPTY_CHUNK: ChunkClass = ChunkClass {
    m_name: "EmptyChunk",
    m_parent: Some(&CHUNK),
    m_interfaces: &[],
    isAtLocation: |this, x, z| x == this.m_xPosition && z == this.m_zPosition,
    getHeightValue: |_, _, _| 0,
    populateLight: |_| {},
    generateHeightMap: |_| {},
    populateHeightMap: |_| {},
    populateBlockLight: |_| {},
    getBlockID: |_, _, _, _| 0,
    setBlockIDWithMetadata: |_, _, _, _, _, _| true,
    setBlockID: |_, _, _, _, _| true,
    getBlockMetadata: |_, _, _, _| 0,
    setBlockMetadata: |_, _, _, _, _| {},
    getSavedLightValue: |_, _, _, _, _| 0,
    setLightValue: |_, _, _, _, _, _| {},
    getBlockLightValue: |_, _, _, _, _| 0,
    addEntity: |_, _| {},
    removeEntity: |_, _| {},
    removeEntityAtIndex: |_, _, _| {},
    canBlockSeeTheSky: |_, _, _, _| false,
    getChunkBlockTileEntity: |_, _, _, _| None,
    addTileEntity: |_, _| {},
    setChunkBlockTileEntity: |_, _, _, _, _| {},
    removeChunkBlockTileEntity: |_, _, _, _| {},
    onChunkLoad: |_| {},
    onChunkUnload: |_| {},
    setChunkModified: |_| {},
    getEntitiesWithinAABBForEntity: |_, _, _, _| {},
    getEntitiesOfTypeWithinAAAB: |_, _, _, _| {},
    needsSaving: |_, _| false,
    setChunkData: |_, _, minX, minY, minZ, maxX, maxY, maxZ, _| {
        let n: i32 = maxX - minX;
        let n2: i32 = maxY - minY;
        let n3: i32 = maxZ - minZ;
        let n4: i32 = n * n2 * n3;
        n4 + n4 / 2 * 3
    },
    getChunkData: |_, data, minX, minY, minZ, maxX, maxY, maxZ, size| {
        let n: i32 = maxX - minX;
        let n2: i32 = maxY - minY;
        let n3: i32 = maxZ - minZ;
        let n4: i32 = n * n2 * n3;
        let n5: i32 = n4 + n4 / 2 * 3;
        data[size as usize..(size + n5) as usize].fill(0);
        n5
    },
    getRandomForSlime: |this, seed| this.getRandomForSlimeImpl(seed),
    isEmpty: |_| true,
};
