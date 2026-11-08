use crate::block::{BLOCK, Block, BlockClass, Material, model_bed};
use crate::item::Item;
use crate::world::{IBlockAccess, World};
use common::world::ChunkPosition;
use jvm::field::Field;
use jvm::hash_map::JHashSet;
use jvm::random::Random;
use jvm::throwable::OrNpe;
use std::cell::Cell;
use std::rc::Rc;

pub struct WireExt
{
    m_wiresProvidePower: Cell<bool>,
    m_neighborsToUpdate: Field<JHashSet<ChunkPosition>>,
}

fn ext(this: &Block) -> &WireExt { this.ext::<WireExt>() }

fn updateAndPropagateCurrentStrength(this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32)
{
    doUpdatePower(this, world, x, y, z, x, y, z);
    let arrayList: Vec<ChunkPosition> = ext(this).m_neighborsToUpdate.with(JHashSet::items);
    ext(this).m_neighborsToUpdate.update(JHashSet::clear);
    for chunkPosition in arrayList
    {
        world.notifyBlocksOfNeighborChange(chunkPosition.m_x, chunkPosition.m_y, chunkPosition.m_z, this.m_blockID);
    }
}

fn doUpdatePower(this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32, sourceX: i32, sourceY: i32, sourceZ: i32)
{
    let i: i32 = world.getBlockMetadata(x, y, z);
    let mut j: i32 = 0;
    ext(this).m_wiresProvidePower.set(false);
    let flag: bool = world.isBlockIndirectlyGettingPowered(x, y, z);
    ext(this).m_wiresProvidePower.set(true);
    if flag
    {
        j = 15;
    }
    else
    {
        for k in 0..4
        {
            let mut l: i32 = x;
            let mut i1: i32 = z;
            match k
            {
                0 => l -= 1,
                1 => l += 1,
                2 => i1 -= 1,
                _ => i1 += 1,
            }
            if l != sourceX || y != sourceY || i1 != sourceZ
            {
                j = getMaxCurrentStrength(this, world, l, y, i1, j);
            }
            if world.isBlockNormalCube(l, y, i1) && !world.isBlockNormalCube(x, y + 1, z)
            {
                if l != sourceX || y + 1 != sourceY || i1 != sourceZ
                {
                    j = getMaxCurrentStrength(this, world, l, y + 1, i1, j);
                }
            }
            else if !world.isBlockNormalCube(l, y, i1) && (l != sourceX || y - 1 != sourceY || i1 != sourceZ)
            {
                j = getMaxCurrentStrength(this, world, l, y - 1, i1, j);
            }
        }
        if j > 0
        {
            j -= 1;
        }
        else
        {
            j = 0;
        }
    }

    if i == j
    {
        return;
    }

    world.m_editingBlocks.set(true);
    world.setBlockMetadataWithNotify(x, y, z, j);
    world.markBlocksDirty(x, y, z, x, y, z);
    world.m_editingBlocks.set(false);
    for l1 in 0..4
    {
        let mut i2: i32 = x;
        let mut j2: i32 = z;
        let mut j1: i32 = y - 1;
        match l1
        {
            0 => i2 -= 1,
            1 => i2 += 1,
            2 => j2 -= 1,
            _ => j2 += 1,
        }
        if world.isBlockNormalCube(i2, y, j2)
        {
            j1 += 2;
        }
        let mut k1: i32 = getMaxCurrentStrength(this, world, i2, y, j2, -1);
        j = world.getBlockMetadata(x, y, z);
        if j > 0
        {
            j -= 1;
        }
        if k1 >= 0 && k1 != j
        {
            doUpdatePower(this, world, i2, y, j2, x, y, z);
        }
        k1 = getMaxCurrentStrength(this, world, i2, j1, j2, -1);
        j = world.getBlockMetadata(x, y, z);
        if j > 0
        {
            j -= 1;
        }
        if k1 >= 0 && k1 != j
        {
            doUpdatePower(this, world, i2, j1, j2, x, y, z);
        }
    }

    if i == 0 || j == 0
    {
        let positions: [ChunkPosition; 7] = [
            ChunkPosition::new(x, y, z),
            ChunkPosition::new(x - 1, y, z),
            ChunkPosition::new(x + 1, y, z),
            ChunkPosition::new(x, y - 1, z),
            ChunkPosition::new(x, y + 1, z),
            ChunkPosition::new(x, y, z - 1),
            ChunkPosition::new(x, y, z + 1),
        ];
        ext(this).m_neighborsToUpdate.update(|set| {
            for position in positions
            {
                set.add(position);
            }
        });
    }
}

fn notifyWireNeighborsOfNeighborChange(this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32)
{
    if world.getBlockId(x, y, z) != this.m_blockID
    {
        return;
    }
    let id: i32 = this.m_blockID;
    world.notifyBlocksOfNeighborChange(x, y, z, id);
    world.notifyBlocksOfNeighborChange(x - 1, y, z, id);
    world.notifyBlocksOfNeighborChange(x + 1, y, z, id);
    world.notifyBlocksOfNeighborChange(x, y, z - 1, id);
    world.notifyBlocksOfNeighborChange(x, y, z + 1, id);
    world.notifyBlocksOfNeighborChange(x, y - 1, z, id);
    world.notifyBlocksOfNeighborChange(x, y + 1, z, id);
}

fn notifyDiagonals(this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32)
{
    notifyWireNeighborsOfNeighborChange(this, world, x - 1, y, z);
    notifyWireNeighborsOfNeighborChange(this, world, x + 1, y, z);
    notifyWireNeighborsOfNeighborChange(this, world, x, y, z - 1);
    notifyWireNeighborsOfNeighborChange(this, world, x, y, z + 1);
    for (dx, dz) in [(-1, 0), (1, 0), (0, -1), (0, 1)]
    {
        if world.isBlockNormalCube(x + dx, y, z + dz)
        {
            notifyWireNeighborsOfNeighborChange(this, world, x + dx, y + 1, z + dz);
        }
        else
        {
            notifyWireNeighborsOfNeighborChange(this, world, x + dx, y - 1, z + dz);
        }
    }
}

fn onBlockAdded(this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32)
{
    (BLOCK.onBlockAdded)(this, world, x, y, z);
    if world.m_multiplayerWorld.get()
    {
        return;
    }
    updateAndPropagateCurrentStrength(this, world, x, y, z);
    world.notifyBlocksOfNeighborChange(x, y + 1, z, this.m_blockID);
    world.notifyBlocksOfNeighborChange(x, y - 1, z, this.m_blockID);
    notifyDiagonals(this, world, x, y, z);
}

fn onBlockRemoval(this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32)
{
    (BLOCK.onBlockRemoval)(this, world, x, y, z);
    if world.m_multiplayerWorld.get()
    {
        return;
    }
    world.notifyBlocksOfNeighborChange(x, y + 1, z, this.m_blockID);
    world.notifyBlocksOfNeighborChange(x, y - 1, z, this.m_blockID);
    updateAndPropagateCurrentStrength(this, world, x, y, z);
    notifyDiagonals(this, world, x, y, z);
}

fn getMaxCurrentStrength(this: &Block, world: &World, x: i32, y: i32, z: i32, wirePower: i32) -> i32
{
    if world.getBlockId(x, y, z) != this.m_blockID
    {
        return wirePower;
    }
    let n: i32 = world.getBlockMetadata(x, y, z);
    if n > wirePower { n } else { wirePower }
}

fn onNeighborBlockChange(this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32, neighborBlock: i32)
{
    if world.m_multiplayerWorld.get()
    {
        return;
    }
    let n: i32 = world.getBlockMetadata(x, y, z);
    let bl: bool = this.canPlaceBlockAt(world, x, y, z);
    if !bl
    {
        this.dropBlockAsItem(world, x, y, z, n);
        world.setBlockWithNotify(x, y, z, 0);
    }
    else
    {
        updateAndPropagateCurrentStrength(this, world, x, y, z);
    }
    (BLOCK.onNeighborBlockChange)(this, world, x, y, z, neighborBlock);
}

fn isPoweringTo(this: &Rc<Block>, world: &dyn IBlockAccess, x: i32, y: i32, z: i32, dir: i32) -> bool
{
    if !ext(this).m_wiresProvidePower.get() || world.getBlockMetadata(x, y, z) == 0
    {
        return false;
    }
    if dir == 1
    {
        return true;
    }
    let mut bl2: bool = isPowerProviderOrWire(world, x - 1, y, z, 1)
        || !world.isBlockNormalCube(x - 1, y, z) && isPowerProviderOrWire(world, x - 1, y - 1, z, -1);
    let mut bl3: bool = isPowerProviderOrWire(world, x + 1, y, z, 3)
        || !world.isBlockNormalCube(x + 1, y, z) && isPowerProviderOrWire(world, x + 1, y - 1, z, -1);
    let mut bl4: bool = isPowerProviderOrWire(world, x, y, z - 1, 2)
        || !world.isBlockNormalCube(x, y, z - 1) && isPowerProviderOrWire(world, x, y - 1, z - 1, -1);
    let mut bl: bool = isPowerProviderOrWire(world, x, y, z + 1, 0)
        || !world.isBlockNormalCube(x, y, z + 1) && isPowerProviderOrWire(world, x, y - 1, z + 1, -1);
    if !world.isBlockNormalCube(x, y + 1, z)
    {
        if world.isBlockNormalCube(x - 1, y, z) && isPowerProviderOrWire(world, x - 1, y + 1, z, -1)
        {
            bl2 = true;
        }
        if world.isBlockNormalCube(x + 1, y, z) && isPowerProviderOrWire(world, x + 1, y + 1, z, -1)
        {
            bl3 = true;
        }
        if world.isBlockNormalCube(x, y, z - 1) && isPowerProviderOrWire(world, x, y + 1, z - 1, -1)
        {
            bl4 = true;
        }
        if world.isBlockNormalCube(x, y, z + 1) && isPowerProviderOrWire(world, x, y + 1, z + 1, -1)
        {
            bl = true;
        }
    }
    if !(bl4 || bl3 || bl2 || bl || dir < 2 || dir > 5)
    {
        return true;
    }
    if dir == 2 && bl4 && !bl2 && !bl3
    {
        return true;
    }
    if dir == 3 && bl && !bl2 && !bl3
    {
        return true;
    }
    if dir == 4 && bl2 && !bl4 && !bl
    {
        return true;
    }
    dir == 5 && bl3 && !bl4 && !bl
}

fn randomDisplayTick(_this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32, random: &Random)
{
    let n: i32 = world.getBlockMetadata(x, y, z);
    if n <= 0
    {
        return;
    }
    let d: f64 = x as f64 + 0.5 + (random.nextFloat() as f64 - 0.5) * 0.2;
    let d2: f64 = (y as f32 + 0.0625) as f64;
    let d3: f64 = z as f64 + 0.5 + (random.nextFloat() as f64 - 0.5) * 0.2;
    let f: f32 = n as f32 / 15.0;
    let mut f2: f32 = f * 0.6 + 0.4;
    if n == 0
    {
        f2 = 0.0;
    }
    let mut f3: f32 = f * f * 0.7 - 0.5;
    let mut f4: f32 = f * f * 0.6 - 0.7;
    if f3 < 0.0
    {
        f3 = 0.0;
    }
    if f4 < 0.0
    {
        f4 = 0.0;
    }
    world.spawnParticle("reddust", d, d2, d3, f2 as f64, f3 as f64, f4 as f64);
}

pub fn isPowerProviderOrWire(world: &dyn IBlockAccess, x: i32, y: i32, z: i32, side: i32) -> bool
{
    let n: i32 = world.getBlockId(x, y, z);
    if n == Block::redstoneWire().m_blockID
    {
        return true;
    }
    if n == 0
    {
        return false;
    }
    if Block::byId(n).orNpe().canProvidePower()
    {
        return true;
    }
    if n == Block::redstoneRepeaterIdle().m_blockID || n == Block::redstoneRepeaterActive().m_blockID
    {
        let n2: i32 = world.getBlockMetadata(x, y, z);
        return side == model_bed::OPPOSITE[(n2 & 3) as usize];
    }
    false
}

pub const CLASS: BlockClass = BlockClass {
    m_name: "BlockRedstoneWire",
    m_parent: Some(&BLOCK),
    getBlockTextureFromSideAndMetadata: |this, _, _| this.m_blockIndexInTexture.get(),
    getCollisionBoundingBoxFromPool: |_, _, _, _, _| None,
    isOpaqueCube: |_| false,
    renderAsNormalBlock: |_| false,
    getRenderType: |_| 5,
    colorMultiplier: |_, _, _, _, _| 0x800000,
    canPlaceBlockAt: |_, world, x, y, z| world.isBlockNormalCube(x, y - 1, z),
    onBlockAdded,
    onBlockRemoval,
    onNeighborBlockChange,
    idDropped: |_, _, _| Item::redstone().m_shiftedIndex,
    isIndirectlyPoweringTo: |this, world, x, y, z, dir| {
        ext(this).m_wiresProvidePower.get() && this.isPoweringTo(&**world, x, y, z, dir)
    },
    isPoweringTo,
    canProvidePower: |this| ext(this).m_wiresProvidePower.get(),
    randomDisplayTick,
    ..BLOCK
};

pub fn create(id: i32, sprite: i32) -> Rc<Block>
{
    let ext: WireExt =
        WireExt { m_wiresProvidePower: Cell::new(false), m_neighborsToUpdate: Field::new(JHashSet::new()) };
    let block: Rc<Block> = Block::constructWithSprite(&CLASS, Box::new(ext), id, sprite, Material::circuits());
    block.ext::<WireExt>().m_wiresProvidePower.set(true);
    block.setBlockBounds(0.0, 0.0, 0.0, 1.0, 0.0625, 1.0);
    block
}
