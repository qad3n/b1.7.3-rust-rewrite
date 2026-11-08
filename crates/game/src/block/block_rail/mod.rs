mod rail_logic;

use crate::block::{BLOCK, Block, BlockClass, Material};
use crate::world::{IBlockAccess, World};
use jvm::throwable::OrNpe;
pub use rail_logic::RailLogic;
use std::rc::Rc;

pub struct RailExt
{
    m_isPowered: bool,
}

pub fn isRailBlockAt(world: &Rc<World>, x: i32, y: i32, z: i32) -> bool
{
    let blockId: i32 = world.getBlockId(x, y, z);
    blockId == Block::rail().m_blockID
        || blockId == Block::railPowered().m_blockID
        || blockId == Block::railDetector().m_blockID
}

pub fn isRailBlock(block: i32) -> bool
{
    block == Block::rail().m_blockID
        || block == Block::railPowered().m_blockID
        || block == Block::railDetector().m_blockID
}

pub fn getIsPowered(this: &Block) -> bool { this.ext::<RailExt>().m_isPowered }

fn setBlockBoundsBasedOnState(this: &Rc<Block>, world: &dyn IBlockAccess, x: i32, y: i32, z: i32)
{
    let blockMetadata: i32 = world.getBlockMetadata(x, y, z);
    if (2..=5).contains(&blockMetadata)
    {
        this.setBlockBounds(0.0, 0.0, 0.0, 1.0, 0.625, 1.0);
    }
    else
    {
        this.setBlockBounds(0.0, 0.0, 0.0, 1.0, 0.125, 1.0);
    }
}

fn getBlockTextureFromSideAndMetadata(this: &Rc<Block>, _face: i32, metadata: i32) -> i32
{
    if getIsPowered(this)
    {
        if this.m_blockID == Block::railPowered().m_blockID && (metadata & 0x8) == 0x0
        {
            return this.m_blockIndexInTexture.get() - 16;
        }
    }
    else if metadata >= 6
    {
        return this.m_blockIndexInTexture.get() - 16;
    }
    this.m_blockIndexInTexture.get()
}

fn onNeighborBlockChange(this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32, neighborBlock: i32)
{
    if world.m_multiplayerWorld.get()
    {
        return;
    }
    let n: i32 = world.getBlockMetadata(x, y, z);
    let mut blockMetadata: i32 = n;
    if getIsPowered(this)
    {
        blockMetadata &= 0x7;
    }
    let mut b: bool = false;
    if !world.isBlockNormalCube(x, y - 1, z)
    {
        b = true;
    }
    if blockMetadata == 2 && !world.isBlockNormalCube(x + 1, y, z)
    {
        b = true;
    }
    if blockMetadata == 3 && !world.isBlockNormalCube(x - 1, y, z)
    {
        b = true;
    }
    if blockMetadata == 4 && !world.isBlockNormalCube(x, y, z - 1)
    {
        b = true;
    }
    if blockMetadata == 5 && !world.isBlockNormalCube(x, y, z + 1)
    {
        b = true;
    }
    if b
    {
        this.dropBlockAsItem(world, x, y, z, world.getBlockMetadata(x, y, z));
        world.setBlockWithNotify(x, y, z, 0);
    }
    else if this.m_blockID == Block::railPowered().m_blockID
    {
        let b2: bool = world.isBlockIndirectlyGettingPowered(x, y, z)
            || world.isBlockIndirectlyGettingPowered(x, y + 1, z)
            || isPoweredByConnectedRails(world, x, y, z, n, true, 0)
            || isPoweredByConnectedRails(world, x, y, z, n, false, 0);
        let mut b3: bool = false;
        if b2 && (n & 0x8) == 0x0
        {
            world.setBlockMetadataWithNotify(x, y, z, blockMetadata | 0x8);
            b3 = true;
        }
        else if !b2 && (n & 0x8) != 0x0
        {
            world.setBlockMetadataWithNotify(x, y, z, blockMetadata);
            b3 = true;
        }
        if b3
        {
            world.notifyBlocksOfNeighborChange(x, y - 1, z, this.m_blockID);
            if blockMetadata == 2 || blockMetadata == 3 || blockMetadata == 4 || blockMetadata == 5
            {
                world.notifyBlocksOfNeighborChange(x, y + 1, z, this.m_blockID);
            }
        }
    }
    else if neighborBlock > 0
        && Block::byId(neighborBlock).orNpe().canProvidePower()
        && !getIsPowered(this)
        && RailLogic::new(world, x, y, z).getAdjacentTracks() == 3
    {
        updateShape(world, x, y, z, false);
    }
}

fn updateShape(world: &Rc<World>, x: i32, y: i32, z: i32, force: bool)
{
    if world.m_multiplayerWorld.get()
    {
        return;
    }
    RailLogic::new(world, x, y, z).updateState(world.isBlockIndirectlyGettingPowered(x, y, z), force);
}

fn isPoweredByConnectedRails(
    world: &Rc<World>,
    mut x: i32,
    mut y: i32,
    mut z: i32,
    metadata: i32,
    towardsNegative: bool,
    depth: i32,
) -> bool
{
    if depth >= 8
    {
        return false;
    }
    let mut n: i32 = metadata & 0x7;
    let mut b: bool = true;
    match n
    {
        0 =>
        {
            if towardsNegative
            {
                z += 1;
            }
            else
            {
                z -= 1;
            }
        }
        1 =>
        {
            if towardsNegative
            {
                x -= 1;
            }
            else
            {
                x += 1;
            }
        }
        2 =>
        {
            if towardsNegative
            {
                x -= 1;
            }
            else
            {
                x += 1;
                y += 1;
                b = false;
            }
            n = 1;
        }
        3 =>
        {
            if towardsNegative
            {
                x -= 1;
                y += 1;
                b = false;
            }
            else
            {
                x += 1;
            }
            n = 1;
        }
        4 =>
        {
            if towardsNegative
            {
                z += 1;
            }
            else
            {
                z -= 1;
                y += 1;
                b = false;
            }
            n = 0;
        }
        5 =>
        {
            if towardsNegative
            {
                z += 1;
                y += 1;
                b = false;
            }
            else
            {
                z -= 1;
            }
            n = 0;
        }
        _ =>
        {}
    }
    isPoweredByRail(world, x, y, z, towardsNegative, depth, n)
        || (b && isPoweredByRail(world, x, y - 1, z, towardsNegative, depth, n))
}

fn isPoweredByRail(world: &Rc<World>, x: i32, y: i32, z: i32, towardsNegative: bool, depth: i32, shape: i32) -> bool
{
    let n: i32 = world.getBlockId(x, y, z);
    if n == Block::railPowered().m_blockID
    {
        let n2: i32 = world.getBlockMetadata(x, y, z);
        let n3: i32 = n2 & 7;
        if shape == 1 && (n3 == 0 || n3 == 4 || n3 == 5)
        {
            return false;
        }
        if shape == 0 && (n3 == 1 || n3 == 2 || n3 == 3)
        {
            return false;
        }
        if (n2 & 8) != 0
        {
            if world.isBlockIndirectlyGettingPowered(x, y, z) || world.isBlockIndirectlyGettingPowered(x, y + 1, z)
            {
                return true;
            }
            return isPoweredByConnectedRails(world, x, y, z, n2, towardsNegative, depth + 1);
        }
    }
    false
}

pub const CLASS: BlockClass = BlockClass {
    m_name: "BlockRail",
    m_parent: Some(&BLOCK),
    getCollisionBoundingBoxFromPool: |_, _, _, _, _| None,
    isOpaqueCube: |_| false,
    collisionRayTrace: |this, world, x, y, z, from, to| {
        this.setBlockBoundsBasedOnState(&**world, x, y, z);
        (BLOCK.collisionRayTrace)(this, world, x, y, z, from, to)
    },
    setBlockBoundsBasedOnState,
    getBlockTextureFromSideAndMetadata,
    renderAsNormalBlock: |_| false,
    getRenderType: |_| 9,
    quantityDropped: |_, _| 1,
    canPlaceBlockAt: |_, world, x, y, z| world.isBlockNormalCube(x, y - 1, z),
    onBlockAdded: |_, world, x, y, z| {
        if !world.m_multiplayerWorld.get()
        {
            updateShape(world, x, y, z, true);
        }
    },
    onNeighborBlockChange,
    getMobilityFlag: |_| 0,
    ..BLOCK
};

pub fn construct(class: &'static BlockClass, id: i32, sprite: i32, alwaysStraight: bool) -> Rc<Block>
{
    let block: Rc<Block> = Block::constructWithSprite(
        class,
        Box::new(RailExt { m_isPowered: alwaysStraight }),
        id,
        sprite,
        Material::circuits(),
    );
    block.setBlockBounds(0.0, 0.0, 0.0, 1.0, 0.125, 1.0);
    block
}

pub fn create(id: i32, sprite: i32, alwaysStraight: bool) -> Rc<Block> { construct(&CLASS, id, sprite, alwaysStraight) }
