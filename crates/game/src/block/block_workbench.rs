use crate::block::{BLOCK, Block, BlockClass, Material};
use std::rc::Rc;

fn getBlockTextureFromSide(this: &Rc<Block>, face: i32) -> i32
{
    if face == 1
    {
        return this.m_blockIndexInTexture.get() - 16;
    }
    if face == 0
    {
        return Block::planks().getBlockTextureFromSide(0);
    }
    if face == 2 || face == 4
    {
        return this.m_blockIndexInTexture.get() + 1;
    }
    this.m_blockIndexInTexture.get()
}

pub const CLASS: BlockClass = BlockClass {
    m_name: "BlockWorkbench",
    m_parent: Some(&BLOCK),
    getBlockTextureFromSide,
    blockActivated: |_, world, x, y, z, player| {
        if world.m_multiplayerWorld.get()
        {
            return true;
        }
        player.displayWorkbenchGUI(x, y, z);
        true
    },
    ..BLOCK
};

pub fn create(id: i32) -> Rc<Block>
{
    let block: Rc<Block> = Block::construct(&CLASS, Box::new(()), id, Material::wood());
    block.m_blockIndexInTexture.set(59);
    block
}
