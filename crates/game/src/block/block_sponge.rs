use crate::block::{BLOCK, Block, BlockClass, Material};
use crate::world::World;
use std::rc::Rc;

fn onBlockAdded(_this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32)
{
    let b0: i32 = 2;
    for i in (x - b0)..=(x + b0)
    {
        for j in (y - b0)..=(y + b0)
        {
            for k in (z - b0)..=(z + b0)
            {
                let _ = Rc::ptr_eq(&world.getBlockMaterial(i, j, k), &Material::water());
            }
        }
    }
}

fn onBlockRemoval(_this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32)
{
    let b0: i32 = 2;
    for i in (x - b0)..=(x + b0)
    {
        for j in (y - b0)..=(y + b0)
        {
            for k in (z - b0)..=(z + b0)
            {
                world.notifyBlocksOfNeighborChange(i, j, k, world.getBlockId(i, j, k));
            }
        }
    }
}

pub const CLASS: BlockClass =
    BlockClass { m_name: "BlockSponge", m_parent: Some(&BLOCK), onBlockAdded, onBlockRemoval, ..BLOCK };

pub fn create(id: i32) -> Rc<Block>
{
    let block: Rc<Block> = Block::construct(&CLASS, Box::new(()), id, Material::sponge());
    block.m_blockIndexInTexture.set(48);
    block
}
