use crate::block::Material;
use crate::tileentity::TileEntity;
use crate::world::World;
use crate::worldgen::WorldChunkManager;
use std::rc::Rc;

pub trait IBlockAccess
{
    fn getBlockId(&self, x: i32, y: i32, z: i32) -> i32;

    fn getBlockTileEntity(&self, x: i32, y: i32, z: i32) -> Option<Rc<TileEntity>>;

    fn getBrightness(&self, x: i32, y: i32, z: i32, blockLight: i32) -> f32;

    fn getLightBrightness(&self, x: i32, y: i32, z: i32) -> f32;

    fn getBlockMetadata(&self, x: i32, y: i32, z: i32) -> i32;

    fn getBlockMaterial(&self, x: i32, y: i32, z: i32) -> Rc<Material>;

    fn isBlockOpaqueCube(&self, x: i32, y: i32, z: i32) -> bool;

    fn isBlockNormalCube(&self, x: i32, y: i32, z: i32) -> bool;

    fn getWorldChunkManager(&self) -> Rc<WorldChunkManager>;
}

impl IBlockAccess for World
{
    fn getBlockId(&self, x: i32, y: i32, z: i32) -> i32 { Self::getBlockId(self, x, y, z) }

    fn getBlockTileEntity(&self, x: i32, y: i32, z: i32) -> Option<Rc<TileEntity>>
    {
        Self::getBlockTileEntity(self, x, y, z)
    }

    fn getBrightness(&self, x: i32, y: i32, z: i32, blockLight: i32) -> f32
    {
        Self::getBrightness(self, x, y, z, blockLight)
    }

    fn getLightBrightness(&self, x: i32, y: i32, z: i32) -> f32 { Self::getLightBrightness(self, x, y, z) }

    fn getBlockMetadata(&self, x: i32, y: i32, z: i32) -> i32 { Self::getBlockMetadata(self, x, y, z) }

    fn getBlockMaterial(&self, x: i32, y: i32, z: i32) -> Rc<Material> { Self::getBlockMaterial(self, x, y, z) }

    fn isBlockOpaqueCube(&self, x: i32, y: i32, z: i32) -> bool { Self::isBlockOpaqueCube(self, x, y, z) }

    fn isBlockNormalCube(&self, x: i32, y: i32, z: i32) -> bool { Self::isBlockNormalCube(self, x, y, z) }

    fn getWorldChunkManager(&self) -> Rc<WorldChunkManager> { Self::getWorldChunkManager(self) }
}
