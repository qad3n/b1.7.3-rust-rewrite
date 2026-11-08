use crate::block::{BLOCK, Block, BlockClass, Material};
use crate::world::World;
use std::any::Any;
use std::rc::Rc;

pub fn canSurviveOrBreak(this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32)
{
    if !this.canBlockStay(world, x, y, z)
    {
        this.dropBlockAsItem(world, x, y, z, world.getBlockMetadata(x, y, z));
        world.setBlockWithNotify(x, y, z, 0);
    }
}

pub const CLASS: BlockClass = BlockClass {
    m_name: "BlockFlower",
    m_parent: Some(&BLOCK),
    canPlaceBlockAt: |this, world, x, y, z| {
        (BLOCK.canPlaceBlockAt)(this, world, x, y, z)
            && this.canThisPlantGrowOnThisBlockID(world.getBlockId(x, y - 1, z))
    },
    canThisPlantGrowOnThisBlockID: |_, block| {
        block == Block::grass().m_blockID || block == Block::dirt().m_blockID || block == Block::tilledField().m_blockID
    },
    onNeighborBlockChange: |this, world, x, y, z, neighborBlock| {
        (BLOCK.onNeighborBlockChange)(this, world, x, y, z, neighborBlock);
        canSurviveOrBreak(this, world, x, y, z);
    },
    updateTick: |this, world, x, y, z, _| canSurviveOrBreak(this, world, x, y, z),
    canBlockStay: |this, world, x, y, z| {
        (world.getFullBlockLightValue(x, y, z) >= 8 || world.canBlockSeeTheSky(x, y, z))
            && this.canThisPlantGrowOnThisBlockID(world.getBlockId(x, y - 1, z))
    },
    getCollisionBoundingBoxFromPool: |_, _, _, _, _| None,
    isOpaqueCube: |_| false,
    renderAsNormalBlock: |_| false,
    getRenderType: |_| 1,
    ..BLOCK
};

pub fn construct(class: &'static BlockClass, ext: Box<dyn Any>, id: i32, sprite: i32) -> Rc<Block>
{
    let block: Rc<Block> = Block::construct(class, ext, id, Material::plants());
    block.m_blockIndexInTexture.set(sprite);
    let block: Rc<Block> = block.setTickOnLoad(true);
    let f: f32 = 0.2;
    block.setBlockBounds(0.5 - f, 0.0, 0.5 - f, 0.5 + f, f * 3.0, 0.5 + f);
    block
}

pub fn create(id: i32, sprite: i32) -> Rc<Block> { construct(&CLASS, Box::new(()), id, sprite) }
