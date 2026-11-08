use crate::block::{BLOCK, Block, BlockClass, Material};
use crate::world::IBlockAccess;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

#[derive(Default)]
pub struct LeavesExt
{
    pub m_graphicsLevel: Cell<bool>,
    pub m_baseIndexInPNG: Cell<i32>,
    pub m_adjacentTreeBlocks: RefCell<Option<Vec<i32>>>,
}

fn shouldSideBeRendered(this: &Rc<Block>, world: &dyn IBlockAccess, x: i32, y: i32, z: i32, face: i32) -> bool
{
    let n: i32 = world.getBlockId(x, y, z);
    if !this.ext::<LeavesExt>().m_graphicsLevel.get() && n == this.m_blockID
    {
        return false;
    }
    (BLOCK.shouldSideBeRendered)(this, world, x, y, z, face)
}

pub const CLASS: BlockClass = BlockClass {
    m_name: "BlockLeavesBase",
    m_parent: Some(&BLOCK),
    isOpaqueCube: |_| false,
    shouldSideBeRendered,
    ..BLOCK
};

pub fn construct(class: &'static BlockClass, id: i32, sprite: i32, material: Rc<Material>, culling: bool) -> Rc<Block>
{
    let block: Rc<Block> = Block::constructWithSprite(class, Box::new(LeavesExt::default()), id, sprite, material);
    block.ext::<LeavesExt>().m_graphicsLevel.set(culling);
    block
}
