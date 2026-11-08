use crate::block::{Block, BlockClass, block_torch};
use crate::world::{IBlockAccess, World};
use jvm::array_list::JArrayList;
use jvm::random::Random;
use std::cell::Cell;
use std::rc::Rc;

#[derive(Clone)]
pub struct RedstoneUpdateInfo
{
    m_x: i32,
    m_y: i32,
    m_z: i32,
    m_updateTime: i64,
}

thread_local! {
    static TORCH_UPDATES: JArrayList<RedstoneUpdateInfo> = JArrayList::new();
}

#[derive(Default)]
pub struct TorchExt
{
    m_torchActive: Cell<bool>,
}

fn active(this: &Block) -> bool { this.ext::<TorchExt>().m_torchActive.get() }

fn checkForBurnout(world: &Rc<World>, x: i32, y: i32, z: i32, logToggle: bool) -> bool
{
    TORCH_UPDATES.with(|updates| {
        if logToggle
        {
            updates.add(RedstoneUpdateInfo { m_x: x, m_y: y, m_z: z, m_updateTime: world.getWorldTime() });
        }
        let mut i: i32 = 0;
        for j in 0..updates.size()
        {
            let info: RedstoneUpdateInfo = updates.get(j);
            if info.m_x == x && info.m_y == y && info.m_z == z
            {
                i += 1;
                if i >= 8
                {
                    return true;
                }
            }
        }
        false
    })
}

fn notifyAround(this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32)
{
    let id: i32 = this.m_blockID;
    world.notifyBlocksOfNeighborChange(x, y - 1, z, id);
    world.notifyBlocksOfNeighborChange(x, y + 1, z, id);
    world.notifyBlocksOfNeighborChange(x - 1, y, z, id);
    world.notifyBlocksOfNeighborChange(x + 1, y, z, id);
    world.notifyBlocksOfNeighborChange(x, y, z - 1, id);
    world.notifyBlocksOfNeighborChange(x, y, z + 1, id);
}

fn isPoweringTo(this: &Rc<Block>, world: &dyn IBlockAccess, x: i32, y: i32, z: i32, dir: i32) -> bool
{
    if !active(this)
    {
        return false;
    }
    let n: i32 = world.getBlockMetadata(x, y, z);
    if n == 5 && dir == 1 || n == 3 && dir == 3 || n == 4 && dir == 2 || n == 1 && dir == 5
    {
        return false;
    }
    n != 2 || dir != 4
}

fn hasNeighborSignal(world: &Rc<World>, x: i32, y: i32, z: i32) -> bool
{
    let n: i32 = world.getBlockMetadata(x, y, z);
    if n == 5 && world.isBlockIndirectlyProvidingPowerTo(x, y - 1, z, 0)
    {
        return true;
    }
    if n == 3 && world.isBlockIndirectlyProvidingPowerTo(x, y, z - 1, 2)
    {
        return true;
    }
    if n == 4 && world.isBlockIndirectlyProvidingPowerTo(x, y, z + 1, 3)
    {
        return true;
    }
    if n == 1 && world.isBlockIndirectlyProvidingPowerTo(x - 1, y, z, 4)
    {
        return true;
    }
    n == 2 && world.isBlockIndirectlyProvidingPowerTo(x + 1, y, z, 5)
}

fn updateTick(this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32, random: &Random)
{
    let bl: bool = hasNeighborSignal(world, x, y, z);
    TORCH_UPDATES.with(|updates| {
        while updates.size() > 0 && world.getWorldTime() - updates.get(0).m_updateTime > 100
        {
            updates.removeAt(0);
        }
    });

    if active(this)
    {
        if bl
        {
            world.setBlockAndMetadataWithNotify(
                x,
                y,
                z,
                Block::torchRedstoneIdle().m_blockID,
                world.getBlockMetadata(x, y, z),
            );
            if checkForBurnout(world, x, y, z, true)
            {
                let pitch: f32 = 2.6 + (world.m_rand.nextFloat() - world.m_rand.nextFloat()) * 0.8;
                world.playSoundEffect(
                    (x as f32 + 0.5) as f64,
                    (y as f32 + 0.5) as f64,
                    (z as f32 + 0.5) as f64,
                    "random.fizz",
                    0.5,
                    pitch,
                );
                for _ in 0..5
                {
                    let d: f64 = x as f64 + random.nextDouble() * 0.6 + 0.2;
                    let d2: f64 = y as f64 + random.nextDouble() * 0.6 + 0.2;
                    let d3: f64 = z as f64 + random.nextDouble() * 0.6 + 0.2;
                    world.spawnParticle("smoke", d, d2, d3, 0.0, 0.0, 0.0);
                }
            }
        }
    }
    else if !bl && !checkForBurnout(world, x, y, z, false)
    {
        world.setBlockAndMetadataWithNotify(
            x,
            y,
            z,
            Block::torchRedstoneActive().m_blockID,
            world.getBlockMetadata(x, y, z),
        );
    }
}

fn randomDisplayTick(this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32, random: &Random)
{
    if !active(this)
    {
        return;
    }
    let n: i32 = world.getBlockMetadata(x, y, z);
    let d: f64 = (x as f32 + 0.5) as f64 + (random.nextFloat() - 0.5) as f64 * 0.2;
    let d2: f64 = (y as f32 + 0.7) as f64 + (random.nextFloat() - 0.5) as f64 * 0.2;
    let d3: f64 = (z as f32 + 0.5) as f64 + (random.nextFloat() - 0.5) as f64 * 0.2;
    let d4: f64 = 0.22f32 as f64;
    let d5: f64 = 0.27f32 as f64;
    let (px, py, pz): (f64, f64, f64) = match n
    {
        1 => (d - d5, d2 + d4, d3),
        2 => (d + d5, d2 + d4, d3),
        3 => (d, d2 + d4, d3 - d5),
        4 => (d, d2 + d4, d3 + d5),
        _ => (d, d2, d3),
    };
    world.spawnParticle("reddust", px, py, pz, 0.0, 0.0, 0.0);
}

pub const CLASS: BlockClass = BlockClass {
    m_name: "BlockRedstoneTorch",
    m_parent: Some(&block_torch::CLASS),
    getBlockTextureFromSideAndMetadata: |this, face, metadata| {
        if face == 1
        {
            return Block::redstoneWire().getBlockTextureFromSideAndMetadata(face, metadata);
        }
        (block_torch::CLASS.getBlockTextureFromSideAndMetadata)(this, face, metadata)
    },
    tickRate: |_| 2,
    onBlockAdded: |this, world, x, y, z| {
        if world.getBlockMetadata(x, y, z) == 0
        {
            (block_torch::CLASS.onBlockAdded)(this, world, x, y, z);
        }
        if active(this)
        {
            notifyAround(this, world, x, y, z);
        }
    },
    onBlockRemoval: |this, world, x, y, z| {
        if active(this)
        {
            notifyAround(this, world, x, y, z);
        }
    },
    isPoweringTo,
    updateTick,
    onNeighborBlockChange: |this, world, x, y, z, neighborBlock| {
        (block_torch::CLASS.onNeighborBlockChange)(this, world, x, y, z, neighborBlock);
        world.scheduleBlockUpdate(x, y, z, this.m_blockID, this.tickRate());
    },
    isIndirectlyPoweringTo: |this, world, x, y, z, dir| {
        if dir == 0
        {
            this.isPoweringTo(&**world, x, y, z, dir)
        }
        else
        {
            false
        }
    },
    idDropped: |_, _, _| Block::torchRedstoneActive().m_blockID,
    canProvidePower: |_| true,
    randomDisplayTick,
    ..block_torch::CLASS
};

pub fn create(id: i32, sprite: i32, lit: bool) -> Rc<Block>
{
    let block: Rc<Block> = block_torch::construct(&CLASS, Box::new(TorchExt::default()), id, sprite);
    block.ext::<TorchExt>().m_torchActive.set(lit);
    block.setTickOnLoad(true)
}
