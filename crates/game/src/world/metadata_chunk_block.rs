use crate::block::Block;
use crate::world::{Chunk, EnumSkyBlock, World};
use std::cell::Cell;
use std::rc::Rc;

pub struct MetadataChunkBlock
{
    pub m_type: EnumSkyBlock,
    pub m_minX: Cell<i32>,
    pub m_minY: Cell<i32>,
    pub m_minZ: Cell<i32>,
    pub m_maxX: Cell<i32>,
    pub m_maxY: Cell<i32>,
    pub m_maxZ: Cell<i32>,
}

impl MetadataChunkBlock
{
    pub fn new(kind: EnumSkyBlock, minX: i32, minY: i32, minZ: i32, maxX: i32, maxY: i32, maxZ: i32) -> Self
    {
        Self {
            m_type: kind,
            m_minX: Cell::new(minX),
            m_minY: Cell::new(minY),
            m_minZ: Cell::new(minZ),
            m_maxX: Cell::new(maxX),
            m_maxY: Cell::new(maxY),
            m_maxZ: Cell::new(maxZ),
        }
    }

    pub fn run(&self, world: &World)
    {
        let i: i32 = self.m_maxX.get() - self.m_minX.get() + 1;
        let j: i32 = self.m_maxY.get() - self.m_minY.get() + 1;
        let k: i32 = self.m_maxZ.get() - self.m_minZ.get() + 1;
        let l: i32 = i * j * k;
        if l > 32768
        {
            jvm::system::println("Light too large, skipping!");
            return;
        }

        let mut i1: i32 = 0;
        let mut j1: i32 = 0;
        let flag: bool = false;
        let mut flag1: bool = false;
        for k1 in self.m_minX.get()..=self.m_maxX.get()
        {
            for l1 in self.m_minZ.get()..=self.m_maxZ.get()
            {
                let i2: i32 = k1 >> 4;
                let j2: i32 = l1 >> 4;
                let mut flag2: bool;
                if flag && i2 == i1 && j2 == j1
                {
                    flag2 = flag1;
                }
                else
                {
                    flag2 = world.doChunksNearChunkExist(k1, 0, l1, 1);
                    if flag2
                    {
                        let chunk: Rc<Chunk> = world.getChunkFromChunkCoords(k1 >> 4, l1 >> 4);
                        if chunk.isEmpty()
                        {
                            flag2 = false;
                        }
                    }
                    flag1 = flag2;
                    i1 = i2;
                    j1 = j2;
                }

                if !flag2
                {
                    continue;
                }

                if self.m_minY.get() < 0
                {
                    self.m_minY.set(0);
                }
                if self.m_maxY.get() >= 128
                {
                    self.m_maxY.set(127);
                }

                let mut j5: i32 = self.m_minY.get();
                while j5 <= self.m_maxY.get()
                {
                    self.relight(world, k1, j5, l1);
                    j5 += 1;
                }
            }
        }
    }

    fn relight(&self, world: &World, k1: i32, j5: i32, l1: i32)
    {
        let kind: EnumSkyBlock = self.m_type;
        let k2: i32 = world.getSavedLightValue(kind, k1, j5, l1);
        let mut l2: i32;
        let i3: i32 = world.getBlockId(k1, j5, l1);
        let mut j3: i32 = Block::lightOpacity(i3);
        if j3 == 0
        {
            j3 = 1;
        }
        let mut k3: i32 = 0;
        if kind == EnumSkyBlock::Sky
        {
            if world.canExistingBlockSeeTheSky(k1, j5, l1)
            {
                k3 = 15;
            }
        }
        else if kind == EnumSkyBlock::Block
        {
            k3 = Block::lightValue(i3);
        }

        if j3 >= 15 && k3 == 0
        {
            l2 = 0;
        }
        else
        {
            let l3: i32 = world.getSavedLightValue(kind, k1 - 1, j5, l1);
            let i4: i32 = world.getSavedLightValue(kind, k1 + 1, j5, l1);
            let j4: i32 = world.getSavedLightValue(kind, k1, j5 - 1, l1);
            let k4: i32 = world.getSavedLightValue(kind, k1, j5 + 1, l1);
            let l4: i32 = world.getSavedLightValue(kind, k1, j5, l1 - 1);
            let i5: i32 = world.getSavedLightValue(kind, k1, j5, l1 + 1);
            l2 = l3;
            if i4 > l2
            {
                l2 = i4;
            }
            if j4 > l2
            {
                l2 = j4;
            }
            if k4 > l2
            {
                l2 = k4;
            }
            if l4 > l2
            {
                l2 = l4;
            }
            if i5 > l2
            {
                l2 = i5;
            }
            l2 -= j3;
            if l2 < 0
            {
                l2 = 0;
            }
            if k3 > l2
            {
                l2 = k3;
            }
        }

        if k2 != l2
        {
            world.setLightValue(kind, k1, j5, l1, l2);
            let mut k5: i32 = l2 - 1;
            if k5 < 0
            {
                k5 = 0;
            }
            world.neighborLightPropagationChanged(kind, k1 - 1, j5, l1, k5);
            world.neighborLightPropagationChanged(kind, k1, j5 - 1, l1, k5);
            world.neighborLightPropagationChanged(kind, k1, j5, l1 - 1, k5);
            if k1 + 1 >= self.m_maxX.get()
            {
                world.neighborLightPropagationChanged(kind, k1 + 1, j5, l1, k5);
            }
            if j5 + 1 >= self.m_maxY.get()
            {
                world.neighborLightPropagationChanged(kind, k1, j5 + 1, l1, k5);
            }
            if l1 + 1 >= self.m_maxZ.get()
            {
                world.neighborLightPropagationChanged(kind, k1, j5, l1 + 1, k5);
            }
        }
    }

    pub fn expand(&self, minX: i32, minY: i32, minZ: i32, maxX: i32, maxY: i32, maxZ: i32) -> bool
    {
        let (mut minX, mut minY, mut minZ, mut maxX, mut maxY, mut maxZ): (i32, i32, i32, i32, i32, i32) =
            (minX, minY, minZ, maxX, maxY, maxZ);
        if minX >= self.m_minX.get()
            && minY >= self.m_minY.get()
            && minZ >= self.m_minZ.get()
            && maxX <= self.m_maxX.get()
            && maxY <= self.m_maxY.get()
            && maxZ <= self.m_maxZ.get()
        {
            return true;
        }

        let b0: i32 = 1;
        if minX >= self.m_minX.get() - b0
            && minY >= self.m_minY.get() - b0
            && minZ >= self.m_minZ.get() - b0
            && maxX <= self.m_maxX.get() + b0
            && maxY <= self.m_maxY.get() + b0
            && maxZ <= self.m_maxZ.get() + b0
        {
            let i: i32 = self.m_maxX.get() - self.m_minX.get();
            let j: i32 = self.m_maxY.get() - self.m_minY.get();
            let k: i32 = self.m_maxZ.get() - self.m_minZ.get();
            if minX > self.m_minX.get()
            {
                minX = self.m_minX.get();
            }
            if minY > self.m_minY.get()
            {
                minY = self.m_minY.get();
            }
            if minZ > self.m_minZ.get()
            {
                minZ = self.m_minZ.get();
            }
            if maxX < self.m_maxX.get()
            {
                maxX = self.m_maxX.get();
            }
            if maxY < self.m_maxY.get()
            {
                maxY = self.m_maxY.get();
            }
            if maxZ < self.m_maxZ.get()
            {
                maxZ = self.m_maxZ.get();
            }

            let l: i32 = maxX - minX;
            let i1: i32 = maxY - minY;
            let j1: i32 = maxZ - minZ;
            let k1: i32 = i * j * k;
            let l1: i32 = l * i1 * j1;
            if l1 - k1 <= 2
            {
                self.m_minX.set(minX);
                self.m_minY.set(minY);
                self.m_minZ.set(minZ);
                self.m_maxX.set(maxX);
                self.m_maxY.set(maxY);
                self.m_maxZ.set(maxZ);
                return true;
            }
        }
        false
    }
}
