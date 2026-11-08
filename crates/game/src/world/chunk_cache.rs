use crate::block::{Block, Material};
use crate::tileentity::TileEntity;
use crate::world::{Chunk, IBlockAccess, World};
use crate::worldgen::WorldChunkManager;
use jvm::throwable::OrNpe;
use std::rc::Rc;

pub struct ChunkCache
{
    m_chunkX: i32,
    m_chunkZ: i32,
    m_chunkArray: Vec<Vec<Rc<Chunk>>>,
    m_worldObj: Rc<World>,
}

fn arrayIndex(index: i32, length: usize) -> usize
{
    if !(0..length as i32).contains(&index)
    {
        jvm::throwable::throw(&jvm::throwable::ARRAY_INDEX, &index.to_string());
    }
    index as usize
}

impl ChunkCache
{
    pub fn new(world: &Rc<World>, minX: i32, _minY: i32, minZ: i32, maxX: i32, _maxY: i32, maxZ: i32)
    -> Rc<ChunkCache>
    {
        let chunkX: i32 = minX >> 4;
        let chunkZ: i32 = minZ >> 4;
        let n: i32 = maxX >> 4;
        let n2: i32 = maxZ >> 4;
        let mut chunkArray: Vec<Vec<Rc<Chunk>>> = Vec::with_capacity((n - chunkX + 1) as usize);
        for i in chunkX..=n
        {
            let mut column: Vec<Rc<Chunk>> = Vec::with_capacity((n2 - chunkZ + 1) as usize);
            for j in chunkZ..=n2
            {
                column.push(world.getChunkFromChunkCoords(i, j));
            }
            chunkArray.push(column);
        }
        Rc::new(ChunkCache {
            m_chunkX: chunkX,
            m_chunkZ: chunkZ,
            m_chunkArray: chunkArray,
            m_worldObj: Rc::clone(world),
        })
    }

    fn chunkAt(&self, x: i32, z: i32) -> Rc<Chunk>
    {
        let column: &Vec<Rc<Chunk>> = &self.m_chunkArray[arrayIndex((x >> 4) - self.m_chunkX, self.m_chunkArray.len())];
        Rc::clone(&column[arrayIndex((z >> 4) - self.m_chunkZ, column.len())])
    }

    fn brightness(&self, light: i32) -> f32
    {
        let table: [f32; 16] = self.m_worldObj.m_worldProvider.get().orNpe().m_lightBrightnessTable.get();
        table[arrayIndex(light, table.len())]
    }

    pub fn getLightValue(&self, x: i32, y: i32, z: i32) -> i32 { self.getLightValueExt(x, y, z, true) }

    pub fn getLightValueExt(&self, x: i32, y: i32, z: i32, useNeighborLight: bool) -> i32
    {
        if x < -32000000 || z < -32000000 || x >= 32000000 || z > 32000000
        {
            return 15;
        }
        if useNeighborLight
        {
            let n: i32 = self.getBlockId(x, y, z);
            if n == Block::stairSingle().m_blockID
                || n == Block::tilledField().m_blockID
                || n == Block::stairCompactPlanks().m_blockID
                || n == Block::stairCompactCobblestone().m_blockID
            {
                let mut n2: i32 = self.getLightValueExt(x, y + 1, z, false);
                let n3: i32 = self.getLightValueExt(x + 1, y, z, false);
                let n4: i32 = self.getLightValueExt(x - 1, y, z, false);
                let n5: i32 = self.getLightValueExt(x, y, z + 1, false);
                let n6: i32 = self.getLightValueExt(x, y, z - 1, false);
                if n3 > n2
                {
                    n2 = n3;
                }
                if n4 > n2
                {
                    n2 = n4;
                }
                if n5 > n2
                {
                    n2 = n5;
                }
                if n6 > n2
                {
                    n2 = n6;
                }
                return n2;
            }
        }
        if y < 0
        {
            return 0;
        }
        if y >= 128
        {
            return (15 - self.m_worldObj.m_skylightSubtracted.get()).max(0);
        }
        self.chunkAt(x, z).getBlockLightValue(x & 0xF, y, z & 0xF, self.m_worldObj.m_skylightSubtracted.get())
    }
}

impl IBlockAccess for ChunkCache
{
    fn getBlockId(&self, x: i32, y: i32, z: i32) -> i32
    {
        if !(0..128).contains(&y)
        {
            return 0;
        }
        let n: i32 = (x >> 4) - self.m_chunkX;
        let n2: i32 = (z >> 4) - self.m_chunkZ;
        if n < 0 || n >= self.m_chunkArray.len() as i32 || n2 < 0 || n2 >= self.m_chunkArray[n as usize].len() as i32
        {
            return 0;
        }
        self.m_chunkArray[n as usize][n2 as usize].getBlockID(x & 0xF, y, z & 0xF)
    }

    fn getBlockTileEntity(&self, x: i32, y: i32, z: i32) -> Option<Rc<TileEntity>>
    {
        self.chunkAt(x, z).getChunkBlockTileEntity(x & 0xF, y, z & 0xF)
    }

    fn getBrightness(&self, x: i32, y: i32, z: i32, blockLight: i32) -> f32
    {
        let mut n: i32 = self.getLightValue(x, y, z);
        if n < blockLight
        {
            n = blockLight;
        }
        self.brightness(n)
    }

    fn getLightBrightness(&self, x: i32, y: i32, z: i32) -> f32 { self.brightness(self.getLightValue(x, y, z)) }

    fn getBlockMetadata(&self, x: i32, y: i32, z: i32) -> i32
    {
        if !(0..128).contains(&y)
        {
            return 0;
        }
        self.chunkAt(x, z).getBlockMetadata(x & 0xF, y, z & 0xF)
    }

    fn getBlockMaterial(&self, x: i32, y: i32, z: i32) -> Rc<Material>
    {
        let n: i32 = self.getBlockId(x, y, z);
        if n == 0
        {
            return Material::air();
        }
        Rc::clone(&Block::byId(n).orNpe().m_blockMaterial)
    }

    fn isBlockOpaqueCube(&self, x: i32, y: i32, z: i32) -> bool
    {
        match Block::byId(self.getBlockId(x, y, z))
        {
            Some(block) => block.isOpaqueCube(),
            None => false,
        }
    }

    fn isBlockNormalCube(&self, x: i32, y: i32, z: i32) -> bool
    {
        match Block::byId(self.getBlockId(x, y, z))
        {
            Some(block) => block.m_blockMaterial.getIsSolid() && block.renderAsNormalBlock(),
            None => false,
        }
    }

    fn getWorldChunkManager(&self) -> Rc<WorldChunkManager> { self.m_worldObj.getWorldChunkManager() }
}
