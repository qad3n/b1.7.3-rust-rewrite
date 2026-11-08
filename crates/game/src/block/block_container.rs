use crate::block::{BLOCK, Block, BlockClass, Material};
use std::any::Any;
use std::rc::Rc;

pub const CLASS: BlockClass = BlockClass {
    m_name: "BlockContainer",
    m_parent: Some(&BLOCK),
    onBlockAdded: |this, world, x, y, z| {
        (BLOCK.onBlockAdded)(this, world, x, y, z);
        world.setBlockTileEntity(x, y, z, &this.getBlockEntity());
    },
    onBlockRemoval: |this, world, x, y, z| {
        (BLOCK.onBlockRemoval)(this, world, x, y, z);
        world.removeBlockTileEntity(x, y, z);
    },
    ..BLOCK
};

pub fn construct(class: &'static BlockClass, ext: Box<dyn Any>, id: i32, material: Rc<Material>) -> Rc<Block>
{
    let block: Rc<Block> = Block::construct(class, ext, id, material);
    Block::setIsBlockContainerEntry(id, true);
    block
}

pub fn constructWithSprite(
    class: &'static BlockClass,
    ext: Box<dyn Any>,
    id: i32,
    sprite: i32,
    material: Rc<Material>,
) -> Rc<Block>
{
    let block: Rc<Block> = Block::constructWithSprite(class, ext, id, sprite, material);
    Block::setIsBlockContainerEntry(id, true);
    block
}
