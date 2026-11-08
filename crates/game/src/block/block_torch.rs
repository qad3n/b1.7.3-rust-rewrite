use crate::block::{BLOCK, Block, BlockClass, Material};
use crate::math::{MovingObjectPosition, Vec3D};
use crate::world::World;
use jvm::random::Random;
use std::rc::Rc;

fn canSitOnTop(world: &Rc<World>, x: i32, y: i32, z: i32) -> bool
{
    world.isBlockNormalCube(x, y, z) || world.getBlockId(x, y, z) == Block::fence().m_blockID
}

fn canPlaceBlockAt(_this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32) -> bool
{
    world.isBlockNormalCube(x - 1, y, z)
        || world.isBlockNormalCube(x + 1, y, z)
        || world.isBlockNormalCube(x, y, z - 1)
        || world.isBlockNormalCube(x, y, z + 1)
        || canSitOnTop(world, x, y - 1, z)
}

fn onBlockPlaced(_this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32, face: i32)
{
    let mut n: i32 = world.getBlockMetadata(x, y, z);
    if face == 1 && canSitOnTop(world, x, y - 1, z)
    {
        n = 5;
    }
    if face == 2 && world.isBlockNormalCube(x, y, z + 1)
    {
        n = 4;
    }
    if face == 3 && world.isBlockNormalCube(x, y, z - 1)
    {
        n = 3;
    }
    if face == 4 && world.isBlockNormalCube(x + 1, y, z)
    {
        n = 2;
    }
    if face == 5 && world.isBlockNormalCube(x - 1, y, z)
    {
        n = 1;
    }
    world.setBlockMetadataWithNotify(x, y, z, n);
}

pub fn onBlockAdded(this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32)
{
    if world.isBlockNormalCube(x - 1, y, z)
    {
        world.setBlockMetadataWithNotify(x, y, z, 1);
    }
    else if world.isBlockNormalCube(x + 1, y, z)
    {
        world.setBlockMetadataWithNotify(x, y, z, 2);
    }
    else if world.isBlockNormalCube(x, y, z - 1)
    {
        world.setBlockMetadataWithNotify(x, y, z, 3);
    }
    else if world.isBlockNormalCube(x, y, z + 1)
    {
        world.setBlockMetadataWithNotify(x, y, z, 4);
    }
    else if canSitOnTop(world, x, y - 1, z)
    {
        world.setBlockMetadataWithNotify(x, y, z, 5);
    }
    dropTorchIfCantStay(this, world, x, y, z);
}

pub fn onNeighborBlockChange(this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32, _neighborBlock: i32)
{
    if !dropTorchIfCantStay(this, world, x, y, z)
    {
        return;
    }
    let n: i32 = world.getBlockMetadata(x, y, z);
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
    if !canSitOnTop(world, x, y - 1, z) && n == 5
    {
        bl = true;
    }
    if bl
    {
        this.dropBlockAsItem(world, x, y, z, world.getBlockMetadata(x, y, z));
        world.setBlockWithNotify(x, y, z, 0);
    }
}

fn dropTorchIfCantStay(this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32) -> bool
{
    if !this.canPlaceBlockAt(world, x, y, z)
    {
        this.dropBlockAsItem(world, x, y, z, world.getBlockMetadata(x, y, z));
        world.setBlockWithNotify(x, y, z, 0);
        return false;
    }
    true
}

fn collisionRayTrace(
    this: &Rc<Block>,
    world: &Rc<World>,
    x: i32,
    y: i32,
    z: i32,
    from: &Vec3D,
    to: &Vec3D,
) -> Option<MovingObjectPosition>
{
    let n: i32 = world.getBlockMetadata(x, y, z) & 7;
    let mut f: f32 = 0.15;
    match n
    {
        1 => this.setBlockBounds(0.0, 0.2, 0.5 - f, f * 2.0, 0.8, 0.5 + f),
        2 => this.setBlockBounds(1.0 - f * 2.0, 0.2, 0.5 - f, 1.0, 0.8, 0.5 + f),
        3 => this.setBlockBounds(0.5 - f, 0.2, 0.0, 0.5 + f, 0.8, f * 2.0),
        4 => this.setBlockBounds(0.5 - f, 0.2, 1.0 - f * 2.0, 0.5 + f, 0.8, 1.0),
        _ =>
        {
            f = 0.1;
            this.setBlockBounds(0.5 - f, 0.0, 0.5 - f, 0.5 + f, 0.6, 0.5 + f);
        }
    }
    (BLOCK.collisionRayTrace)(this, world, x, y, z, from, to)
}

fn randomDisplayTick(_this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32, _random: &Random)
{
    let n: i32 = world.getBlockMetadata(x, y, z);
    let d: f64 = (x as f32 + 0.5) as f64;
    let d2: f64 = (y as f32 + 0.7) as f64;
    let d3: f64 = (z as f32 + 0.5) as f64;
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
    world.spawnParticle("smoke", px, py, pz, 0.0, 0.0, 0.0);
    world.spawnParticle("flame", px, py, pz, 0.0, 0.0, 0.0);
}

pub const CLASS: BlockClass = BlockClass {
    m_name: "BlockTorch",
    m_parent: Some(&BLOCK),
    getCollisionBoundingBoxFromPool: |_, _, _, _, _| None,
    isOpaqueCube: |_| false,
    renderAsNormalBlock: |_| false,
    getRenderType: |_| 2,
    canPlaceBlockAt,
    onBlockPlaced,
    updateTick: |this, world, x, y, z, random| {
        (BLOCK.updateTick)(this, world, x, y, z, random);
        if world.getBlockMetadata(x, y, z) == 0
        {
            this.onBlockAdded(world, x, y, z);
        }
    },
    onBlockAdded,
    onNeighborBlockChange,
    collisionRayTrace,
    randomDisplayTick,
    ..BLOCK
};

pub fn construct(class: &'static BlockClass, ext: Box<dyn std::any::Any>, id: i32, sprite: i32) -> Rc<Block>
{
    Block::constructWithSprite(class, ext, id, sprite, Material::circuits()).setTickOnLoad(true)
}

pub fn create(id: i32, sprite: i32) -> Rc<Block> { construct(&CLASS, Box::new(()), id, sprite) }
