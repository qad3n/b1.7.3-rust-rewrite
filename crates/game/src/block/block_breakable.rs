use crate::block::{BLOCK, Block, BlockClass, Material};
use crate::world::IBlockAccess;
use std::cell::Cell;
use std::rc::Rc;

#[derive(Default)]
pub struct BreakableExt
{
    pub m_localFlag: Cell<bool>,
}

fn shouldSideBeRendered(this: &Rc<Block>, world: &dyn IBlockAccess, x: i32, y: i32, z: i32, face: i32) -> bool
{
    let n: i32 = world.getBlockId(x, y, z);
    if !this.ext::<BreakableExt>().m_localFlag.get() && n == this.m_blockID
    {
        return false;
    }
    (BLOCK.shouldSideBeRendered)(this, world, x, y, z, face)
}

pub const CLASS: BlockClass = BlockClass {
    m_name: "BlockBreakable",
    m_parent: Some(&BLOCK),
    isOpaqueCube: |_| false,
    shouldSideBeRendered,
    ..BLOCK
};

pub fn construct(class: &'static BlockClass, id: i32, sprite: i32, material: Rc<Material>, culling: bool) -> Rc<Block>
{
    let block: Rc<Block> = Block::constructWithSprite(class, Box::new(BreakableExt::default()), id, sprite, material);
    block.ext::<BreakableExt>().m_localFlag.set(culling);
    block
}
