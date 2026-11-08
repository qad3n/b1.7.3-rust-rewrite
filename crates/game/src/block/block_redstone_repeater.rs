use crate::block::{BLOCK, Block, BlockClass, Material};
use crate::item::Item;
use crate::math::math_helper;
use crate::world::{IBlockAccess, World};
use jvm::random::Random;
use std::cell::Cell;
use std::rc::Rc;

pub const RANDOM_DISPLAY: [f64; 4] = [-0.0625, 0.0625, 0.1875, 0.3125];
const DELAY_SETTINGS: [i32; 4] = [1, 2, 3, 4];

#[derive(Default)]
pub struct RepeaterExt
{
    m_isRepeaterPowered: Cell<bool>,
}

fn powered(this: &Block) -> bool { this.ext::<RepeaterExt>().m_isRepeaterPowered.get() }

fn shouldBePowered(world: &Rc<World>, x: i32, y: i32, z: i32, metadata: i32) -> bool
{
    let wire: i32 = Block::redstoneWire().m_blockID;
    match metadata & 3
    {
        0 =>
        {
            world.isBlockIndirectlyProvidingPowerTo(x, y, z + 1, 3)
                || world.getBlockId(x, y, z + 1) == wire && world.getBlockMetadata(x, y, z + 1) > 0
        }
        2 =>
        {
            world.isBlockIndirectlyProvidingPowerTo(x, y, z - 1, 2)
                || world.getBlockId(x, y, z - 1) == wire && world.getBlockMetadata(x, y, z - 1) > 0
        }
        3 =>
        {
            world.isBlockIndirectlyProvidingPowerTo(x + 1, y, z, 5)
                || world.getBlockId(x + 1, y, z) == wire && world.getBlockMetadata(x + 1, y, z) > 0
        }
        1 =>
        {
            world.isBlockIndirectlyProvidingPowerTo(x - 1, y, z, 4)
                || world.getBlockId(x - 1, y, z) == wire && world.getBlockMetadata(x - 1, y, z) > 0
        }
        _ => false,
    }
}

fn updateTick(this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32, _random: &Random)
{
    let n: i32 = world.getBlockMetadata(x, y, z);
    let bl: bool = shouldBePowered(world, x, y, z, n);
    if powered(this) && !bl
    {
        world.setBlockAndMetadataWithNotify(x, y, z, Block::redstoneRepeaterIdle().m_blockID, n);
    }
    else if !powered(this)
    {
        world.setBlockAndMetadataWithNotify(x, y, z, Block::redstoneRepeaterActive().m_blockID, n);
        if !bl
        {
            let n2: i32 = (n & 0xC) >> 2;
            world.scheduleBlockUpdate(
                x,
                y,
                z,
                Block::redstoneRepeaterActive().m_blockID,
                DELAY_SETTINGS[n2 as usize] * 2,
            );
        }
    }
}

fn isPoweringTo(this: &Rc<Block>, world: &dyn IBlockAccess, x: i32, y: i32, z: i32, dir: i32) -> bool
{
    if !powered(this)
    {
        return false;
    }
    let n: i32 = world.getBlockMetadata(x, y, z) & 3;
    n == 0 && dir == 3 || n == 1 && dir == 4 || n == 2 && dir == 2 || n == 3 && dir == 5
}

fn onNeighborBlockChange(this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32, _neighborBlock: i32)
{
    if !this.canBlockStay(world, x, y, z)
    {
        this.dropBlockAsItem(world, x, y, z, world.getBlockMetadata(x, y, z));
        world.setBlockWithNotify(x, y, z, 0);
        return;
    }
    let n: i32 = world.getBlockMetadata(x, y, z);
    let bl: bool = shouldBePowered(world, x, y, z, n);
    let n2: i32 = (n & 0xC) >> 2;
    if powered(this) && !bl || !powered(this) && bl
    {
        world.scheduleBlockUpdate(x, y, z, this.m_blockID, DELAY_SETTINGS[n2 as usize] * 2);
    }
}

fn randomDisplayTick(this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32, random: &Random)
{
    if !powered(this)
    {
        return;
    }
    let n: i32 = world.getBlockMetadata(x, y, z);
    let d: f64 = (x as f32 + 0.5) as f64 + (random.nextFloat() - 0.5) as f64 * 0.2;
    let d2: f64 = (y as f32 + 0.4) as f64 + (random.nextFloat() - 0.5) as f64 * 0.2;
    let d3: f64 = (z as f32 + 0.5) as f64 + (random.nextFloat() - 0.5) as f64 * 0.2;
    let mut d4: f64 = 0.0;
    let mut d5: f64 = 0.0;
    if random.nextIntBounded(2) == 0
    {
        match n & 3
        {
            0 => d5 = -0.3125,
            2 => d5 = 0.3125,
            3 => d4 = -0.3125,
            _ => d4 = 0.3125,
        }
    }
    else
    {
        let n2: usize = ((n & 0xC) >> 2) as usize;
        match n & 3
        {
            0 => d5 = RANDOM_DISPLAY[n2],
            2 => d5 = -RANDOM_DISPLAY[n2],
            3 => d4 = RANDOM_DISPLAY[n2],
            _ => d4 = -RANDOM_DISPLAY[n2],
        }
    }
    world.spawnParticle("reddust", d + d4, d2, d3 + d5, 0.0, 0.0, 0.0);
}

pub const CLASS: BlockClass = BlockClass {
    m_name: "BlockRedstoneRepeater",
    m_parent: Some(&BLOCK),
    renderAsNormalBlock: |_| false,
    canPlaceBlockAt: |this, world, x, y, z| {
        world.isBlockNormalCube(x, y - 1, z) && (BLOCK.canPlaceBlockAt)(this, world, x, y, z)
    },
    canBlockStay: |this, world, x, y, z| {
        world.isBlockNormalCube(x, y - 1, z) && (BLOCK.canBlockStay)(this, world, x, y, z)
    },
    updateTick,
    getBlockTextureFromSideAndMetadata: |this, face, _| match face
    {
        0 =>
        {
            if powered(this)
            {
                99
            }
            else
            {
                115
            }
        }
        1 =>
        {
            if powered(this)
            {
                147
            }
            else
            {
                131
            }
        }
        _ => 5,
    },
    shouldSideBeRendered: |_, _, _, _, _, face| face != 0 && face != 1,
    getRenderType: |_| 15,
    getBlockTextureFromSide: |this, face| this.getBlockTextureFromSideAndMetadata(face, 0),
    isIndirectlyPoweringTo: |this, world, x, y, z, dir| this.isPoweringTo(&**world, x, y, z, dir),
    isPoweringTo,
    onNeighborBlockChange,
    blockActivated: |_, world, x, y, z, _| {
        let n: i32 = world.getBlockMetadata(x, y, z);
        let mut n2: i32 = (n & 0xC) >> 2;
        n2 = n2 + 1 << 2 & 0xC;
        world.setBlockMetadataWithNotify(x, y, z, n2 | n & 3);
        true
    },
    canProvidePower: |_| false,
    onBlockPlacedBy: |this, world, x, y, z, entity| {
        let n: i32 = ((math_helper::floor_double((entity.m_rotationYaw.get() * 4.0 / 360.0) as f64 + 0.5) & 3) + 2) % 4;
        world.setBlockMetadataWithNotify(x, y, z, n);
        if shouldBePowered(world, x, y, z, n)
        {
            world.scheduleBlockUpdate(x, y, z, this.m_blockID, 1);
        }
    },
    onBlockAdded: |this, world, x, y, z| {
        let id: i32 = this.m_blockID;
        world.notifyBlocksOfNeighborChange(x + 1, y, z, id);
        world.notifyBlocksOfNeighborChange(x - 1, y, z, id);
        world.notifyBlocksOfNeighborChange(x, y, z + 1, id);
        world.notifyBlocksOfNeighborChange(x, y, z - 1, id);
        world.notifyBlocksOfNeighborChange(x, y - 1, z, id);
        world.notifyBlocksOfNeighborChange(x, y + 1, z, id);
    },
    isOpaqueCube: |_| false,
    idDropped: |_, _, _| Item::redstoneRepeater().m_shiftedIndex,
    randomDisplayTick,
    ..BLOCK
};

pub fn create(id: i32, powered: bool) -> Rc<Block>
{
    let block: Rc<Block> =
        Block::constructWithSprite(&CLASS, Box::new(RepeaterExt::default()), id, 6, Material::circuits());
    block.ext::<RepeaterExt>().m_isRepeaterPowered.set(powered);
    block.setBlockBounds(0.0, 0.0, 0.0, 1.0, 0.125, 1.0);
    block
}
