use crate::block::{BLOCK, Block, BlockClass, Material};
use crate::math::AxisAlignedBB;
use std::rc::Rc;

pub const CLASS: BlockClass = BlockClass {
    m_name: "BlockFence",
    m_parent: Some(&BLOCK),
    canPlaceBlockAt: |this, world, x, y, z| {
        if world.getBlockId(x, y - 1, z) == this.m_blockID
        {
            return true;
        }
        if !world.getBlockMaterial(x, y - 1, z).isSolid()
        {
            return false;
        }
        (BLOCK.canPlaceBlockAt)(this, world, x, y, z)
    },
    getCollisionBoundingBoxFromPool: |_, _, x, y, z| {
        Some(AxisAlignedBB::getBoundingBoxFromPool(
            x as f64,
            y as f64,
            z as f64,
            (x + 1) as f64,
            (y as f32 + 1.5) as f64,
            (z + 1) as f64,
        ))
    },
    isOpaqueCube: |_| false,
    renderAsNormalBlock: |_| false,
    getRenderType: |_| 11,
    ..BLOCK
};

pub fn create(id: i32, sprite: i32) -> Rc<Block>
{
    Block::constructWithSprite(&CLASS, Box::new(()), id, sprite, Material::wood())
}
