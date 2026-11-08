use crate::block::{BLOCK, Block, BlockClass, Material, StepSound, block_piston_base};
use crate::math::AxisAlignedBB;
use crate::world::{IBlockAccess, World};
use common::world::piston_block_textures::{OPPOSITE, X_OFFSET, Y_OFFSET, Z_OFFSET};
use jvm::array_list::JArrayList;
use jvm::throwable::OrNpe;
use std::cell::Cell;
use std::rc::Rc;

pub struct ExtensionExt
{
    m_forcedSprite: Cell<i32>,
}

fn forcedSprite(this: &Block) -> &Cell<i32> { &this.ext::<ExtensionExt>().m_forcedSprite }

pub fn setForcedSprite(this: &Block, sprite: i32) { forcedSprite(this).set(sprite); }

pub fn clearForcedSprite(this: &Block) { forcedSprite(this).set(-1); }

fn onBlockRemoval(this: &Rc<Block>, world: &Rc<World>, mut x: i32, mut y: i32, mut z: i32)
{
    (BLOCK.onBlockRemoval)(this, world, x, y, z);
    let mut i: i32 = world.getBlockMetadata(x, y, z);
    let j: usize = OPPOSITE[getFacing(i) as usize] as usize;
    x += X_OFFSET[j];
    y += Y_OFFSET[j];
    z += Z_OFFSET[j];
    let k: i32 = world.getBlockId(x, y, z);
    if k == Block::pistonBase().m_blockID || k == Block::pistonStickyBase().m_blockID
    {
        i = world.getBlockMetadata(x, y, z);
        if block_piston_base::isPowered(i)
        {
            Block::byId(k).orNpe().dropBlockAsItem(world, x, y, z, i);
            world.setBlockWithNotify(x, y, z, 0);
        }
    }
}

fn getBlockTextureFromSideAndMetadata(this: &Rc<Block>, face: i32, metadata: i32) -> i32
{
    let n: i32 = getFacing(metadata);
    if face == n
    {
        if forcedSprite(this).get() >= 0
        {
            return forcedSprite(this).get();
        }
        if (metadata & 8) != 0
        {
            return this.m_blockIndexInTexture.get() - 1;
        }
        return this.m_blockIndexInTexture.get();
    }
    if face == OPPOSITE[n as usize]
    {
        return 107;
    }
    108
}

fn getCollidingBoundingBoxes(
    this: &Rc<Block>,
    world: &Rc<World>,
    x: i32,
    y: i32,
    z: i32,
    shape: &AxisAlignedBB,
    collisions: &JArrayList<Rc<AxisAlignedBB>>,
)
{
    let i: i32 = world.getBlockMetadata(x, y, z);
    let collide = |bounds: [f32; 6]| {
        this.setBlockBounds(bounds[0], bounds[1], bounds[2], bounds[3], bounds[4], bounds[5]);
        (BLOCK.getCollidingBoundingBoxes)(this, world, x, y, z, shape, collisions);
    };
    match getFacing(i)
    {
        0 =>
        {
            collide([0.0, 0.0, 0.0, 1.0, 0.25, 1.0]);
            collide([0.375, 0.25, 0.375, 0.625, 1.0, 0.625]);
        }
        1 =>
        {
            collide([0.0, 0.75, 0.0, 1.0, 1.0, 1.0]);
            collide([0.375, 0.0, 0.375, 0.625, 0.75, 0.625]);
        }
        2 =>
        {
            collide([0.0, 0.0, 0.0, 1.0, 1.0, 0.25]);
            collide([0.25, 0.375, 0.25, 0.75, 0.625, 1.0]);
        }
        3 =>
        {
            collide([0.0, 0.0, 0.75, 1.0, 1.0, 1.0]);
            collide([0.25, 0.375, 0.0, 0.75, 0.625, 0.75]);
        }
        4 =>
        {
            collide([0.0, 0.0, 0.0, 0.25, 1.0, 1.0]);
            collide([0.375, 0.25, 0.25, 0.625, 0.75, 1.0]);
        }
        5 =>
        {
            collide([0.75, 0.0, 0.0, 1.0, 1.0, 1.0]);
            collide([0.0, 0.375, 0.25, 0.75, 0.625, 0.75]);
        }
        _ =>
        {}
    }
    this.setBlockBounds(0.0, 0.0, 0.0, 1.0, 1.0, 1.0);
}

fn setBlockBoundsBasedOnState(this: &Rc<Block>, world: &dyn IBlockAccess, x: i32, y: i32, z: i32)
{
    let i: i32 = world.getBlockMetadata(x, y, z);
    match getFacing(i)
    {
        0 => this.setBlockBounds(0.0, 0.0, 0.0, 1.0, 0.25, 1.0),
        1 => this.setBlockBounds(0.0, 0.75, 0.0, 1.0, 1.0, 1.0),
        2 => this.setBlockBounds(0.0, 0.0, 0.0, 1.0, 1.0, 0.25),
        3 => this.setBlockBounds(0.0, 0.0, 0.75, 1.0, 1.0, 1.0),
        4 => this.setBlockBounds(0.0, 0.0, 0.0, 0.25, 1.0, 1.0),
        5 => this.setBlockBounds(0.75, 0.0, 0.0, 1.0, 1.0, 1.0),
        _ =>
        {}
    }
}

fn onNeighborBlockChange(_this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32, neighborBlock: i32)
{
    let i: usize = getFacing(world.getBlockMetadata(x, y, z)) as usize;
    let j: i32 = world.getBlockId(x - X_OFFSET[i], y - Y_OFFSET[i], z - Z_OFFSET[i]);
    if j != Block::pistonBase().m_blockID && j != Block::pistonStickyBase().m_blockID
    {
        world.setBlockWithNotify(x, y, z, 0);
    }
    else
    {
        Block::byId(j).orNpe().onNeighborBlockChange(
            world,
            x - X_OFFSET[i],
            y - Y_OFFSET[i],
            z - Z_OFFSET[i],
            neighborBlock,
        );
    }
}

pub fn getFacing(metadata: i32) -> i32 { metadata & 7 }

pub const CLASS: BlockClass = BlockClass {
    m_name: "BlockPistonExtension",
    m_parent: Some(&BLOCK),
    onBlockRemoval,
    getBlockTextureFromSideAndMetadata,
    getRenderType: |_| 17,
    isOpaqueCube: |_| false,
    renderAsNormalBlock: |_| false,
    canPlaceBlockAt: |_, _, _, _, _| false,
    canPlaceBlockOnSide: |_, _, _, _, _, _| false,
    quantityDropped: |_, _| 0,
    getCollidingBoundingBoxes,
    setBlockBoundsBasedOnState,
    onNeighborBlockChange,
    ..BLOCK
};

pub fn create(id: i32, sprite: i32) -> Rc<Block>
{
    let ext: ExtensionExt = ExtensionExt { m_forcedSprite: Cell::new(-1) };
    let block: Rc<Block> = Block::constructWithSprite(&CLASS, Box::new(ext), id, sprite, Material::PISTON());
    block.setStepSound(StepSound::sounds(|s| Rc::clone(&s.m_stone))).setHardness(0.5)
}
