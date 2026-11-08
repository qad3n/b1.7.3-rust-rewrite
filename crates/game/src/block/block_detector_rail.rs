use crate::block::{Block, BlockClass, block_rail};
use crate::entity::Entity;
use crate::math::AxisAlignedBB;
use crate::world::World;
use jvm::random::Random;
use std::rc::Rc;

fn onEntityCollidedWithBlock(this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32, _entity: &Rc<Entity>)
{
    if world.m_multiplayerWorld.get()
    {
        return;
    }
    let n: i32 = world.getBlockMetadata(x, y, z);
    if (n & 8) != 0
    {
        return;
    }
    setStateIfMinecartInteractsWithRail(this, world, x, y, z, n);
}

fn updateTick(this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32, _random: &Random)
{
    if world.m_multiplayerWorld.get()
    {
        return;
    }
    let n: i32 = world.getBlockMetadata(x, y, z);
    if (n & 8) == 0
    {
        return;
    }
    setStateIfMinecartInteractsWithRail(this, world, x, y, z, n);
}

fn setStateIfMinecartInteractsWithRail(this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32, metadata: i32)
{
    let bl: bool = (metadata & 8) != 0;
    let mut bl2: bool = false;
    let f: f32 = 0.125;
    let list: Vec<Rc<Entity>> = world.getEntitiesWithinAABB(
        "EntityMinecart",
        &AxisAlignedBB::getBoundingBoxFromPool(
            (x as f32 + f) as f64,
            y as f64,
            (z as f32 + f) as f64,
            ((x + 1) as f32 - f) as f64,
            y as f64 + 0.25,
            ((z + 1) as f32 - f) as f64,
        ),
    );
    if !list.is_empty()
    {
        bl2 = true;
    }
    if bl2 && !bl
    {
        world.setBlockMetadataWithNotify(x, y, z, metadata | 8);
        world.notifyBlocksOfNeighborChange(x, y, z, this.m_blockID);
        world.notifyBlocksOfNeighborChange(x, y - 1, z, this.m_blockID);
        world.markBlocksDirty(x, y, z, x, y, z);
    }
    if !bl2 && bl
    {
        world.setBlockMetadataWithNotify(x, y, z, metadata & 7);
        world.notifyBlocksOfNeighborChange(x, y, z, this.m_blockID);
        world.notifyBlocksOfNeighborChange(x, y - 1, z, this.m_blockID);
        world.markBlocksDirty(x, y, z, x, y, z);
    }
    if bl2
    {
        world.scheduleBlockUpdate(x, y, z, this.m_blockID, this.tickRate());
    }
}

pub const CLASS: BlockClass = BlockClass {
    m_name: "BlockDetectorRail",
    m_parent: Some(&block_rail::CLASS),
    tickRate: |_| 20,
    canProvidePower: |_| true,
    onEntityCollidedWithBlock,
    updateTick,
    isPoweringTo: |_, world, x, y, z, _| (world.getBlockMetadata(x, y, z) & 8) != 0,
    isIndirectlyPoweringTo: |_, world, x, y, z, dir| {
        if (world.getBlockMetadata(x, y, z) & 8) == 0
        {
            return false;
        }
        dir == 1
    },
    ..block_rail::CLASS
};

pub fn create(id: i32, sprite: i32) -> Rc<Block> { block_rail::construct(&CLASS, id, sprite, true).setTickOnLoad(true) }
