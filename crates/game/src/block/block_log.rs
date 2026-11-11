use crate::block::{BLOCK, Block, BlockClass, Material, block_leaves};
use crate::tags::{self, Tag};
use crate::world::World;
use std::rc::Rc;

fn onBlockRemoval(_this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32) { shed(world, x, y, z); }

// rewrite: a mod log in minecraft:logs lets its leaves go as beta's log does in its own removal
pub fn removed(world: &Rc<World>, block: i32, x: i32, y: i32, z: i32)
{
    if block != Block::WOOD && tags::has(Tag::Logs, block)
    {
        shed(world, x, y, z);
    }
}

fn shed(world: &Rc<World>, x: i32, y: i32, z: i32)
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
                // rewrite: mod leaves decay around a broken log too
                if tags::has(Tag::Leaves, i1)
                {
                    let j1: i32 = world.getBlockMetadata(x + j, y + k, z + l);
                    if !block_leaves::decaying(i1, j1)
                    {
                        world.setBlockMetadata(x + j, y + k, z + l, block_leaves::marked(i1, j1));
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

pub const CLASS: BlockClass = BlockClass
{
    m_name: "BlockLog",
    m_parent: Some(&BLOCK),
    quantityDropped: |_, _| 1,
    idDropped: |_, _, _| Block::WOOD,
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
