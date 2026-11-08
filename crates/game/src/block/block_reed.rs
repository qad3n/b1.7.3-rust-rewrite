use crate::block::{BLOCK, Block, BlockClass, Material};
use crate::item::Item;
use crate::world::World;
use jvm::random::Random;
use std::rc::Rc;

pub fn growUp(this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32)
{
    if !world.isAirBlock(x, y + 1, z)
    {
        return;
    }
    let mut n: i32 = 1;
    while world.getBlockId(x, y - n, z) == this.m_blockID
    {
        n += 1;
    }
    if n < 3
    {
        let n2: i32 = world.getBlockMetadata(x, y, z);
        if n2 == 15
        {
            world.setBlockWithNotify(x, y + 1, z, this.m_blockID);
            world.setBlockMetadataWithNotify(x, y, z, 0);
        }
        else
        {
            world.setBlockMetadataWithNotify(x, y, z, n2 + 1);
        }
    }
}

fn canPlaceBlockAt(this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32) -> bool
{
    let n: i32 = world.getBlockId(x, y - 1, z);
    if n == this.m_blockID
    {
        return true;
    }
    if n != Block::grass().m_blockID && n != Block::dirt().m_blockID
    {
        return false;
    }
    let water: Rc<Material> = Material::water();
    Rc::ptr_eq(&world.getBlockMaterial(x - 1, y - 1, z), &water)
        || Rc::ptr_eq(&world.getBlockMaterial(x + 1, y - 1, z), &water)
        || Rc::ptr_eq(&world.getBlockMaterial(x, y - 1, z - 1), &water)
        || Rc::ptr_eq(&world.getBlockMaterial(x, y - 1, z + 1), &water)
}

fn checkBlockCoordValid(this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32)
{
    if !this.canBlockStay(world, x, y, z)
    {
        this.dropBlockAsItem(world, x, y, z, world.getBlockMetadata(x, y, z));
        world.setBlockWithNotify(x, y, z, 0);
    }
}

pub const CLASS: BlockClass = BlockClass {
    m_name: "BlockReed",
    m_parent: Some(&BLOCK),
    updateTick: |this, world, x, y, z, _: &Random| growUp(this, world, x, y, z),
    canPlaceBlockAt,
    onNeighborBlockChange: |this, world, x, y, z, _| checkBlockCoordValid(this, world, x, y, z),
    canBlockStay: |this, world, x, y, z| this.canPlaceBlockAt(world, x, y, z),
    getCollisionBoundingBoxFromPool: |_, _, _, _, _| None,
    idDropped: |_, _, _| Item::reed().m_shiftedIndex,
    isOpaqueCube: |_| false,
    renderAsNormalBlock: |_| false,
    getRenderType: |_| 1,
    ..BLOCK
};

pub fn create(id: i32, sprite: i32) -> Rc<Block>
{
    let block: Rc<Block> = Block::construct(&CLASS, Box::new(()), id, Material::plants());
    block.m_blockIndexInTexture.set(sprite);
    let f: f32 = 0.375;
    block.setBlockBounds(0.5 - f, 0.0, 0.5 - f, 0.5 + f, 1.0, 0.5 + f);
    block.setTickOnLoad(true)
}
