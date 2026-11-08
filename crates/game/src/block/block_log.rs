use crate::block::{BLOCK, Block, BlockClass, Material};
use crate::world::World;
use std::rc::Rc;

fn onBlockRemoval(_this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32)
{
    let b0: i32 = 4;
    let i: i32 = b0 + 1;
    if !world.checkChunksExist(x - i, y - i, z - i, x + i, y + i, z + i)
    {
        return;
    }
    for j in -b0..=b0
    {
        for k in -b0..=b0
        {
            for l in -b0..=b0
            {
                let i1: i32 = world.getBlockId(x + j, y + k, z + l);
                if i1 == Block::leaves().m_blockID
                {
                    let j1: i32 = world.getBlockMetadata(x + j, y + k, z + l);
                    if (j1 & 8) == 0
                    {
                        world.setBlockMetadata(x + j, y + k, z + l, j1 | 8);
                    }
                }
            }
        }
    }
}

fn getBlockTextureFromSideAndMetadata(_this: &Rc<Block>, face: i32, metadata: i32) -> i32
{
    if face == 1 || face == 0
    {
        return 21;
    }
    match metadata
    {
        1 => 116,
        2 => 117,
        _ => 20,
    }
}

pub const CLASS: BlockClass = BlockClass {
    m_name: "BlockLog",
    m_parent: Some(&BLOCK),
    quantityDropped: |_, _| 1,
    idDropped: |_, _, _| Block::wood().m_blockID,
    harvestBlock: |this, world, player, x, y, z, metadata| (BLOCK.harvestBlock)(this, world, player, x, y, z, metadata),
    onBlockRemoval,
    getBlockTextureFromSideAndMetadata,
    damageDropped: |_, metadata| metadata,
    ..BLOCK
};

pub fn create(id: i32) -> Rc<Block>
{
    let block: Rc<Block> = Block::construct(&CLASS, Box::new(()), id, Material::wood());
    block.m_blockIndexInTexture.set(20);
    block
}
