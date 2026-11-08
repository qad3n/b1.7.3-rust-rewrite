use crate::block::{BLOCK, Block, BlockClass, Material};
use crate::entity::Entity;
use crate::math::Vec3D;
use crate::world::{IBlockAccess, World};
use jvm::random::Random;
use jvm::throwable::OrNpe;
use std::any::Any;
use std::rc::Rc;

pub fn getPercentAir(state: i32) -> f32
{
    let state: i32 = if state >= 8 { 0 } else { state };
    (state + 1) as f32 / 9.0
}

fn is(material: &Rc<Material>, other: &Rc<Material>) -> bool { Rc::ptr_eq(material, other) }

pub fn getFlowDecay(this: &Block, world: &World, x: i32, y: i32, z: i32) -> i32
{
    if !is(&world.getBlockMaterial(x, y, z), &this.m_blockMaterial)
    {
        return -1;
    }
    world.getBlockMetadata(x, y, z)
}

pub fn getEffectiveFlowDecay(this: &Block, world: &dyn IBlockAccess, x: i32, y: i32, z: i32) -> i32
{
    if !is(&world.getBlockMaterial(x, y, z), &this.m_blockMaterial)
    {
        return -1;
    }
    let n: i32 = world.getBlockMetadata(x, y, z);
    if n >= 8 { 0 } else { n }
}

fn getIsBlockSolid(this: &Rc<Block>, world: &dyn IBlockAccess, x: i32, y: i32, z: i32, face: i32) -> bool
{
    let material: Rc<Material> = world.getBlockMaterial(x, y, z);
    if is(&material, &this.m_blockMaterial) || is(&material, &Material::ice())
    {
        return false;
    }
    if face == 1
    {
        return true;
    }
    (BLOCK.getIsBlockSolid)(this, world, x, y, z, face)
}

fn shouldSideBeRendered(this: &Rc<Block>, world: &dyn IBlockAccess, x: i32, y: i32, z: i32, face: i32) -> bool
{
    let material: Rc<Material> = world.getBlockMaterial(x, y, z);
    if is(&material, &this.m_blockMaterial) || is(&material, &Material::ice())
    {
        return false;
    }
    if face == 1
    {
        return true;
    }
    (BLOCK.shouldSideBeRendered)(this, world, x, y, z, face)
}

fn getFlowVector(this: &Rc<Block>, world: &dyn IBlockAccess, x: i32, y: i32, z: i32) -> Rc<Vec3D>
{
    let mut vec3D: Rc<Vec3D> = Vec3D::createVector(0.0, 0.0, 0.0);
    let n2: i32 = getEffectiveFlowDecay(this, world, x, y, z);
    for n in 0..4
    {
        let mut n5: i32 = x;
        let n6: i32 = y;
        let mut n7: i32 = z;
        if n == 0
        {
            n5 -= 1;
        }
        if n == 1
        {
            n7 -= 1;
        }
        if n == 2
        {
            n5 += 1;
        }
        if n == 3
        {
            n7 += 1;
        }

        let mut n4: i32 = getEffectiveFlowDecay(this, world, n5, n6, n7);
        if n4 < 0
        {
            if world.getBlockMaterial(n5, n6, n7).getIsSolid()
            {
                continue;
            }
            n4 = getEffectiveFlowDecay(this, world, n5, n6 - 1, n7);
            if n4 < 0
            {
                continue;
            }
            let n3: i32 = n4 - (n2 - 8);
            vec3D = vec3D.addVector(((n5 - x) * n3) as f64, ((n6 - y) * n3) as f64, ((n7 - z) * n3) as f64);
            continue;
        }

        let n3: i32 = n4 - n2;
        vec3D = vec3D.addVector(((n5 - x) * n3) as f64, ((n6 - y) * n3) as f64, ((n7 - z) * n3) as f64);
    }

    if world.getBlockMetadata(x, y, z) >= 8
    {
        let checks: [(i32, i32, i32, i32); 8] = [
            (x, y, z - 1, 2),
            (x, y, z + 1, 3),
            (x - 1, y, z, 4),
            (x + 1, y, z, 5),
            (x, y + 1, z - 1, 2),
            (x, y + 1, z + 1, 3),
            (x - 1, y + 1, z, 4),
            (x + 1, y + 1, z, 5),
        ];
        let mut n: bool = false;
        for (cx, cy, cz, face) in checks
        {
            if n || this.getIsBlockSolid(world, cx, cy, cz, face)
            {
                n = true;
            }
        }
        if n
        {
            vec3D = vec3D.normalize().addVector(0.0, -6.0, 0.0);
        }
    }
    vec3D.normalize()
}

fn velocityToAddToEntity(
    this: &Rc<Block>,
    world: &Rc<World>,
    x: i32,
    y: i32,
    z: i32,
    _entity: &Rc<Entity>,
    velocity: &Vec3D,
)
{
    let vec3D: Rc<Vec3D> = getFlowVector(this, &**world, x, y, z);
    velocity.m_xCoord.set(velocity.x() + vec3D.x());
    velocity.m_yCoord.set(velocity.y() + vec3D.y());
    velocity.m_zCoord.set(velocity.z() + vec3D.z());
}

fn tickRate(this: &Rc<Block>) -> i32
{
    if is(&this.m_blockMaterial, &Material::water())
    {
        return 5;
    }
    if is(&this.m_blockMaterial, &Material::lava())
    {
        return 30;
    }
    0
}

fn getBlockBrightness(_this: &Rc<Block>, world: &dyn IBlockAccess, x: i32, y: i32, z: i32) -> f32
{
    let f: f32 = world.getLightBrightness(x, y, z);
    let f1: f32 = world.getLightBrightness(x, y + 1, z);
    if f > f1 { f } else { f1 }
}

fn randomDisplayTick(this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32, random: &Random)
{
    if is(&this.m_blockMaterial, &Material::water()) && random.nextIntBounded(64) == 0
    {
        let i: i32 = world.getBlockMetadata(x, y, z);
        if i > 0 && i < 8
        {
            let volume: f32 = random.nextFloat() * 0.25 + 0.75;
            let pitch: f32 = random.nextFloat() * 1.0 + 0.5;
            world.playSoundEffect(
                (x as f32 + 0.5) as f64,
                (y as f32 + 0.5) as f64,
                (z as f32 + 0.5) as f64,
                "liquid.water",
                volume,
                pitch,
            );
        }
    }

    if is(&this.m_blockMaterial, &Material::lava())
        && is(&world.getBlockMaterial(x, y + 1, z), &Material::air())
        && !world.isBlockOpaqueCube(x, y + 1, z)
        && random.nextIntBounded(100) == 0
    {
        let d2: f64 = (x as f32 + random.nextFloat()) as f64;
        let d0: f64 = y as f64 + this.m_maxY.get();
        let d1: f64 = (z as f32 + random.nextFloat()) as f64;
        world.spawnParticle("lava", d2, d0, d1, 0.0, 0.0, 0.0);
    }
}

pub fn getFlowAngle(world: &dyn IBlockAccess, x: i32, y: i32, z: i32, material: &Rc<Material>) -> f64
{
    let mut vec3D: Option<Rc<Vec3D>> = None;
    if is(material, &Material::water())
    {
        vec3D = Some(getFlowVector(&Block::waterMoving(), world, x, y, z));
    }
    if is(material, &Material::lava())
    {
        vec3D = Some(getFlowVector(&Block::lavaMoving(), world, x, y, z));
    }
    let vec3D: Rc<Vec3D> = vec3D.orNpe();
    if vec3D.x() == 0.0 && vec3D.z() == 0.0
    {
        return -1000.0;
    }
    jvm::math::atan2(vec3D.z(), vec3D.x()) - 1.5707963267948966
}

fn checkForHarden(this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32)
{
    if world.getBlockId(x, y, z) != this.m_blockID || !is(&this.m_blockMaterial, &Material::lava())
    {
        return;
    }
    let water: Rc<Material> = Material::water();
    let mut bl: bool = false;
    for (cx, cy, cz) in [(x, y, z - 1), (x, y, z + 1), (x - 1, y, z), (x + 1, y, z), (x, y + 1, z)]
    {
        if bl || is(&world.getBlockMaterial(cx, cy, cz), &water)
        {
            bl = true;
        }
    }
    if bl
    {
        let n: i32 = world.getBlockMetadata(x, y, z);
        if n == 0
        {
            world.setBlockWithNotify(x, y, z, Block::obsidian().m_blockID);
        }
        else if n <= 4
        {
            world.setBlockWithNotify(x, y, z, Block::cobblestone().m_blockID);
        }
        triggerLavaMixEffects(world, x, y, z);
    }
}

pub fn triggerLavaMixEffects(world: &Rc<World>, x: i32, y: i32, z: i32)
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
    for _ in 0..8
    {
        let dx: f64 = x as f64 + jvm::math::random();
        let dz: f64 = z as f64 + jvm::math::random();
        world.spawnParticle("largesmoke", dx, y as f64 + 1.2, dz, 0.0, 0.0, 0.0);
    }
}

pub const CLASS: BlockClass = BlockClass {
    m_name: "BlockFluid",
    m_parent: Some(&BLOCK),
    colorMultiplier: |_, _, _, _, _| 0xFFFFFF,
    getBlockTextureFromSide: |this, face| {
        if face == 0 || face == 1
        {
            this.m_blockIndexInTexture.get()
        }
        else
        {
            this.m_blockIndexInTexture.get() + 1
        }
    },
    renderAsNormalBlock: |_| false,
    isOpaqueCube: |_| false,
    canCollideCheck: |_, metadata, allowLiquids| allowLiquids && metadata == 0,
    getIsBlockSolid,
    shouldSideBeRendered,
    getCollisionBoundingBoxFromPool: |_, _, _, _, _| None,
    getRenderType: |_| 4,
    idDropped: |_, _, _| 0,
    quantityDropped: |_, _| 0,
    velocityToAddToEntity,
    tickRate,
    getBlockBrightness,
    updateTick: |this, world, x, y, z, random| (BLOCK.updateTick)(this, world, x, y, z, random),
    getRenderBlockPass: |this| {
        if is(&this.m_blockMaterial, &Material::water())
        {
            1
        }
        else
        {
            0
        }
    },
    randomDisplayTick,
    onBlockAdded: checkForHarden,
    onNeighborBlockChange: |this, world, x, y, z, _| checkForHarden(this, world, x, y, z),
    ..BLOCK
};

pub fn construct(class: &'static BlockClass, ext: Box<dyn Any>, id: i32, material: Rc<Material>) -> Rc<Block>
{
    let sprite: i32 = (if is(&material, &Material::lava())
    {
        14
    }
    else
    {
        12
    }) * 16
        + 13;
    let block: Rc<Block> = Block::constructWithSprite(class, ext, id, sprite, material);
    let f: f32 = 0.0;
    let f2: f32 = 0.0;
    block.setBlockBounds(0.0 + f2, 0.0 + f, 0.0 + f2, 1.0 + f2, 1.0 + f, 1.0 + f2);
    block.setTickOnLoad(true)
}
