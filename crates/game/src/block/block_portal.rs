use crate::block::{Block, BlockClass, Material, block_breakable};
use crate::world::{IBlockAccess, World};
use jvm::random::Random;
use std::rc::Rc;

pub fn tryToCreatePortal(world: &Rc<World>, x: i32, y: i32, z: i32) -> bool
{
    let (mut x, mut z): (i32, i32) = (x, z);
    let obsidian: i32 = Block::obsidian().m_blockID;
    let mut n3: i32 = 0;
    let mut n4: i32 = 0;
    if world.getBlockId(x - 1, y, z) == obsidian || world.getBlockId(x + 1, y, z) == obsidian
    {
        n3 = 1;
    }
    if world.getBlockId(x, y, z - 1) == obsidian || world.getBlockId(x, y, z + 1) == obsidian
    {
        n4 = 1;
    }
    if n3 == n4
    {
        return false;
    }
    if world.getBlockId(x - n3, y, z - n4) == 0
    {
        x -= n3;
        z -= n4;
    }

    for n2 in -1..=2
    {
        for n in -1..=3
        {
            let bl: bool = n2 == -1 || n2 == 2 || n == -1 || n == 3;
            if !(n2 != -1 && n2 != 2 || n != -1 && n != 3)
            {
                continue;
            }
            let n5: i32 = world.getBlockId(x + n3 * n2, y + n, z + n4 * n2);
            if !(if bl
            {
                n5 != obsidian
            }
            else
            {
                n5 != 0 && n5 != Block::fire().m_blockID
            })
            {
                continue;
            }
            return false;
        }
    }

    world.m_editingBlocks.set(true);
    for n2 in 0..2
    {
        for n in 0..3
        {
            world.setBlockWithNotify(x + n3 * n2, y + n, z + n4 * n2, Block::portal().m_blockID);
        }
    }
    world.m_editingBlocks.set(false);
    true
}

fn onNeighborBlockChange(this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32, _neighborBlock: i32)
{
    let id: i32 = this.m_blockID;
    let obsidian: i32 = Block::obsidian().m_blockID;
    let mut n2: i32 = 0;
    let mut n3: i32 = 1;
    if world.getBlockId(x - 1, y, z) == id || world.getBlockId(x + 1, y, z) == id
    {
        n2 = 1;
        n3 = 0;
    }
    let mut n4: i32 = y;
    while world.getBlockId(x, n4 - 1, z) == id
    {
        n4 -= 1;
    }
    if world.getBlockId(x, n4 - 1, z) != obsidian
    {
        world.setBlockWithNotify(x, y, z, 0);
        return;
    }
    let mut n: i32 = 1;
    while n < 4 && world.getBlockId(x, n4 + n, z) == id
    {
        n += 1;
    }
    if n != 3 || world.getBlockId(x, n4 + n, z) != obsidian
    {
        world.setBlockWithNotify(x, y, z, 0);
        return;
    }
    let bl2: bool = world.getBlockId(x - 1, y, z) == id || world.getBlockId(x + 1, y, z) == id;
    let bl: bool = world.getBlockId(x, y, z - 1) == id || world.getBlockId(x, y, z + 1) == id;
    if bl2 && bl
    {
        world.setBlockWithNotify(x, y, z, 0);
        return;
    }
    if !(world.getBlockId(x + n2, y, z + n3) == obsidian && world.getBlockId(x - n2, y, z - n3) == id
        || world.getBlockId(x - n2, y, z - n3) == obsidian && world.getBlockId(x + n2, y, z + n3) == id)
    {
        world.setBlockWithNotify(x, y, z, 0);
    }
}

fn shouldSideBeRendered(this: &Rc<Block>, world: &dyn IBlockAccess, x: i32, y: i32, z: i32, face: i32) -> bool
{
    let id: i32 = this.m_blockID;
    if world.getBlockId(x, y, z) == id
    {
        return false;
    }
    let bl2: bool = world.getBlockId(x - 1, y, z) == id && world.getBlockId(x - 2, y, z) != id;
    let bl3: bool = world.getBlockId(x + 1, y, z) == id && world.getBlockId(x + 2, y, z) != id;
    let bl4: bool = world.getBlockId(x, y, z - 1) == id && world.getBlockId(x, y, z - 2) != id;
    let bl5: bool = world.getBlockId(x, y, z + 1) == id && world.getBlockId(x, y, z + 2) != id;
    let bl6: bool = bl2 || bl3;
    let bl: bool = bl4 || bl5;
    if bl6 && face == 4 || bl6 && face == 5 || bl && face == 2
    {
        return true;
    }
    bl && face == 3
}

fn randomDisplayTick(this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32, random: &Random)
{
    if random.nextIntBounded(100) == 0
    {
        let pitch: f32 = random.nextFloat() * 0.4 + 0.8;
        world.playSoundEffect(x as f64 + 0.5, y as f64 + 0.5, z as f64 + 0.5, "portal.portal", 1.0, pitch);
    }
    for _ in 0..4
    {
        let mut d: f64 = (x as f32 + random.nextFloat()) as f64;
        let d2: f64 = (y as f32 + random.nextFloat()) as f64;
        let mut d3: f64 = (z as f32 + random.nextFloat()) as f64;
        let n: i32 = random.nextIntBounded(2) * 2 - 1;
        let mut d4: f64 = (random.nextFloat() as f64 - 0.5) * 0.5;
        let d5: f64 = (random.nextFloat() as f64 - 0.5) * 0.5;
        let mut d6: f64 = (random.nextFloat() as f64 - 0.5) * 0.5;
        if world.getBlockId(x - 1, y, z) == this.m_blockID || world.getBlockId(x + 1, y, z) == this.m_blockID
        {
            d3 = z as f64 + 0.5 + 0.25 * n as f64;
            d6 = (random.nextFloat() * 2.0 * n as f32) as f64;
        }
        else
        {
            d = x as f64 + 0.5 + 0.25 * n as f64;
            d4 = (random.nextFloat() * 2.0 * n as f32) as f64;
        }
        world.spawnParticle("portal", d, d2, d3, d4, d5, d6);
    }
}

pub const CLASS: BlockClass = BlockClass {
    m_name: "BlockPortal",
    m_parent: Some(&block_breakable::CLASS),
    getCollisionBoundingBoxFromPool: |_, _, _, _, _| None,
    setBlockBoundsBasedOnState: |this, world, x, y, z| {
        if world.getBlockId(x - 1, y, z) == this.m_blockID || world.getBlockId(x + 1, y, z) == this.m_blockID
        {
            let f: f32 = 0.5;
            let f2: f32 = 0.125;
            this.setBlockBounds(0.5 - f, 0.0, 0.5 - f2, 0.5 + f, 1.0, 0.5 + f2);
        }
        else
        {
            let f: f32 = 0.125;
            let f3: f32 = 0.5;
            this.setBlockBounds(0.5 - f, 0.0, 0.5 - f3, 0.5 + f, 1.0, 0.5 + f3);
        }
    },
    isOpaqueCube: |_| false,
    renderAsNormalBlock: |_| false,
    onNeighborBlockChange,
    shouldSideBeRendered,
    quantityDropped: |_, _| 0,
    getRenderBlockPass: |_| 1,
    onEntityCollidedWithBlock: |_, _, _, _, _, entity| {
        if entity.m_ridingEntity.with(Option::is_none) && entity.m_riddenByEntity.with(Option::is_none)
        {
            entity.setInPortal();
        }
    },
    randomDisplayTick,
    ..block_breakable::CLASS
};

pub fn create(id: i32, sprite: i32) -> Rc<Block>
{
    block_breakable::construct(&CLASS, id, sprite, Material::portal(), false)
}
