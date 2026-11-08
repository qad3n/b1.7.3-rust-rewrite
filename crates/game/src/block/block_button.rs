use crate::block::{BLOCK, Block, BlockClass, Material};
use crate::entity::Entity;
use crate::world::{IBlockAccess, World};
use jvm::random::Random;
use std::rc::Rc;

fn canPlaceBlockOnSide(_this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32, face: i32) -> bool
{
    if face == 2 && world.isBlockNormalCube(x, y, z + 1)
    {
        return true;
    }
    if face == 3 && world.isBlockNormalCube(x, y, z - 1)
    {
        return true;
    }
    if face == 4 && world.isBlockNormalCube(x + 1, y, z)
    {
        return true;
    }
    face == 5 && world.isBlockNormalCube(x - 1, y, z)
}

fn canPlaceBlockAt(_this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32) -> bool
{
    if world.isBlockNormalCube(x - 1, y, z)
    {
        return true;
    }
    if world.isBlockNormalCube(x + 1, y, z)
    {
        return true;
    }
    if world.isBlockNormalCube(x, y, z - 1)
    {
        return true;
    }
    world.isBlockNormalCube(x, y, z + 1)
}

fn onBlockPlaced(_this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32, face: i32)
{
    let mut i: i32 = world.getBlockMetadata(x, y, z);
    let j: i32 = i & 8;
    i &= 7;
    if face == 2 && world.isBlockNormalCube(x, y, z + 1)
    {
        i = 4;
    }
    else if face == 3 && world.isBlockNormalCube(x, y, z - 1)
    {
        i = 3;
    }
    else if face == 4 && world.isBlockNormalCube(x + 1, y, z)
    {
        i = 2;
    }
    else if face == 5 && world.isBlockNormalCube(x - 1, y, z)
    {
        i = 1;
    }
    else
    {
        i = getOrientation(world, x, y, z);
    }
    world.setBlockMetadataWithNotify(x, y, z, i + j);
}

fn getOrientation(world: &Rc<World>, x: i32, y: i32, z: i32) -> i32
{
    if world.isBlockNormalCube(x - 1, y, z)
    {
        return 1;
    }
    if world.isBlockNormalCube(x + 1, y, z)
    {
        return 2;
    }
    if world.isBlockNormalCube(x, y, z - 1)
    {
        return 3;
    }
    if world.isBlockNormalCube(x, y, z + 1)
    {
        return 4;
    }
    1
}

fn onNeighborBlockChange(this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32, _neighborBlock: i32)
{
    if canSurviveOrBreak(this, world, x, y, z)
    {
        let n: i32 = world.getBlockMetadata(x, y, z) & 7;
        let mut bl: bool = false;
        if !world.isBlockNormalCube(x - 1, y, z) && n == 1
        {
            bl = true;
        }
        if !world.isBlockNormalCube(x + 1, y, z) && n == 2
        {
            bl = true;
        }
        if !world.isBlockNormalCube(x, y, z - 1) && n == 3
        {
            bl = true;
        }
        if !world.isBlockNormalCube(x, y, z + 1) && n == 4
        {
            bl = true;
        }
        if bl
        {
            this.dropBlockAsItem(world, x, y, z, world.getBlockMetadata(x, y, z));
            world.setBlockWithNotify(x, y, z, 0);
        }
    }
}

fn canSurviveOrBreak(this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32) -> bool
{
    if !this.canPlaceBlockAt(world, x, y, z)
    {
        this.dropBlockAsItem(world, x, y, z, world.getBlockMetadata(x, y, z));
        world.setBlockWithNotify(x, y, z, 0);
        return false;
    }
    true
}

fn setBlockBoundsBasedOnState(this: &Rc<Block>, world: &dyn IBlockAccess, x: i32, y: i32, z: i32)
{
    let n: i32 = world.getBlockMetadata(x, y, z);
    let n2: i32 = n & 7;
    let bl: bool = (n & 8) > 0;
    let f: f32 = 0.375;
    let f2: f32 = 0.625;
    let f3: f32 = 0.1875;
    let mut f4: f32 = 0.125;
    if bl
    {
        f4 = 0.0625;
    }
    if n2 == 1
    {
        this.setBlockBounds(0.0, f, 0.5 - f3, f4, f2, 0.5 + f3);
    }
    else if n2 == 2
    {
        this.setBlockBounds(1.0 - f4, f, 0.5 - f3, 1.0, f2, 0.5 + f3);
    }
    else if n2 == 3
    {
        this.setBlockBounds(0.5 - f3, f, 0.0, 0.5 + f3, f2, f4);
    }
    else if n2 == 4
    {
        this.setBlockBounds(0.5 - f3, f, 1.0 - f4, 0.5 + f3, f2, 1.0);
    }
}

fn notifyAttached(world: &Rc<World>, x: i32, y: i32, z: i32, n2: i32, id: i32)
{
    if n2 == 1
    {
        world.notifyBlocksOfNeighborChange(x - 1, y, z, id);
    }
    else if n2 == 2
    {
        world.notifyBlocksOfNeighborChange(x + 1, y, z, id);
    }
    else if n2 == 3
    {
        world.notifyBlocksOfNeighborChange(x, y, z - 1, id);
    }
    else if n2 == 4
    {
        world.notifyBlocksOfNeighborChange(x, y, z + 1, id);
    }
    else
    {
        world.notifyBlocksOfNeighborChange(x, y - 1, z, id);
    }
}

fn blockActivated(this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32, _player: &Rc<Entity>) -> bool
{
    let n: i32 = world.getBlockMetadata(x, y, z);
    let n2: i32 = n & 7;
    let n3: i32 = 8 - (n & 8);
    if n3 == 0
    {
        return true;
    }
    world.setBlockMetadataWithNotify(x, y, z, n2 + n3);
    world.markBlocksDirty(x, y, z, x, y, z);
    world.playSoundEffect(x as f64 + 0.5, y as f64 + 0.5, z as f64 + 0.5, "random.click", 0.3, 0.6);
    world.notifyBlocksOfNeighborChange(x, y, z, this.m_blockID);
    notifyAttached(world, x, y, z, n2, this.m_blockID);
    world.scheduleBlockUpdate(x, y, z, this.m_blockID, this.tickRate());
    true
}

fn onBlockRemoval(this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32)
{
    let n: i32 = world.getBlockMetadata(x, y, z);
    if (n & 8) > 0
    {
        world.notifyBlocksOfNeighborChange(x, y, z, this.m_blockID);
        let n2: i32 = n & 7;
        notifyAttached(world, x, y, z, n2, this.m_blockID);
    }
    (BLOCK.onBlockRemoval)(this, world, x, y, z);
}

fn isIndirectlyPoweringTo(_this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32, dir: i32) -> bool
{
    let n: i32 = world.getBlockMetadata(x, y, z);
    if (n & 8) == 0
    {
        return false;
    }
    let n2: i32 = n & 7;
    if n2 == 5 && dir == 1
    {
        return true;
    }
    if n2 == 4 && dir == 2
    {
        return true;
    }
    if n2 == 3 && dir == 3
    {
        return true;
    }
    if n2 == 2 && dir == 4
    {
        return true;
    }
    n2 == 1 && dir == 5
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
    world.setBlockMetadataWithNotify(x, y, z, n & 7);
    world.notifyBlocksOfNeighborChange(x, y, z, this.m_blockID);
    let n2: i32 = n & 7;
    notifyAttached(world, x, y, z, n2, this.m_blockID);
    world.playSoundEffect(x as f64 + 0.5, y as f64 + 0.5, z as f64 + 0.5, "random.click", 0.3, 0.5);
    world.markBlocksDirty(x, y, z, x, y, z);
}

pub const CLASS: BlockClass = BlockClass {
    m_name: "BlockButton",
    m_parent: Some(&BLOCK),
    getCollisionBoundingBoxFromPool: |_, _, _, _, _| None,
    tickRate: |_| 20,
    isOpaqueCube: |_| false,
    renderAsNormalBlock: |_| false,
    canPlaceBlockOnSide,
    canPlaceBlockAt,
    onBlockPlaced,
    onNeighborBlockChange,
    setBlockBoundsBasedOnState,
    onBlockClicked: |this, world, x, y, z, player| {
        this.blockActivated(world, x, y, z, player);
    },
    blockActivated,
    onBlockRemoval,
    isPoweringTo: |_, world, x, y, z, _| (world.getBlockMetadata(x, y, z) & 8) > 0,
    isIndirectlyPoweringTo,
    canProvidePower: |_| true,
    updateTick,
    setBlockBoundsForItemRender: |this| {
        let f: f32 = 0.1875;
        let f2: f32 = 0.125;
        let f3: f32 = 0.125;
        this.setBlockBounds(0.5 - f, 0.5 - f2, 0.5 - f3, 0.5 + f, 0.5 + f2, 0.5 + f3);
    },
    ..BLOCK
};

pub fn create(id: i32, sprite: i32) -> Rc<Block>
{
    Block::constructWithSprite(&CLASS, Box::new(()), id, sprite, Material::circuits()).setTickOnLoad(true)
}
