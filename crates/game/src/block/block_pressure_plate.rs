use crate::block::{BLOCK, Block, BlockClass, Material};
use crate::entity::Entity;
use crate::math::AxisAlignedBB;
use crate::world::{IBlockAccess, World};
use jvm::random::Random;
use jvm::throwable::OrNpe;
use std::rc::Rc;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum EnumMobType
{
    Everything,
    Mobs,
    Players,
}

pub struct PlateExt
{
    m_triggerMobType: EnumMobType,
}

fn onNeighborBlockChange(this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32, _neighborBlock: i32)
{
    let mut flag: bool = false;
    if !world.isBlockNormalCube(x, y - 1, z)
    {
        flag = true;
    }
    if flag
    {
        this.dropBlockAsItem(world, x, y, z, world.getBlockMetadata(x, y, z));
        world.setBlockWithNotify(x, y, z, 0);
    }
}

fn updateTick(this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32, _random: &Random)
{
    if world.m_multiplayerWorld.get()
    {
        return;
    }
    if world.getBlockMetadata(x, y, z) == 0
    {
        return;
    }
    setStateIfMobInteractsWithPlate(this, world, x, y, z);
}

fn onEntityCollidedWithBlock(this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32, _entity: &Rc<Entity>)
{
    if world.m_multiplayerWorld.get()
    {
        return;
    }
    if world.getBlockMetadata(x, y, z) == 1
    {
        return;
    }
    setStateIfMobInteractsWithPlate(this, world, x, y, z);
}

fn plateBounds(x: i32, y: i32, z: i32, f: f32) -> Rc<AxisAlignedBB>
{
    AxisAlignedBB::getBoundingBoxFromPool(
        (x as f32 + f) as f64,
        y as f64,
        (z as f32 + f) as f64,
        ((x + 1) as f32 - f) as f64,
        y as f64 + 0.25,
        ((z + 1) as f32 - f) as f64,
    )
}

fn setStateIfMobInteractsWithPlate(this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32)
{
    let flag: bool = world.getBlockMetadata(x, y, z) == 1;
    let mut flag1: bool = false;
    let f: f32 = 0.125;
    let mut list: Option<usize> = None;
    let rule: EnumMobType = this.ext::<PlateExt>().m_triggerMobType;
    if rule == EnumMobType::Everything
    {
        list = Some(world.getEntitiesWithinAABBExcludingEntity(None, &plateBounds(x, y, z, f)).size() as usize);
    }
    if rule == EnumMobType::Mobs
    {
        list = Some(world.getEntitiesWithinAABB("EntityLiving", &plateBounds(x, y, z, f)).len());
    }
    if rule == EnumMobType::Players
    {
        list = Some(world.getEntitiesWithinAABB("EntityPlayer", &plateBounds(x, y, z, f)).len());
    }
    if list.orNpe() > 0
    {
        flag1 = true;
    }
    if flag1 && !flag
    {
        world.setBlockMetadataWithNotify(x, y, z, 1);
        world.notifyBlocksOfNeighborChange(x, y, z, this.m_blockID);
        world.notifyBlocksOfNeighborChange(x, y - 1, z, this.m_blockID);
        world.markBlocksDirty(x, y, z, x, y, z);
        world.playSoundEffect(x as f64 + 0.5, y as f64 + 0.1, z as f64 + 0.5, "random.click", 0.3, 0.6);
    }
    if !flag1 && flag
    {
        world.setBlockMetadataWithNotify(x, y, z, 0);
        world.notifyBlocksOfNeighborChange(x, y, z, this.m_blockID);
        world.notifyBlocksOfNeighborChange(x, y - 1, z, this.m_blockID);
        world.markBlocksDirty(x, y, z, x, y, z);
        world.playSoundEffect(x as f64 + 0.5, y as f64 + 0.1, z as f64 + 0.5, "random.click", 0.3, 0.5);
    }
    if flag1
    {
        world.scheduleBlockUpdate(x, y, z, this.m_blockID, this.tickRate());
    }
}

fn onBlockRemoval(this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32)
{
    let i: i32 = world.getBlockMetadata(x, y, z);
    if i > 0
    {
        world.notifyBlocksOfNeighborChange(x, y, z, this.m_blockID);
        world.notifyBlocksOfNeighborChange(x, y - 1, z, this.m_blockID);
    }
    (BLOCK.onBlockRemoval)(this, world, x, y, z);
}

fn setBlockBoundsBasedOnState(this: &Rc<Block>, world: &dyn IBlockAccess, x: i32, y: i32, z: i32)
{
    let flag: bool = world.getBlockMetadata(x, y, z) == 1;
    let f: f32 = 0.0625;
    if flag
    {
        this.setBlockBounds(f, 0.0, f, 1.0 - f, 0.03125, 1.0 - f);
    }
    else
    {
        this.setBlockBounds(f, 0.0, f, 1.0 - f, 0.0625, 1.0 - f);
    }
}

pub const CLASS: BlockClass = BlockClass {
    m_name: "BlockPressurePlate",
    m_parent: Some(&BLOCK),
    tickRate: |_| 20,
    getCollisionBoundingBoxFromPool: |_, _, _, _, _| None,
    isOpaqueCube: |_| false,
    renderAsNormalBlock: |_| false,
    canPlaceBlockAt: |_, world, x, y, z| world.isBlockNormalCube(x, y - 1, z),
    onBlockAdded: |_, _, _, _, _| {},
    onNeighborBlockChange,
    updateTick,
    onEntityCollidedWithBlock,
    onBlockRemoval,
    setBlockBoundsBasedOnState,
    isPoweringTo: |_, world, x, y, z, _| world.getBlockMetadata(x, y, z) > 0,
    isIndirectlyPoweringTo: |_, world, x, y, z, dir| {
        if world.getBlockMetadata(x, y, z) == 0
        {
            false
        }
        else
        {
            dir == 1
        }
    },
    canProvidePower: |_| true,
    setBlockBoundsForItemRender: |this| {
        let f: f32 = 0.5;
        let f1: f32 = 0.125;
        let f2: f32 = 0.5;
        this.setBlockBounds(0.5 - f, 0.5 - f1, 0.5 - f2, 0.5 + f, 0.5 + f1, 0.5 + f2);
    },
    getMobilityFlag: |_| 1,
    ..BLOCK
};

pub fn create(id: i32, sprite: i32, rule: EnumMobType, material: Rc<Material>) -> Rc<Block>
{
    let block: Rc<Block> =
        Block::constructWithSprite(&CLASS, Box::new(PlateExt { m_triggerMobType: rule }), id, sprite, material)
            .setTickOnLoad(true);
    let f: f32 = 0.0625;
    block.setBlockBounds(f, 0.0, f, 1.0 - f, 0.03125, 1.0 - f);
    block
}
