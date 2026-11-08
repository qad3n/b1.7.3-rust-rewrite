use crate::block::{BLOCK, Block, BlockClass, Material};
use crate::math::AxisAlignedBB;
use std::rc::Rc;

pub const CLASS: BlockClass = BlockClass {
    m_name: "BlockSoulSand",
    m_parent: Some(&BLOCK),
    getCollisionBoundingBoxFromPool: |_, _, x, y, z| {
        let f: f32 = 0.125;
        Some(AxisAlignedBB::getBoundingBoxFromPool(
            x as f64,
            y as f64,
            z as f64,
            (x + 1) as f64,
            ((y + 1) as f32 - f) as f64,
            (z + 1) as f64,
        ))
    },
    onEntityCollidedWithBlock: |_, _, _, _, _, entity| {
        entity.m_motionX.set(entity.m_motionX.get() * 0.4);
        entity.m_motionZ.set(entity.m_motionZ.get() * 0.4);
    },
    ..BLOCK
};

pub fn create(id: i32, sprite: i32) -> Rc<Block>
{
    Block::constructWithSprite(&CLASS, Box::new(()), id, sprite, Material::sand())
}
