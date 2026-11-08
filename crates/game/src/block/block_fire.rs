use crate::block::{BLOCK, Block, BlockClass, Material};
use crate::world::{IBlockAccess, World};
use jvm::random::Random;
use std::cell::RefCell;
use std::rc::Rc;

pub struct FireExt
{
    m_chanceToEncourageFire: RefCell<[i32; 256]>,
    m_abilityToCatchFire: RefCell<[i32; 256]>,
}

fn ext(this: &Block) -> &FireExt { this.ext::<FireExt>() }

fn setBurnRate(this: &Block, block: i32, flammability: i32, burnChance: i32)
{
    ext(this).m_chanceToEncourageFire.borrow_mut()[block as usize] = flammability;
    ext(this).m_abilityToCatchFire.borrow_mut()[block as usize] = burnChance;
}

fn initializeBlock(this: &Rc<Block>)
{
    setBurnRate(this, Block::planks().m_blockID, 5, 20);
    setBurnRate(this, Block::fence().m_blockID, 5, 20);
    setBurnRate(this, Block::stairCompactPlanks().m_blockID, 5, 20);
    setBurnRate(this, Block::wood().m_blockID, 5, 5);
    setBurnRate(this, Block::leaves().m_blockID, 30, 60);
    setBurnRate(this, Block::bookShelf().m_blockID, 30, 20);
    setBurnRate(this, Block::tnt().m_blockID, 15, 100);
    setBurnRate(this, Block::tallGrass().m_blockID, 60, 100);
    setBurnRate(this, Block::cloth().m_blockID, 30, 60);
}

fn updateTick(this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32, random: &Random)
{
    let bl: bool = world.getBlockId(x, y - 1, z) == Block::netherrack().m_blockID;
    if !this.canPlaceBlockAt(world, x, y, z)
    {
        world.setBlockWithNotify(x, y, z, 0);
    }
    if !bl
        && world.isRaining()
        && (world.canBlockBeRainedOn(x, y, z)
            || world.canBlockBeRainedOn(x - 1, y, z)
            || world.canBlockBeRainedOn(x + 1, y, z)
            || world.canBlockBeRainedOn(x, y, z - 1)
            || world.canBlockBeRainedOn(x, y, z + 1))
    {
        world.setBlockWithNotify(x, y, z, 0);
        return;
    }

    let n: i32 = world.getBlockMetadata(x, y, z);
    if n < 15
    {
        world.setBlockMetadata(x, y, z, n + random.nextIntBounded(3) / 2);
    }
    world.scheduleBlockUpdate(x, y, z, this.m_blockID, this.tickRate());
    if !bl && !hasFlammableNeighbor(this, world, x, y, z)
    {
        if !world.isBlockNormalCube(x, y - 1, z) || n > 3
        {
            world.setBlockWithNotify(x, y, z, 0);
        }
        return;
    }
    if !bl && !canBlockCatchFire(this, &**world, x, y - 1, z) && n == 15 && random.nextIntBounded(4) == 0
    {
        world.setBlockWithNotify(x, y, z, 0);
        return;
    }

    tryToCatchBlockOnFire(this, world, x + 1, y, z, 300, random, n);
    tryToCatchBlockOnFire(this, world, x - 1, y, z, 300, random, n);
    tryToCatchBlockOnFire(this, world, x, y - 1, z, 250, random, n);
    tryToCatchBlockOnFire(this, world, x, y + 1, z, 250, random, n);
    tryToCatchBlockOnFire(this, world, x, y, z - 1, 300, random, n);
    tryToCatchBlockOnFire(this, world, x, y, z + 1, 300, random, n);
    for i in (x - 1)..=(x + 1)
    {
        for j in (z - 1)..=(z + 1)
        {
            for k in (y - 1)..=(y + 4)
            {
                if i == x && k == y && j == z
                {
                    continue;
                }
                let mut n4: i32 = 100;
                if k > y + 1
                {
                    n4 += (k - (y + 1)) * 100;
                }
                let n3: i32 = getChanceOfNeighborsEncouragingFire(this, world, i, k, j);
                if n3 <= 0
                {
                    continue;
                }
                let n2: i32 = (n3 + 40) / (n + 30);
                if n2 <= 0
                    || random.nextIntBounded(n4) > n2
                    || world.isRaining() && world.canBlockBeRainedOn(i, k, j)
                    || world.canBlockBeRainedOn(i - 1, k, z)
                    || world.canBlockBeRainedOn(i + 1, k, j)
                    || world.canBlockBeRainedOn(i, k, j - 1)
                    || world.canBlockBeRainedOn(i, k, j + 1)
                {
                    continue;
                }
                let mut n5: i32 = n + random.nextIntBounded(5) / 4;
                if n5 > 15
                {
                    n5 = 15;
                }
                world.setBlockAndMetadataWithNotify(i, k, j, this.m_blockID, n5);
            }
        }
    }
}

fn tryToCatchBlockOnFire(
    this: &Rc<Block>,
    world: &Rc<World>,
    x: i32,
    y: i32,
    z: i32,
    chance: i32,
    random: &Random,
    metadata: i32,
)
{
    let i: i32 = ext(this).m_abilityToCatchFire.borrow()[world.getBlockId(x, y, z) as usize];
    if random.nextIntBounded(chance) >= i
    {
        return;
    }
    let flag: bool = world.getBlockId(x, y, z) == Block::tnt().m_blockID;
    if random.nextIntBounded(metadata + 10) < 5 && !world.canBlockBeRainedOn(x, y, z)
    {
        let mut j: i32 = metadata + random.nextIntBounded(5) / 4;
        if j > 15
        {
            j = 15;
        }
        world.setBlockAndMetadataWithNotify(x, y, z, this.m_blockID, j);
    }
    else
    {
        world.setBlockWithNotify(x, y, z, 0);
    }
    if flag
    {
        Block::tnt().onBlockDestroyedByPlayer(world, x, y, z, 1);
    }
}

fn hasFlammableNeighbor(this: &Block, world: &Rc<World>, x: i32, y: i32, z: i32) -> bool
{
    let access: &dyn IBlockAccess = &**world;
    canBlockCatchFire(this, access, x + 1, y, z)
        || canBlockCatchFire(this, access, x - 1, y, z)
        || canBlockCatchFire(this, access, x, y - 1, z)
        || canBlockCatchFire(this, access, x, y + 1, z)
        || canBlockCatchFire(this, access, x, y, z - 1)
        || canBlockCatchFire(this, access, x, y, z + 1)
}

fn getChanceOfNeighborsEncouragingFire(this: &Block, world: &Rc<World>, x: i32, y: i32, z: i32) -> i32
{
    if !world.isAirBlock(x, y, z)
    {
        return 0;
    }
    let mut n: i32 = 0;
    n = getChanceToEncourageFire(this, world, x + 1, y, z, n);
    n = getChanceToEncourageFire(this, world, x - 1, y, z, n);
    n = getChanceToEncourageFire(this, world, x, y - 1, z, n);
    n = getChanceToEncourageFire(this, world, x, y + 1, z, n);
    n = getChanceToEncourageFire(this, world, x, y, z - 1, n);
    n = getChanceToEncourageFire(this, world, x, y, z + 1, n);
    n
}

pub fn canBlockCatchFire(this: &Block, world: &dyn IBlockAccess, x: i32, y: i32, z: i32) -> bool
{
    ext(this).m_chanceToEncourageFire.borrow()[world.getBlockId(x, y, z) as usize] > 0
}

pub fn getChanceToEncourageFire(this: &Block, world: &World, x: i32, y: i32, z: i32, min: i32) -> i32
{
    let n: i32 = ext(this).m_chanceToEncourageFire.borrow()[world.getBlockId(x, y, z) as usize];
    if n > min { n } else { min }
}

fn randomDisplayTick(_this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32, random: &Random)
{
    let fire: Rc<Block> = Block::fire();
    let access: &dyn IBlockAccess = &**world;
    let spawn =
        |fx: f32, fy: f32, fz: f32| world.spawnParticle("largesmoke", fx as f64, fy as f64, fz as f64, 0.0, 0.0, 0.0);
    if random.nextIntBounded(24) == 0
    {
        let volume: f32 = 1.0 + random.nextFloat();
        let pitch: f32 = random.nextFloat() * 0.7 + 0.3;
        world.playSoundEffect(
            (x as f32 + 0.5) as f64,
            (y as f32 + 0.5) as f64,
            (z as f32 + 0.5) as f64,
            "fire.fire",
            volume,
            pitch,
        );
    }

    if world.isBlockNormalCube(x, y - 1, z) || canBlockCatchFire(&fire, access, x, y - 1, z)
    {
        for _ in 0..3
        {
            let f4: f32 = x as f32 + random.nextFloat();
            let f5: f32 = y as f32 + random.nextFloat() * 0.5 + 0.5;
            let f6: f32 = z as f32 + random.nextFloat();
            spawn(f4, f5, f6);
        }
        return;
    }

    if canBlockCatchFire(&fire, access, x - 1, y, z)
    {
        for _ in 0..2
        {
            let f3: f32 = x as f32 + random.nextFloat() * 0.1;
            let f2: f32 = y as f32 + random.nextFloat();
            let f: f32 = z as f32 + random.nextFloat();
            spawn(f3, f2, f);
        }
    }
    if canBlockCatchFire(&fire, access, x + 1, y, z)
    {
        for _ in 0..2
        {
            let f3: f32 = (x + 1) as f32 - random.nextFloat() * 0.1;
            let f2: f32 = y as f32 + random.nextFloat();
            let f: f32 = z as f32 + random.nextFloat();
            spawn(f3, f2, f);
        }
    }
    if canBlockCatchFire(&fire, access, x, y, z - 1)
    {
        for _ in 0..2
        {
            let f3: f32 = x as f32 + random.nextFloat();
            let f2: f32 = y as f32 + random.nextFloat();
            let f: f32 = z as f32 + random.nextFloat() * 0.1;
            spawn(f3, f2, f);
        }
    }
    if canBlockCatchFire(&fire, access, x, y, z + 1)
    {
        for _ in 0..2
        {
            let f3: f32 = x as f32 + random.nextFloat();
            let f2: f32 = y as f32 + random.nextFloat();
            let f: f32 = (z + 1) as f32 - random.nextFloat() * 0.1;
            spawn(f3, f2, f);
        }
    }
    if !canBlockCatchFire(&fire, access, x, y + 1, z)
    {
        return;
    }
    for _ in 0..2
    {
        let f3: f32 = x as f32 + random.nextFloat();
        let f2: f32 = (y + 1) as f32 - random.nextFloat() * 0.1;
        let f: f32 = z as f32 + random.nextFloat();
        spawn(f3, f2, f);
    }
}

pub const CLASS: BlockClass = BlockClass {
    m_name: "BlockFire",
    m_parent: Some(&BLOCK),
    initializeBlock,
    getCollisionBoundingBoxFromPool: |_, _, _, _, _| None,
    isOpaqueCube: |_| false,
    renderAsNormalBlock: |_| false,
    getRenderType: |_| 3,
    quantityDropped: |_, _| 0,
    tickRate: |_| 40,
    updateTick,
    isCollidable: |_| false,
    canPlaceBlockAt: |this, world, x, y, z| {
        world.isBlockNormalCube(x, y - 1, z) || hasFlammableNeighbor(this, world, x, y, z)
    },
    onNeighborBlockChange: |this, world, x, y, z, _| {
        if !world.isBlockNormalCube(x, y - 1, z) && !hasFlammableNeighbor(this, world, x, y, z)
        {
            world.setBlockWithNotify(x, y, z, 0);
        }
    },
    onBlockAdded: |this, world, x, y, z| {
        if world.getBlockId(x, y - 1, z) == Block::obsidian().m_blockID
            && crate::block::block_portal::tryToCreatePortal(world, x, y, z)
        {
            return;
        }
        if !world.isBlockNormalCube(x, y - 1, z) && !hasFlammableNeighbor(this, world, x, y, z)
        {
            world.setBlockWithNotify(x, y, z, 0);
            return;
        }
        world.scheduleBlockUpdate(x, y, z, this.m_blockID, this.tickRate());
    },
    randomDisplayTick,
    ..BLOCK
};

pub fn create(id: i32, sprite: i32) -> Rc<Block>
{
    let ext: FireExt =
        FireExt { m_chanceToEncourageFire: RefCell::new([0; 256]), m_abilityToCatchFire: RefCell::new([0; 256]) };
    Block::constructWithSprite(&CLASS, Box::new(ext), id, sprite, Material::fire()).setTickOnLoad(true)
}
