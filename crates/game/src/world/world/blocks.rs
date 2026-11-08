use crate::block::{Block, Material};
use crate::math::{AxisAlignedBB, math_helper};
use crate::tileentity::TileEntity;
use crate::world::{Chunk, World};
use jvm::throwable::OrNpe;
use std::rc::Rc;

fn outside(x: i32, z: i32) -> bool { x < -32000000 || z < -32000000 || x >= 32000000 || z > 32000000 }

pub fn setBlockAndMetadata(this: &Rc<World>, x: i32, y: i32, z: i32, block: i32, metadata: i32) -> bool
{
    if outside(x, z) || y < 0 || y >= 128
    {
        return false;
    }
    let chunk: Rc<Chunk> = this.getChunkFromChunkCoords(x >> 4, z >> 4);
    chunk.setBlockIDWithMetadata(x & 15, y, z & 15, block, metadata)
}

pub fn setBlock(this: &Rc<World>, x: i32, y: i32, z: i32, block: i32) -> bool
{
    if outside(x, z) || y < 0 || y >= 128
    {
        return false;
    }
    let chunk: Rc<Chunk> = this.getChunkFromChunkCoords(x >> 4, z >> 4);
    chunk.setBlockID(x & 15, y, z & 15, block)
}

pub fn setBlockMetadata(this: &Rc<World>, x: i32, y: i32, z: i32, metadata: i32) -> bool
{
    if outside(x, z) || y < 0 || y >= 128
    {
        return false;
    }
    let chunk: Rc<Chunk> = this.getChunkFromChunkCoords(x >> 4, z >> 4);
    chunk.setBlockMetadata(x & 15, y, z & 15, metadata);
    true
}

impl World
{
    pub fn getBlockId(&self, x: i32, y: i32, z: i32) -> i32
    {
        if outside(x, z) || y < 0 || y >= 128
        {
            return 0;
        }
        self.getChunkFromChunkCoords(x >> 4, z >> 4).getBlockID(x & 0xF, y, z & 0xF)
    }

    pub fn isAirBlock(&self, x: i32, y: i32, z: i32) -> bool { self.getBlockId(x, y, z) == 0 }

    pub fn blockExists(&self, x: i32, y: i32, z: i32) -> bool
    {
        if y >= 0 && y < 128
        {
            self.chunkExists(x >> 4, z >> 4)
        }
        else
        {
            false
        }
    }

    pub fn doChunksNearChunkExist(&self, x: i32, y: i32, z: i32, range: i32) -> bool
    {
        self.checkChunksExist(x - range, y - range, z - range, x + range, y + range, z + range)
    }

    pub fn checkChunksExist(&self, minX: i32, minY: i32, minZ: i32, maxX: i32, maxY: i32, maxZ: i32) -> bool
    {
        if maxY < 0 || minY >= 128
        {
            return false;
        }
        let minX: i32 = minX >> 4;
        let minZ: i32 = minZ >> 4;
        let maxX: i32 = maxX >> 4;
        let maxZ: i32 = maxZ >> 4;
        for i in minX..=maxX
        {
            for j in minZ..=maxZ
            {
                if !self.chunkExists(i, j)
                {
                    return false;
                }
            }
        }
        true
    }

    pub(crate) fn chunkExists(&self, chunkX: i32, chunkZ: i32) -> bool
    {
        self.chunkProvider().chunkExists(chunkX, chunkZ)
    }

    pub fn getChunkFromBlockCoords(&self, x: i32, z: i32) -> Rc<Chunk> { self.getChunkFromChunkCoords(x >> 4, z >> 4) }

    pub fn getChunkFromChunkCoords(&self, chunkX: i32, chunkZ: i32) -> Rc<Chunk>
    {
        self.chunkProvider().provideChunk(chunkX, chunkZ)
    }

    pub fn getBlockMaterial(&self, x: i32, y: i32, z: i32) -> Rc<Material>
    {
        let blockId: i32 = self.getBlockId(x, y, z);
        if blockId == 0
        {
            return Material::air();
        }
        Rc::clone(&Block::byId(blockId).orNpe().m_blockMaterial)
    }

    pub fn getBlockMetadata(&self, x: i32, y: i32, z: i32) -> i32
    {
        if outside(x, z) || y < 0 || y >= 128
        {
            return 0;
        }
        let chunk: Rc<Chunk> = self.getChunkFromChunkCoords(x >> 4, z >> 4);
        chunk.getBlockMetadata(x & 15, y, z & 15)
    }

    pub fn setBlockMetadataWithNotify(self: &Rc<Self>, x: i32, y: i32, z: i32, metadata: i32)
    {
        if self.setBlockMetadata(x, y, z, metadata)
        {
            let i: i32 = self.getBlockId(x, y, z);
            if Block::updateClients(i & 0xFF)
            {
                self.notifyBlockChange(x, y, z, i);
            }
            else
            {
                self.notifyBlocksOfNeighborChange(x, y, z, i);
            }
        }
    }

    pub fn setBlockWithNotify(self: &Rc<Self>, x: i32, y: i32, z: i32, block: i32) -> bool
    {
        if self.setBlock(x, y, z, block)
        {
            self.notifyBlockChange(x, y, z, block);
            return true;
        }
        false
    }

    pub fn setBlockAndMetadataWithNotify(self: &Rc<Self>, x: i32, y: i32, z: i32, block: i32, metadata: i32) -> bool
    {
        if self.setBlockAndMetadata(x, y, z, block, metadata)
        {
            self.notifyBlockChange(x, y, z, block);
            return true;
        }
        false
    }

    pub fn markBlockNeedsUpdate(&self, x: i32, y: i32, z: i32)
    {
        let mut i: i32 = 0;
        while i < self.m_worldAccesses.size()
        {
            self.m_worldAccesses.get(i).markBlockAndNeighborsNeedsUpdate(x, y, z);
            i += 1;
        }
    }

    pub fn notifyBlockChange(self: &Rc<Self>, x: i32, y: i32, z: i32, block: i32)
    {
        self.markBlockNeedsUpdate(x, y, z);
        self.notifyBlocksOfNeighborChange(x, y, z, block);
    }

    pub fn markBlocksDirtyVertical(&self, x: i32, z: i32, newHeight: i32, oldHeight: i32)
    {
        let (mut newHeight, mut oldHeight): (i32, i32) = (newHeight, oldHeight);
        if newHeight > oldHeight
        {
            let i: i32 = oldHeight;
            oldHeight = newHeight;
            newHeight = i;
        }
        self.markBlocksDirty(x, newHeight, z, x, oldHeight, z);
    }

    pub fn markBlockAsNeedsUpdate(&self, x: i32, y: i32, z: i32)
    {
        let mut i: i32 = 0;
        while i < self.m_worldAccesses.size()
        {
            self.m_worldAccesses.get(i).markBlockRangeNeedsUpdate(x, y, z, x, y, z);
            i += 1;
        }
    }

    pub fn markBlocksDirty(&self, minX: i32, minY: i32, minZ: i32, maxX: i32, maxY: i32, maxZ: i32)
    {
        let mut i: i32 = 0;
        while i < self.m_worldAccesses.size()
        {
            self.m_worldAccesses.get(i).markBlockRangeNeedsUpdate(minX, minY, minZ, maxX, maxY, maxZ);
            i += 1;
        }
    }

    pub fn notifyBlocksOfNeighborChange(self: &Rc<Self>, x: i32, y: i32, z: i32, block: i32)
    {
        self.notifyBlockOfNeighborChange(x - 1, y, z, block);
        self.notifyBlockOfNeighborChange(x + 1, y, z, block);
        self.notifyBlockOfNeighborChange(x, y - 1, z, block);
        self.notifyBlockOfNeighborChange(x, y + 1, z, block);
        self.notifyBlockOfNeighborChange(x, y, z - 1, block);
        self.notifyBlockOfNeighborChange(x, y, z + 1, block);
    }

    fn notifyBlockOfNeighborChange(self: &Rc<Self>, x: i32, y: i32, z: i32, neighborBlock: i32)
    {
        if self.m_editingBlocks.get() || self.m_multiplayerWorld.get()
        {
            return;
        }
        if let Some(block) = Block::byId(self.getBlockId(x, y, z))
        {
            block.onNeighborBlockChange(self, x, y, z, neighborBlock);
        }
    }

    pub fn findTopSolidBlockServer(&self, x: i32, z: i32) -> i32
    {
        let chunk: Rc<Chunk> = self.getChunkFromBlockCoords(x, z);
        let mut i: i32 = 127;
        let x: i32 = x & 15;
        let z: i32 = z & 15;
        while i > 0
        {
            let j: i32 = chunk.getBlockID(x, i, z);
            if j != 0 && Block::byId(j).orNpe().m_blockMaterial.getIsSolid()
            {
                return i + 1;
            }
            i -= 1;
        }
        -1
    }

    pub fn containsNonAir(&self, bounds: &AxisAlignedBB) -> bool
    {
        let mut i: i32 = math_helper::floor_double(bounds.m_minX.get());
        let j: i32 = math_helper::floor_double(bounds.m_maxX.get() + 1.0);
        let mut k: i32 = math_helper::floor_double(bounds.m_minY.get());
        let l: i32 = math_helper::floor_double(bounds.m_maxY.get() + 1.0);
        let mut i1: i32 = math_helper::floor_double(bounds.m_minZ.get());
        let j1: i32 = math_helper::floor_double(bounds.m_maxZ.get() + 1.0);
        if bounds.m_minX.get() < 0.0
        {
            i -= 1;
        }
        if bounds.m_minY.get() < 0.0
        {
            k -= 1;
        }
        if bounds.m_minZ.get() < 0.0
        {
            i1 -= 1;
        }
        for k1 in i..j
        {
            for l1 in k..l
            {
                for i2 in i1..j1
                {
                    if Block::byId(self.getBlockId(k1, l1, i2)).is_some()
                    {
                        return true;
                    }
                }
            }
        }
        false
    }

    pub fn getChunkData(&self, minX: i32, minY: i32, minZ: i32, widthX: i32, height: i32, widthZ: i32) -> Vec<i8>
    {
        let mut abyte: Vec<i8> = vec![0; (widthX * height * widthZ * 5 / 2) as usize];
        let i: i32 = minX >> 4;
        let j: i32 = minZ >> 4;
        let k: i32 = minX + widthX - 1 >> 4;
        let l: i32 = minZ + widthZ - 1 >> 4;
        let mut i1: i32 = 0;
        let mut j1: i32 = minY;
        let mut k1: i32 = minY + height;
        if j1 < 0
        {
            j1 = 0;
        }
        if k1 > 128
        {
            k1 = 128;
        }
        for l1 in i..=k
        {
            let mut i2: i32 = minX - l1 * 16;
            let mut j2: i32 = minX + widthX - l1 * 16;
            if i2 < 0
            {
                i2 = 0;
            }
            if j2 > 16
            {
                j2 = 16;
            }
            for k2 in j..=l
            {
                let mut l2: i32 = minZ - k2 * 16;
                let mut i3: i32 = minZ + widthZ - k2 * 16;
                if l2 < 0
                {
                    l2 = 0;
                }
                if i3 > 16
                {
                    i3 = 16;
                }
                i1 = self.getChunkFromChunkCoords(l1, k2).getChunkData(&mut abyte, i2, j1, l2, j2, k1, i3, i1);
            }
        }
        abyte
    }

    pub fn findTopSolidBlock(&self, x: i32, z: i32) -> i32
    {
        let chunk: Rc<Chunk> = self.getChunkFromBlockCoords(x, z);
        let mut i: i32 = 127;
        let x: i32 = x & 15;
        let z: i32 = z & 15;
        while i > 0
        {
            let j: i32 = chunk.getBlockID(x, i, z);
            let material: Rc<Material> = if j == 0
            {
                Material::air()
            }
            else
            {
                Rc::clone(&Block::byId(j).orNpe().m_blockMaterial)
            };
            if material.getIsSolid() || material.getIsLiquid()
            {
                return i + 1;
            }
            i -= 1;
        }
        -1
    }

    pub fn getBlockTileEntity(&self, x: i32, y: i32, z: i32) -> Option<Rc<TileEntity>>
    {
        self.getChunkFromChunkCoords(x >> 4, z >> 4).getChunkBlockTileEntity(x & 0xF, y, z & 0xF)
    }

    pub fn setBlockTileEntity(&self, x: i32, y: i32, z: i32, blockEntity: &Rc<TileEntity>)
    {
        if blockEntity.isRemoved()
        {
            return;
        }
        if self.m_isTickingBlockEntities.get()
        {
            blockEntity.m_xCoord.set(x);
            blockEntity.m_yCoord.set(y);
            blockEntity.m_zCoord.set(z);
            self.m_pendingBlockEntities.add(Rc::clone(blockEntity));
        }
        else
        {
            self.m_loadedTileEntityList.add(Rc::clone(blockEntity));
            let chunk: Rc<Chunk> = self.getChunkFromChunkCoords(x >> 4, z >> 4);
            chunk.setChunkBlockTileEntity(x & 15, y, z & 15, blockEntity);
        }
    }

    pub fn removeBlockTileEntity(&self, x: i32, y: i32, z: i32)
    {
        let tileentity: Option<Rc<TileEntity>> = self.getBlockTileEntity(x, y, z);
        if let Some(tileentity) = &tileentity
            && self.m_isTickingBlockEntities.get()
        {
            tileentity.markRemoved();
            return;
        }

        if let Some(tileentity) = &tileentity
        {
            self.m_loadedTileEntityList.removeIdentity(tileentity);
        }
        let chunk: Rc<Chunk> = self.getChunkFromChunkCoords(x >> 4, z >> 4);
        chunk.removeChunkBlockTileEntity(x & 15, y, z & 15);
    }

    pub fn isBlockOpaqueCube(&self, x: i32, y: i32, z: i32) -> bool
    {
        Block::byId(self.getBlockId(x, y, z)).is_some_and(|block| block.isOpaqueCube())
    }

    pub fn isBlockNormalCube(&self, x: i32, y: i32, z: i32) -> bool
    {
        match Block::byId(self.getBlockId(x, y, z))
        {
            None => false,
            Some(block) => block.m_blockMaterial.getIsTranslucent() && block.renderAsNormalBlock(),
        }
    }

    pub fn canBlockBePlacedAt(
        self: &Rc<Self>,
        block: i32,
        x: i32,
        y: i32,
        z: i32,
        skipCollisionChecks: bool,
        face: i32,
    ) -> bool
    {
        let i: i32 = self.getBlockId(x, y, z);
        let mut blockx: Option<Rc<Block>> = Block::byId(i);
        let block1x: Rc<Block> = Block::byId(block).orNpe();
        let mut axisalignedbb: Option<Rc<crate::math::AxisAlignedBB>> =
            block1x.getCollisionBoundingBoxFromPool(self, x, y, z);
        if skipCollisionChecks
        {
            axisalignedbb = None;
        }
        if let Some(bb) = &axisalignedbb
            && !self.checkIfAABBIsClear(bb)
        {
            return false;
        }

        if let Some(existing) = &blockx
        {
            let replaceable: [Rc<Block>; 6] = [
                Block::waterMoving(),
                Block::waterStill(),
                Block::lavaMoving(),
                Block::lavaStill(),
                Block::fire(),
                Block::snow(),
            ];
            if replaceable.iter().any(|candidate| Rc::ptr_eq(candidate, existing))
            {
                blockx = None;
            }
        }
        block > 0 && blockx.is_none() && block1x.canPlaceBlockOnSide(self, x, y, z, face)
    }

    pub fn setChunkData(&self, minX: i32, minY: i32, minZ: i32, widthX: i32, height: i32, widthZ: i32, data: &[i8])
    {
        let i: i32 = minX >> 4;
        let j: i32 = minZ >> 4;
        let k: i32 = minX + widthX - 1 >> 4;
        let l: i32 = minZ + widthZ - 1 >> 4;
        let mut i1: i32 = 0;
        let mut j1: i32 = minY;
        let mut k1: i32 = minY + height;
        if j1 < 0
        {
            j1 = 0;
        }
        if k1 > 128
        {
            k1 = 128;
        }
        for l1 in i..=k
        {
            let mut i2: i32 = minX - l1 * 16;
            let mut j2: i32 = minX + widthX - l1 * 16;
            if i2 < 0
            {
                i2 = 0;
            }
            if j2 > 16
            {
                j2 = 16;
            }
            for k2 in j..=l
            {
                let mut l2: i32 = minZ - k2 * 16;
                let mut i3: i32 = minZ + widthZ - k2 * 16;
                if l2 < 0
                {
                    l2 = 0;
                }
                if i3 > 16
                {
                    i3 = 16;
                }
                i1 = self.getChunkFromChunkCoords(l1, k2).setChunkData(data, i2, j1, l2, j2, k1, i3, i1);
                self.markBlocksDirty(l1 * 16 + i2, j1, k2 * 16 + l2, l1 * 16 + j2, k1, k2 * 16 + i3);
            }
        }
    }

    pub fn notifyBlockEntityChanged(&self, x: i32, y: i32, z: i32, blockEntity: &Rc<TileEntity>)
    {
        if self.blockExists(x, y, z)
        {
            self.getChunkFromBlockCoords(x, z).setChunkModified();
        }
        let mut i: i32 = 0;
        while i < self.m_worldAccesses.size()
        {
            self.m_worldAccesses.get(i).doNothingWithTileEntity(x, y, z, blockEntity);
            i += 1;
        }
    }
}

pub fn playNoteAt(this: &Rc<World>, x: i32, y: i32, z: i32, kind: i32, data: i32)
{
    let i: i32 = this.getBlockId(x, y, z);
    if i > 0
    {
        Block::byId(i).orNpe().playBlock(this, x, y, z, kind, data);
    }
}
