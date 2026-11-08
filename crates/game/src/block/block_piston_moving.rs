use crate::block::{Block, BlockClass, Material, block_container};
use crate::math::AxisAlignedBB;
use crate::tileentity::{TileEntity, tile_entity_piston};
use crate::world::{IBlockAccess, World};
use common::world::piston_block_textures::{X_OFFSET, Y_OFFSET, Z_OFFSET};
use jvm::throwable::OrNpe;
use std::rc::Rc;

pub fn createMovingBlockEntity(
    movedBlock: i32,
    movedMetadata: i32,
    facing: i32,
    extending: bool,
    source: bool,
) -> Rc<TileEntity>
{
    tile_entity_piston::create(movedBlock, movedMetadata, facing, extending, source)
}

fn getMovingBlockEntity(world: &dyn IBlockAccess, x: i32, y: i32, z: i32) -> Option<Rc<TileEntity>>
{
    world.getBlockTileEntity(x, y, z).filter(|tileEntity| tileEntity.instanceOf("TileEntityPiston"))
}

fn dropBlockAsItemWithChance(_this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32, _metadata: i32, _luck: f32)
{
    if world.m_multiplayerWorld.get()
    {
        return;
    }
    let Some(tileEntityPiston) = getMovingBlockEntity(&**world, x, y, z)
    else
    {
        return;
    };
    Block::byId(tile_entity_piston::getStoredBlockID(&tileEntityPiston)).orNpe().dropBlockAsItem(
        world,
        x,
        y,
        z,
        tileEntityPiston.getBlockMetadata(),
    );
}

fn getCollisionBoundingBoxFromPool(
    this: &Rc<Block>,
    world: &Rc<World>,
    x: i32,
    y: i32,
    z: i32,
) -> Option<Rc<AxisAlignedBB>>
{
    let tileEntityPiston: Rc<TileEntity> = getMovingBlockEntity(&**world, x, y, z)?;
    let mut f: f32 = tile_entity_piston::getProgress(&tileEntityPiston, 0.0);
    if tile_entity_piston::isExtending(&tileEntityPiston)
    {
        f = 1.0 - f;
    }
    getCollisionShape(
        this,
        world,
        x,
        y,
        z,
        tile_entity_piston::getStoredBlockID(&tileEntityPiston),
        f,
        tile_entity_piston::getFacing(&tileEntityPiston),
    )
}

fn setBlockBoundsBasedOnState(this: &Rc<Block>, world: &dyn IBlockAccess, x: i32, y: i32, z: i32)
{
    let Some(tileEntityPiston) = getMovingBlockEntity(world, x, y, z)
    else
    {
        return;
    };
    let Some(block) = Block::byId(tile_entity_piston::getStoredBlockID(&tileEntityPiston))
    else
    {
        return;
    };
    if Rc::ptr_eq(&block, this)
    {
        return;
    }
    block.setBlockBoundsBasedOnState(world, x, y, z);
    let mut f: f32 = tile_entity_piston::getProgress(&tileEntityPiston, 0.0);
    if tile_entity_piston::isExtending(&tileEntityPiston)
    {
        f = 1.0 - f;
    }
    let n: usize = tile_entity_piston::getFacing(&tileEntityPiston) as usize;
    this.m_minX.set(block.m_minX.get() - (X_OFFSET[n] as f32 * f) as f64);
    this.m_minY.set(block.m_minY.get() - (Y_OFFSET[n] as f32 * f) as f64);
    this.m_minZ.set(block.m_minZ.get() - (Z_OFFSET[n] as f32 * f) as f64);
    this.m_maxX.set(block.m_maxX.get() - (X_OFFSET[n] as f32 * f) as f64);
    this.m_maxY.set(block.m_maxY.get() - (Y_OFFSET[n] as f32 * f) as f64);
    this.m_maxZ.set(block.m_maxZ.get() - (Z_OFFSET[n] as f32 * f) as f64);
}

pub fn getCollisionShape(
    this: &Block,
    world: &Rc<World>,
    x: i32,
    y: i32,
    z: i32,
    movedBlock: i32,
    progress: f32,
    facing: i32,
) -> Option<Rc<AxisAlignedBB>>
{
    if movedBlock == 0 || movedBlock == this.m_blockID
    {
        return None;
    }
    let axisAlignedBB: Rc<AxisAlignedBB> =
        Block::byId(movedBlock).orNpe().getCollisionBoundingBoxFromPool(world, x, y, z)?;
    let n: usize = facing as usize;
    axisAlignedBB.m_minX.set(axisAlignedBB.m_minX.get() - (X_OFFSET[n] as f32 * progress) as f64);
    axisAlignedBB.m_maxX.set(axisAlignedBB.m_maxX.get() - (X_OFFSET[n] as f32 * progress) as f64);
    axisAlignedBB.m_minY.set(axisAlignedBB.m_minY.get() - (Y_OFFSET[n] as f32 * progress) as f64);
    axisAlignedBB.m_maxY.set(axisAlignedBB.m_maxY.get() - (Y_OFFSET[n] as f32 * progress) as f64);
    axisAlignedBB.m_minZ.set(axisAlignedBB.m_minZ.get() - (Z_OFFSET[n] as f32 * progress) as f64);
    axisAlignedBB.m_maxZ.set(axisAlignedBB.m_maxZ.get() - (Z_OFFSET[n] as f32 * progress) as f64);
    Some(axisAlignedBB)
}

pub const CLASS: BlockClass = BlockClass {
    m_name: "BlockPistonMoving",
    m_parent: Some(&block_container::CLASS),
    getBlockEntity: |_| jvm::throwable::npe(),
    onBlockAdded: |_, _, _, _, _| {},
    onBlockRemoval: |this, world, x, y, z| match world
        .getBlockTileEntity(x, y, z)
        .filter(|tileEntity| tileEntity.instanceOf("TileEntityPiston"))
    {
        Some(tileEntity) => tile_entity_piston::finish(&tileEntity),
        None => (block_container::CLASS.onBlockRemoval)(this, world, x, y, z),
    },
    canPlaceBlockAt: |_, _, _, _, _| false,
    canPlaceBlockOnSide: |_, _, _, _, _, _| false,
    getRenderType: |_| -1,
    isOpaqueCube: |_| false,
    renderAsNormalBlock: |_| false,
    blockActivated: |_, world, x, y, z, _| {
        if !world.m_multiplayerWorld.get() && world.getBlockTileEntity(x, y, z).is_none()
        {
            world.setBlockWithNotify(x, y, z, 0);
            return true;
        }
        false
    },
    idDropped: |_, _, _| 0,
    dropBlockAsItemWithChance,
    onNeighborBlockChange: |_, world, x, y, z, _| {
        if world.m_multiplayerWorld.get() || world.getBlockTileEntity(x, y, z).is_none()
        {}
    },
    getCollisionBoundingBoxFromPool,
    setBlockBoundsBasedOnState,
    ..block_container::CLASS
};

pub fn create(id: i32) -> Rc<Block>
{
    block_container::construct(&CLASS, Box::new(()), id, Material::PISTON()).setHardness(-1.0)
}
