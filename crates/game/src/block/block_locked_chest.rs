use crate::block::{BLOCK, Block, BlockClass, Material};
use crate::world::IBlockAccess;
use std::rc::Rc;

fn getBlockTexture(this: &Rc<Block>, world: &dyn IBlockAccess, x: i32, y: i32, z: i32, face: i32) -> i32
{
    let index: i32 = this.m_blockIndexInTexture.get();
    if face == 1
    {
        return index - 1;
    }
    if face == 0
    {
        return index - 1;
    }
    let n: i32 = world.getBlockId(x, y, z - 1);
    let n2: i32 = world.getBlockId(x, y, z + 1);
    let n3: i32 = world.getBlockId(x - 1, y, z);
    let n4: i32 = world.getBlockId(x + 1, y, z);
    let opaque = Block::opaqueCubeLookup;
    let mut n5: i32 = 3;
    if opaque(n) && !opaque(n2)
    {
        n5 = 3;
    }
    if opaque(n2) && !opaque(n)
    {
        n5 = 2;
    }
    if opaque(n3) && !opaque(n4)
    {
        n5 = 5;
    }
    if opaque(n4) && !opaque(n3)
    {
        n5 = 4;
    }
    if face == n5 { index + 1 } else { index }
}

pub const CLASS: BlockClass = BlockClass {
    m_name: "BlockLockedChest",
    m_parent: Some(&BLOCK),
    getBlockTexture,
    getBlockTextureFromSide: |this, face| {
        let index: i32 = this.m_blockIndexInTexture.get();
        if face == 1
        {
            return index - 1;
        }
        if face == 0
        {
            return index - 1;
        }
        if face == 3
        {
            return index + 1;
        }
        index
    },
    canPlaceBlockAt: |_, _, _, _, _| true,
    updateTick: |_, world, x, y, z, _| {
        world.setBlockWithNotify(x, y, z, 0);
    },
    ..BLOCK
};

pub fn create(id: i32) -> Rc<Block>
{
    let block: Rc<Block> = Block::construct(&CLASS, Box::new(()), id, Material::wood());
    block.m_blockIndexInTexture.set(26);
    block
}
