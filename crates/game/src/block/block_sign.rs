use crate::block::{Block, BlockClass, Material, block_container};
use crate::item::Item;
use crate::tileentity::TileEntityKind;
use crate::world::{IBlockAccess, World};
use std::rc::Rc;

pub struct SignExt
{
    m_signEntityClass: TileEntityKind,
    m_isFreestanding: bool,
}

fn sign(this: &Block) -> &SignExt { this.ext::<SignExt>() }

fn setBlockBoundsBasedOnState(this: &Rc<Block>, world: &dyn IBlockAccess, x: i32, y: i32, z: i32)
{
    if sign(this).m_isFreestanding
    {
        return;
    }
    let n: i32 = world.getBlockMetadata(x, y, z);
    let f: f32 = 0.28125;
    let f2: f32 = 0.78125;
    let f3: f32 = 0.0;
    let f4: f32 = 1.0;
    let f5: f32 = 0.125;
    this.setBlockBounds(0.0, 0.0, 0.0, 1.0, 1.0, 1.0);
    if n == 2
    {
        this.setBlockBounds(f3, f, 1.0 - f5, f4, f2, 1.0);
    }
    if n == 3
    {
        this.setBlockBounds(f3, f, 0.0, f4, f2, f5);
    }
    if n == 4
    {
        this.setBlockBounds(1.0 - f5, f, f3, 1.0, f2, f4);
    }
    if n == 5
    {
        this.setBlockBounds(0.0, f, f3, f5, f2, f4);
    }
}

fn onNeighborBlockChange(this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32, neighborBlock: i32)
{
    let mut bl: bool = false;
    if sign(this).m_isFreestanding
    {
        if !world.getBlockMaterial(x, y - 1, z).isSolid()
        {
            bl = true;
        }
    }
    else
    {
        let n: i32 = world.getBlockMetadata(x, y, z);
        bl = true;
        if n == 2 && world.getBlockMaterial(x, y, z + 1).isSolid()
        {
            bl = false;
        }
        if n == 3 && world.getBlockMaterial(x, y, z - 1).isSolid()
        {
            bl = false;
        }
        if n == 4 && world.getBlockMaterial(x + 1, y, z).isSolid()
        {
            bl = false;
        }
        if n == 5 && world.getBlockMaterial(x - 1, y, z).isSolid()
        {
            bl = false;
        }
    }
    if bl
    {
        this.dropBlockAsItem(world, x, y, z, world.getBlockMetadata(x, y, z));
        world.setBlockWithNotify(x, y, z, 0);
    }
    (block_container::CLASS.onNeighborBlockChange)(this, world, x, y, z, neighborBlock);
}

pub const CLASS: BlockClass = BlockClass {
    m_name: "BlockSign",
    m_parent: Some(&block_container::CLASS),
    getCollisionBoundingBoxFromPool: |_, _, _, _, _| None,
    getSelectedBoundingBoxFromPool: |this, world, x, y, z| {
        this.setBlockBoundsBasedOnState(&**world, x, y, z);
        (block_container::CLASS.getSelectedBoundingBoxFromPool)(this, world, x, y, z)
    },
    setBlockBoundsBasedOnState,
    getRenderType: |_| -1,
    renderAsNormalBlock: |_| false,
    isOpaqueCube: |_| false,
    getBlockEntity: |this| sign(this).m_signEntityClass.newInstance(),
    idDropped: |_, _, _| Item::sign().m_shiftedIndex,
    onNeighborBlockChange,
    ..block_container::CLASS
};

pub fn create(id: i32, blockEntityType: TileEntityKind, standing: bool) -> Rc<Block>
{
    let ext: SignExt = SignExt { m_signEntityClass: blockEntityType, m_isFreestanding: standing };
    let block: Rc<Block> = block_container::construct(&CLASS, Box::new(ext), id, Material::wood());
    block.m_blockIndexInTexture.set(4);
    let f: f32 = 0.25;
    let f2: f32 = 1.0;
    block.setBlockBounds(0.5 - f, 0.0, 0.5 - f, 0.5 + f, f2, 0.5 + f);
    block
}
