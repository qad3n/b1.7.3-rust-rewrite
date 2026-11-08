use super::LIGHTING_UPDATES_SCHEDULED;
use crate::block::Block;
use crate::world::{Chunk, EnumSkyBlock, MetadataChunkBlock, World};
use std::rc::Rc;

fn outside(x: i32, z: i32) -> bool { x < -32000000 || z < -32000000 || x >= 32000000 || z > 32000000 }

impl World
{
    pub fn canBlockSeeTheSky(&self, x: i32, y: i32, z: i32) -> bool
    {
        self.getChunkFromChunkCoords(x >> 4, z >> 4).canBlockSeeTheSky(x & 15, y, z & 15)
    }

    pub fn getFullBlockLightValue(&self, x: i32, y: i32, z: i32) -> i32
    {
        if y < 0
        {
            return 0;
        }
        let y: i32 = if y >= 128 { 127 } else { y };
        self.getChunkFromChunkCoords(x >> 4, z >> 4).getBlockLightValue(x & 15, y, z & 15, 0)
    }

    pub fn getBlockLightValue(&self, x: i32, y: i32, z: i32) -> i32 { self.getBlockLightValue_do(x, y, z, true) }

    pub fn getBlockLightValue_do(&self, x: i32, y: i32, z: i32, useNeighborLight: bool) -> i32
    {
        if outside(x, z)
        {
            return 15;
        }

        if useNeighborLight
        {
            let blockId: i32 = self.getBlockId(x, y, z);
            if blockId == Block::stairSingle().m_blockID
                || blockId == Block::tilledField().m_blockID
                || blockId == Block::stairCompactCobblestone().m_blockID
                || blockId == Block::stairCompactPlanks().m_blockID
            {
                let mut value: i32 = self.getBlockLightValue_do(x, y + 1, z, false);
                let value2: i32 = self.getBlockLightValue_do(x + 1, y, z, false);
                let value3: i32 = self.getBlockLightValue_do(x - 1, y, z, false);
                let value4: i32 = self.getBlockLightValue_do(x, y, z + 1, false);
                let value5: i32 = self.getBlockLightValue_do(x, y, z - 1, false);
                if value2 > value
                {
                    value = value2;
                }
                if value3 > value
                {
                    value = value3;
                }
                if value4 > value
                {
                    value = value4;
                }
                if value5 > value
                {
                    value = value5;
                }
                return value;
            }
        }

        if y < 0
        {
            return 0;
        }
        let y: i32 = if y >= 128 { 127 } else { y };
        let chunk: Rc<Chunk> = self.getChunkFromChunkCoords(x >> 4, z >> 4);
        chunk.getBlockLightValue(x & 0xF, y, z & 0xF, self.m_skylightSubtracted.get())
    }

    pub fn canExistingBlockSeeTheSky(&self, x: i32, y: i32, z: i32) -> bool
    {
        if outside(x, z) || y < 0
        {
            return false;
        }
        if y >= 128
        {
            return true;
        }
        if !self.chunkExists(x >> 4, z >> 4)
        {
            return false;
        }
        self.getChunkFromChunkCoords(x >> 4, z >> 4).canBlockSeeTheSky(x & 15, y, z & 15)
    }

    pub fn getHeightValue(&self, x: i32, z: i32) -> i32
    {
        if outside(x, z) || !self.chunkExists(x >> 4, z >> 4)
        {
            return 0;
        }
        self.getChunkFromChunkCoords(x >> 4, z >> 4).getHeightValue(x & 15, z & 15)
    }

    pub fn neighborLightPropagationChanged(&self, kind: EnumSkyBlock, x: i32, y: i32, z: i32, light: i32)
    {
        let mut light: i32 = light;
        if self.worldProvider().m_hasNoSky.get() && kind == EnumSkyBlock::Sky
        {
            return;
        }
        if !self.blockExists(x, y, z)
        {
            return;
        }

        if kind == EnumSkyBlock::Sky
        {
            if self.canExistingBlockSeeTheSky(x, y, z)
            {
                light = 15;
            }
        }
        else if kind == EnumSkyBlock::Block
        {
            let blockId: i32 = self.getBlockId(x, y, z);
            if Block::lightValue(blockId) > light
            {
                light = Block::lightValue(blockId);
            }
        }

        if self.getSavedLightValue(kind, x, y, z) != light
        {
            self.scheduleLightingUpdate(kind, x, y, z, x, y, z);
        }
    }

    pub fn getSavedLightValue(&self, kind: EnumSkyBlock, x: i32, y: i32, z: i32) -> i32
    {
        let mut y: i32 = y;
        if y < 0
        {
            y = 0;
        }
        if y >= 128
        {
            y = 127;
        }
        if y >= 0 && y < 128 && !outside(x, z)
        {
            let i: i32 = x >> 4;
            let j: i32 = z >> 4;
            if !self.chunkExists(i, j)
            {
                return 0;
            }
            return self.getChunkFromChunkCoords(i, j).getSavedLightValue(kind, x & 15, y, z & 15);
        }
        kind.defaultValue()
    }

    pub fn setLightValue(&self, kind: EnumSkyBlock, x: i32, y: i32, z: i32, light: i32)
    {
        if outside(x, z) || y < 0 || y >= 128 || !self.chunkExists(x >> 4, z >> 4)
        {
            return;
        }
        self.getChunkFromChunkCoords(x >> 4, z >> 4).setLightValue(kind, x & 15, y, z & 15, light);
        let mut i: i32 = 0;
        while i < self.m_worldAccesses.size()
        {
            self.m_worldAccesses.get(i).markBlockAndNeighborsNeedsUpdate(x, y, z);
            i += 1;
        }
    }

    pub fn getBrightness(&self, x: i32, y: i32, z: i32, blockLight: i32) -> f32
    {
        let mut i: i32 = self.getBlockLightValue(x, y, z);
        if i < blockLight
        {
            i = blockLight;
        }
        self.worldProvider().brightness(i)
    }

    pub fn getLightBrightness(&self, x: i32, y: i32, z: i32) -> f32
    {
        self.worldProvider().brightness(self.getBlockLightValue(x, y, z))
    }

    pub fn isDaytime(&self) -> bool { self.m_skylightSubtracted.get() < 4 }

    pub fn updatingLighting(&self) -> bool
    {
        if self.m_lightingUpdatesCounter.get() >= 50
        {
            return false;
        }

        self.m_lightingUpdatesCounter.set(self.m_lightingUpdatesCounter.get() + 1);
        let result: Result<bool, jvm::throwable::Throwable> =
            jvm::throwable::catchThrowable(&jvm::throwable::THROWABLE, || {
                let mut i: i32 = 500;
                while self.m_lightingToUpdate.size() > 0
                {
                    i -= 1;
                    if i <= 0
                    {
                        return true;
                    }
                    let last: Rc<MetadataChunkBlock> =
                        self.m_lightingToUpdate.removeAt(self.m_lightingToUpdate.size() - 1);
                    last.run(self);
                }
                false
            });
        self.m_lightingUpdatesCounter.set(self.m_lightingUpdatesCounter.get() - 1);
        match result
        {
            Ok(value) => value,
            Err(throwable) => jvm::throwable::rethrow(throwable),
        }
    }

    pub fn scheduleLightingUpdate(
        &self,
        kind: EnumSkyBlock,
        minX: i32,
        minY: i32,
        minZ: i32,
        maxX: i32,
        maxY: i32,
        maxZ: i32,
    )
    {
        self.scheduleLightingUpdate_do(kind, minX, minY, minZ, maxX, maxY, maxZ, true);
    }

    pub fn scheduleLightingUpdate_do(
        &self,
        kind: EnumSkyBlock,
        minX: i32,
        minY: i32,
        minZ: i32,
        maxX: i32,
        maxY: i32,
        maxZ: i32,
        expand: bool,
    )
    {
        if self.worldProvider().m_hasNoSky.get() && kind == EnumSkyBlock::Sky
        {
            return;
        }

        LIGHTING_UPDATES_SCHEDULED.with(|count| count.set(count.get() + 1));
        let result: Result<(), jvm::throwable::Throwable> =
            jvm::throwable::catchThrowable(&jvm::throwable::THROWABLE, || {
                if LIGHTING_UPDATES_SCHEDULED.with(|count| count.get()) == 50
                {
                    return;
                }

                let i: i32 = (maxX + minX) / 2;
                let j: i32 = (maxZ + minZ) / 2;
                if !self.blockExists(i, 64, j) || self.getChunkFromBlockCoords(i, j).isEmpty()
                {
                    return;
                }

                let k: i32 = self.m_lightingToUpdate.size();
                if expand
                {
                    let mut l: i32 = 5;
                    if l > k
                    {
                        l = k;
                    }
                    for i1 in 0..l
                    {
                        let block: Rc<MetadataChunkBlock> =
                            self.m_lightingToUpdate.get(self.m_lightingToUpdate.size() - i1 - 1);
                        if block.m_type == kind && block.expand(minX, minY, minZ, maxX, maxY, maxZ)
                        {
                            return;
                        }
                    }
                }

                self.m_lightingToUpdate.add(Rc::new(MetadataChunkBlock::new(kind, minX, minY, minZ, maxX, maxY, maxZ)));
                let j1: i32 = 1000000;
                if self.m_lightingToUpdate.size() > 1000000
                {
                    jvm::system::println(&format!("More than {j1} updates, aborting lighting updates"));
                    self.m_lightingToUpdate.clear();
                }
            });
        LIGHTING_UPDATES_SCHEDULED.with(|count| count.set(count.get() - 1));
        if let Err(throwable) = result
        {
            jvm::throwable::rethrow(throwable);
        }
    }

    pub fn calculateInitialSkylight(&self)
    {
        let i: i32 = self.calculateSkylightSubtracted(1.0);
        if i != self.m_skylightSubtracted.get()
        {
            self.m_skylightSubtracted.set(i);
        }
    }
}
