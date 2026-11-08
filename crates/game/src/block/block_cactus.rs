use crate::block::{BLOCK, Block, BlockClass, Material, block_reed};
use crate::math::AxisAlignedBB;
use crate::world::World;
use std::rc::Rc;

fn canBlockStay(_this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32) -> bool
{
    if world.getBlockMaterial(x - 1, y, z).isSolid()
        || world.getBlockMaterial(x + 1, y, z).isSolid()
        || world.getBlockMaterial(x, y, z - 1).isSolid()
        || world.getBlockMaterial(x, y, z + 1).isSolid()
    {
        return false;
    }
    let n: i32 = world.getBlockId(x, y - 1, z);
    n == Block::cactus().m_blockID || n == Block::sand().m_blockID
}

pub const CLASS: BlockClass = BlockClass {
    m_name: "BlockCactus",
    m_parent: Some(&BLOCK),
    updateTick: |this, world, x, y, z, _| block_reed::growUp(this, world, x, y, z),
    getCollisionBoundingBoxFromPool: |_, _, x, y, z| {
        let f: f32 = 0.0625;
        Some(AxisAlignedBB::getBoundingBoxFromPool(
            (x as f32 + f) as f64,
            y as f64,
            (z as f32 + f) as f64,
            ((x + 1) as f32 - f) as f64,
            ((y + 1) as f32 - f) as f64,
            ((z + 1) as f32 - f) as f64,
        ))
    },
    getSelectedBoundingBoxFromPool: |_, _, x, y, z| {
        let f: f32 = 0.0625;
        AxisAlignedBB::getBoundingBoxFromPool(
            (x as f32 + f) as f64,
            y as f64,
            (z as f32 + f) as f64,
            ((x + 1) as f32 - f) as f64,
            (y + 1) as f64,
            ((z + 1) as f32 - f) as f64,
        )
    },
    getBlockTextureFromSide: |this, face| match face
    {
        1 => this.m_blockIndexInTexture.get() - 1,
        0 => this.m_blockIndexInTexture.get() + 1,
        _ => this.m_blockIndexInTexture.get(),
    },
    renderAsNormalBlock: |_| false,
    isOpaqueCube: |_| false,
    getRenderType: |_| 13,
    canPlaceBlockAt: |this, world, x, y, z| {
        (BLOCK.canPlaceBlockAt)(this, world, x, y, z) && this.canBlockStay(world, x, y, z)
    },
    onNeighborBlockChange: |this, world, x, y, z, _| {
        if !this.canBlockStay(world, x, y, z)
        {
            this.dropBlockAsItem(world, x, y, z, world.getBlockMetadata(x, y, z));
            world.setBlockWithNotify(x, y, z, 0);
        }
    },
    canBlockStay,
    onEntityCollidedWithBlock: |_, _, _, _, _, entity| {
        entity.attackEntityFrom(None, 1);
    },
    ..BLOCK
};

pub fn create(id: i32, sprite: i32) -> Rc<Block>
{
    Block::constructWithSprite(&CLASS, Box::new(()), id, sprite, Material::cactus()).setTickOnLoad(true)
}
