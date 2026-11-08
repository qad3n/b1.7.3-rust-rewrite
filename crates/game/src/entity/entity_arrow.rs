use crate::block::Block;
use crate::entity::{ENTITY, Entity, EntityClass, EntityExt, entity_player};
use crate::item::{Item, ItemStack};
use crate::math::math_helper::{cos, sin, sqrt_double};
use crate::math::{AxisAlignedBB, MovingObjectPosition, Vec3D};
use crate::world::World;
use common::nbt::NBTTagCompound;
use jvm::field::Field;
use jvm::throwable::OrNpe;
use std::cell::Cell;
use std::f32::consts::PI;
use std::rc::Rc;

pub struct ArrowFields
{
    m_xTile: Cell<i32>,
    m_yTile: Cell<i32>,
    m_zTile: Cell<i32>,
    m_inTile: Cell<i32>,
    m_blockData: Cell<i32>,
    m_inGround: Cell<bool>,
    pub m_doesArrowBelongToPlayer: Cell<bool>,
    pub m_arrowShake: Cell<i32>,
    pub m_owner: Field<Option<Rc<Entity>>>,
    m_ticksInGround: Cell<i32>,
    m_ticksInAir: Cell<i32>,
}

impl EntityExt for ArrowFields
{
    fn asAny(&self) -> &dyn std::any::Any { self }
}

pub fn fields(this: &Entity) -> &ArrowFields { this.ext::<ArrowFields>() }

fn alloc() -> Rc<Entity>
{
    Entity::alloc(
        &CLASS,
        Box::new(ArrowFields {
            m_xTile: Cell::new(0),
            m_yTile: Cell::new(0),
            m_zTile: Cell::new(0),
            m_inTile: Cell::new(0),
            m_blockData: Cell::new(0),
            m_inGround: Cell::new(false),
            m_doesArrowBelongToPlayer: Cell::new(false),
            m_arrowShake: Cell::new(0),
            m_owner: Field::new(None),
            m_ticksInGround: Cell::new(0),
            m_ticksInAir: Cell::new(0),
        }),
    )
}

fn init(this: &Rc<Entity>, world: Option<Rc<World>>)
{
    Entity::init(this, world);
    let arrow: &ArrowFields = fields(this);
    arrow.m_xTile.set(-1);
    arrow.m_yTile.set(-1);
    arrow.m_zTile.set(-1);
}

pub fn new(world: Option<Rc<World>>) -> Rc<Entity>
{
    let entity: Rc<Entity> = alloc();
    init(&entity, world);
    entity.setSize(0.5, 0.5);
    entity
}

pub fn create(world: &Rc<World>, x: f64, y: f64, z: f64) -> Rc<Entity>
{
    let entity: Rc<Entity> = alloc();
    init(&entity, Some(Rc::clone(world)));
    entity.setSize(0.5, 0.5);
    entity.setPosition(x, y, z);
    entity.m_yOffset.set(0.0);
    entity
}

pub fn createFromShooter(world: &Rc<World>, shooter: &Rc<Entity>) -> Rc<Entity>
{
    let entity: Rc<Entity> = alloc();
    init(&entity, Some(Rc::clone(world)));
    let arrow: &ArrowFields = fields(&entity);
    arrow.m_owner.set(Some(Rc::clone(shooter)));
    arrow.m_doesArrowBelongToPlayer.set(shooter.instanceOf("EntityPlayer"));
    entity.setSize(0.5, 0.5);
    launchFrom(&entity, shooter, shooter.posY() + shooter.getEyeHeight() as f64, 1.0);
    setArrowHeading(&entity, entity.m_motionX.get(), entity.m_motionY.get(), entity.m_motionZ.get(), 1.5, 1.0);
    entity
}

pub(crate) fn launchFrom(this: &Rc<Entity>, thrower: &Rc<Entity>, y: f64, speed: f32)
{
    this.setLocationAndAngles(
        thrower.posX(),
        y,
        thrower.posZ(),
        thrower.m_rotationYaw.get(),
        thrower.m_rotationPitch.get(),
    );
    let yaw: f32 = this.m_rotationYaw.get() / 180.0 * PI;
    this.m_posX.set(this.m_posX.get() - (cos(yaw) * 0.16) as f64);
    this.m_posY.set(this.m_posY.get() - 0.1f32 as f64);
    this.m_posZ.set(this.m_posZ.get() - (sin(yaw) * 0.16) as f64);
    this.setPosition(this.m_posX.get(), this.m_posY.get(), this.m_posZ.get());
    this.m_yOffset.set(0.0);
    let pitch: f32 = this.m_rotationPitch.get() / 180.0 * PI;
    this.m_motionX.set((-sin(yaw) * cos(pitch) * speed) as f64);
    this.m_motionZ.set((cos(yaw) * cos(pitch) * speed) as f64);
    this.m_motionY.set((-sin(pitch) * speed) as f64);
}

pub(crate) fn applyHeading(this: &Entity, mut x: f64, mut y: f64, mut z: f64, speed: f32, spread: f32)
{
    let f: f32 = sqrt_double(x * x + y * y + z * z);
    x /= f as f64;
    y /= f as f64;
    z /= f as f64;
    x += this.m_rand.nextGaussian() * (0.0075f32 * spread) as f64;
    y += this.m_rand.nextGaussian() * (0.0075f32 * spread) as f64;
    z += this.m_rand.nextGaussian() * (0.0075f32 * spread) as f64;
    x *= speed as f64;
    y *= speed as f64;
    z *= speed as f64;
    this.m_motionX.set(x);
    this.m_motionY.set(y);
    this.m_motionZ.set(z);
    let f1: f32 = sqrt_double(x * x + z * z);
    this.m_rotationYaw.set((jvm::math::atan2(x, z) * 180.0 / PI as f64) as f32);
    this.m_prevRotationYaw.set(this.m_rotationYaw.get());
    this.m_rotationPitch.set((jvm::math::atan2(y, f1 as f64) * 180.0 / PI as f64) as f32);
    this.m_prevRotationPitch.set(this.m_rotationPitch.get());
}

pub fn setArrowHeading(this: &Rc<Entity>, x: f64, y: f64, z: f64, speed: f32, spread: f32)
{
    applyHeading(this, x, y, z, speed, spread);
    fields(this).m_ticksInGround.set(0);
}

pub(crate) fn faceVelocity(this: &Entity, x: f64, y: f64, z: f64) -> bool
{
    this.m_motionX.set(x);
    this.m_motionY.set(y);
    this.m_motionZ.set(z);
    if this.m_prevRotationPitch.get() != 0.0 || this.m_prevRotationYaw.get() != 0.0
    {
        return false;
    }
    let f: f32 = sqrt_double(x * x + z * z);
    this.m_rotationYaw.set((jvm::math::atan2(x, z) * 180.0 / PI as f64) as f32);
    this.m_prevRotationYaw.set(this.m_rotationYaw.get());
    this.m_rotationPitch.set((jvm::math::atan2(y, f as f64) * 180.0 / PI as f64) as f32);
    this.m_prevRotationPitch.set(this.m_rotationPitch.get());
    true
}

pub(crate) fn findEntityHit(
    this: &Rc<Entity>,
    thrower: Option<&Rc<Entity>>,
    ignoreThrower: bool,
    from: &Vec3D,
    to: &Vec3D,
) -> Option<Rc<Entity>>
{
    let mut entity: Option<Rc<Entity>> = None;
    let list: Rc<jvm::array_list::JArrayList<Rc<Entity>>> = this.world().getEntitiesWithinAABBExcludingEntity(
        Some(this),
        &this
            .m_boundingBox
            .addCoord(this.m_motionX.get(), this.m_motionY.get(), this.m_motionZ.get())
            .expand(1.0, 1.0, 1.0),
    );
    let mut d: f64 = 0.0;
    let mut i: i32 = 0;
    while i < list.size()
    {
        let entity2: Rc<Entity> = list.get(i);
        i += 1;
        if !entity2.canBeCollidedWith() || ignoreThrower && thrower.is_some_and(|thrower| Rc::ptr_eq(thrower, &entity2))
        {
            continue;
        }
        let f: f32 = 0.3;
        let axisAlignedBB: Rc<AxisAlignedBB> = entity2.m_boundingBox.expand(f as f64, f as f64, f as f64);
        let Some(hit) = axisAlignedBB.clip(from, to)
        else
        {
            continue;
        };
        let d2: f64 = from.distanceTo(&hit.m_hitVec);
        if d2 < d || d == 0.0
        {
            entity = Some(entity2);
            d = d2;
        }
    }
    entity
}

pub(crate) fn rotateTowardsMotion(this: &Entity)
{
    let motionX: f64 = this.m_motionX.get();
    let motionZ: f64 = this.m_motionZ.get();
    let f: f32 = sqrt_double(motionX * motionX + motionZ * motionZ);
    let mut rotationYaw: f32 = (jvm::math::atan2(motionX, motionZ) * 180.0 / PI as f64) as f32;
    let mut rotationPitch: f32 = (jvm::math::atan2(this.m_motionY.get(), f as f64) * 180.0 / PI as f64) as f32;
    let mut prevRotationPitch: f32 = this.m_prevRotationPitch.get();
    let mut prevRotationYaw: f32 = this.m_prevRotationYaw.get();
    while rotationPitch - prevRotationPitch < -180.0
    {
        prevRotationPitch -= 360.0;
    }
    while rotationPitch - prevRotationPitch >= 180.0
    {
        prevRotationPitch += 360.0;
    }
    while rotationYaw - prevRotationYaw < -180.0
    {
        prevRotationYaw -= 360.0;
    }
    while rotationYaw - prevRotationYaw >= 180.0
    {
        prevRotationYaw += 360.0;
    }
    rotationPitch = prevRotationPitch + (rotationPitch - prevRotationPitch) * 0.2;
    rotationYaw = prevRotationYaw + (rotationYaw - prevRotationYaw) * 0.2;
    this.m_prevRotationPitch.set(prevRotationPitch);
    this.m_prevRotationYaw.set(prevRotationYaw);
    this.m_rotationPitch.set(rotationPitch);
    this.m_rotationYaw.set(rotationYaw);
}

pub(crate) fn movePosition(this: &Entity)
{
    this.m_posX.set(this.m_posX.get() + this.m_motionX.get());
    this.m_posY.set(this.m_posY.get() + this.m_motionY.get());
    this.m_posZ.set(this.m_posZ.get() + this.m_motionZ.get());
}

pub(crate) fn spawnBubbles(this: &Entity)
{
    let world: Rc<World> = this.world();
    for _ in 0..4
    {
        let f6: f32 = 0.25;
        world.spawnParticle(
            "bubble",
            this.m_posX.get() - this.m_motionX.get() * f6 as f64,
            this.m_posY.get() - this.m_motionY.get() * f6 as f64,
            this.m_posZ.get() - this.m_motionZ.get() * f6 as f64,
            this.m_motionX.get(),
            this.m_motionY.get(),
            this.m_motionZ.get(),
        );
    }
}

pub(crate) fn scaleMotion(this: &Entity, scale: f64)
{
    this.m_motionX.set(this.m_motionX.get() * scale);
    this.m_motionY.set(this.m_motionY.get() * scale);
    this.m_motionZ.set(this.m_motionZ.get() * scale);
}

pub(crate) fn shakeLoose(this: &Entity)
{
    this.m_motionX.set(this.m_motionX.get() * (this.m_rand.nextFloat() * 0.2) as f64);
    this.m_motionY.set(this.m_motionY.get() * (this.m_rand.nextFloat() * 0.2) as f64);
    this.m_motionZ.set(this.m_motionZ.get() * (this.m_rand.nextFloat() * 0.2) as f64);
}

fn onUpdate(this: &Rc<Entity>)
{
    (ENTITY.onUpdate)(this);
    let arrow: &ArrowFields = fields(this);
    let world: Rc<World> = this.world();
    if this.m_prevRotationPitch.get() == 0.0 && this.m_prevRotationYaw.get() == 0.0
    {
        let f2: f32 =
            sqrt_double(this.m_motionX.get() * this.m_motionX.get() + this.m_motionZ.get() * this.m_motionZ.get());
        this.m_rotationYaw
            .set((jvm::math::atan2(this.m_motionX.get(), this.m_motionZ.get()) * 180.0 / PI as f64) as f32);
        this.m_prevRotationYaw.set(this.m_rotationYaw.get());
        this.m_rotationPitch.set((jvm::math::atan2(this.m_motionY.get(), f2 as f64) * 180.0 / PI as f64) as f32);
        this.m_prevRotationPitch.set(this.m_rotationPitch.get());
    }

    let (xTile, yTile, zTile): (i32, i32, i32) = (arrow.m_xTile.get(), arrow.m_yTile.get(), arrow.m_zTile.get());
    let n: i32 = world.getBlockId(xTile, yTile, zTile);
    if n > 0
    {
        let block: Rc<Block> = Block::byId(n).orNpe();
        block.setBlockBoundsBasedOnState(&*world, xTile, yTile, zTile);
        let object: Option<Rc<AxisAlignedBB>> = block.getCollisionBoundingBoxFromPool(&world, xTile, yTile, zTile);
        if object.is_some_and(|bounds| bounds.isVecInside(&Vec3D::createVector(this.posX(), this.posY(), this.posZ())))
        {
            arrow.m_inGround.set(true);
        }
    }

    if arrow.m_arrowShake.get() > 0
    {
        arrow.m_arrowShake.set(arrow.m_arrowShake.get() - 1);
    }

    if arrow.m_inGround.get()
    {
        let n: i32 = world.getBlockId(xTile, yTile, zTile);
        let n2: i32 = world.getBlockMetadata(xTile, yTile, zTile);
        if n != arrow.m_inTile.get() || n2 != arrow.m_blockData.get()
        {
            arrow.m_inGround.set(false);
            shakeLoose(this);
            arrow.m_ticksInGround.set(0);
            arrow.m_ticksInAir.set(0);
            return;
        }
        arrow.m_ticksInGround.set(arrow.m_ticksInGround.get() + 1);
        if arrow.m_ticksInGround.get() == 1200
        {
            this.setEntityDead();
        }
        return;
    }

    arrow.m_ticksInAir.set(arrow.m_ticksInAir.get() + 1);
    let mut vec3D: Rc<Vec3D> = Vec3D::createVector(this.posX(), this.posY(), this.posZ());
    let mut object: Rc<Vec3D> = Vec3D::createVector(
        this.posX() + this.m_motionX.get(),
        this.posY() + this.m_motionY.get(),
        this.posZ() + this.m_motionZ.get(),
    );
    let mut movingObjectPosition: Option<MovingObjectPosition> = world.rayTrace(&vec3D, &object, false, true);
    vec3D = Vec3D::createVector(this.posX(), this.posY(), this.posZ());
    object = Vec3D::createVector(
        this.posX() + this.m_motionX.get(),
        this.posY() + this.m_motionY.get(),
        this.posZ() + this.m_motionZ.get(),
    );
    if let Some(hit) = &movingObjectPosition
    {
        object = Vec3D::createVector(hit.m_hitVec.x(), hit.m_hitVec.y(), hit.m_hitVec.z());
    }

    let owner: Option<Rc<Entity>> = arrow.m_owner.get();
    if let Some(entity) = findEntityHit(this, owner.as_ref(), arrow.m_ticksInAir.get() < 5, &vec3D, &object)
    {
        movingObjectPosition = Some(MovingObjectPosition::newEntity(&entity));
    }

    if let Some(hit) = movingObjectPosition
    {
        if let Some(entityHit) = &hit.m_entityHit
        {
            if entityHit.attackEntityFrom(owner.as_ref(), 4)
            {
                world.playSoundAtEntity(this, "random.drr", 1.0, 1.2 / (this.m_rand.nextFloat() * 0.2 + 0.9));
                this.setEntityDead();
            }
            else
            {
                scaleMotion(this, -0.1f32 as f64);
                this.m_rotationYaw.set(this.m_rotationYaw.get() + 180.0);
                this.m_prevRotationYaw.set(this.m_prevRotationYaw.get() + 180.0);
                arrow.m_ticksInAir.set(0);
            }
        }
        else
        {
            arrow.m_xTile.set(hit.m_blockX);
            arrow.m_yTile.set(hit.m_blockY);
            arrow.m_zTile.set(hit.m_blockZ);
            arrow.m_inTile.set(world.getBlockId(hit.m_blockX, hit.m_blockY, hit.m_blockZ));
            arrow.m_blockData.set(world.getBlockMetadata(hit.m_blockX, hit.m_blockY, hit.m_blockZ));
            this.m_motionX.set((hit.m_hitVec.x() - this.posX()) as f32 as f64);
            this.m_motionY.set((hit.m_hitVec.y() - this.posY()) as f32 as f64);
            this.m_motionZ.set((hit.m_hitVec.z() - this.posZ()) as f32 as f64);
            let (motionX, motionY, motionZ): (f64, f64, f64) =
                (this.m_motionX.get(), this.m_motionY.get(), this.m_motionZ.get());
            let f3: f32 = sqrt_double(motionX * motionX + motionY * motionY + motionZ * motionZ);
            this.m_posX.set(this.posX() - motionX / f3 as f64 * 0.05f32 as f64);
            this.m_posY.set(this.posY() - motionY / f3 as f64 * 0.05f32 as f64);
            this.m_posZ.set(this.posZ() - motionZ / f3 as f64 * 0.05f32 as f64);
            world.playSoundAtEntity(this, "random.drr", 1.0, 1.2 / (this.m_rand.nextFloat() * 0.2 + 0.9));
            arrow.m_inGround.set(true);
            arrow.m_arrowShake.set(7);
        }
    }

    movePosition(this);
    rotateTowardsMotion(this);
    let mut f5: f32 = 0.99;
    let f: f32 = 0.03;
    if this.isInWater()
    {
        spawnBubbles(this);
        f5 = 0.8;
    }
    scaleMotion(this, f5 as f64);
    this.m_motionY.set(this.m_motionY.get() - f as f64);
    this.setPosition(this.posX(), this.posY(), this.posZ());
}

pub const CLASS: EntityClass = EntityClass {
    m_name: "EntityArrow",
    m_parent: Some(&ENTITY),
    m_interfaces: &[],
    entityInit: |_| {},
    setVelocity: |this, x, y, z| {
        if faceVelocity(this, x, y, z)
        {
            this.m_prevRotationPitch.set(this.m_rotationPitch.get());
            this.m_prevRotationYaw.set(this.m_rotationYaw.get());
            this.setLocationAndAngles(
                this.posX(),
                this.posY(),
                this.posZ(),
                this.m_rotationYaw.get(),
                this.m_rotationPitch.get(),
            );
            fields(this).m_ticksInGround.set(0);
        }
    },
    onUpdate,
    writeEntityToNBT: |this, nbt: &NBTTagCompound| {
        let arrow: &ArrowFields = fields(this);
        nbt.setShort("xTile", arrow.m_xTile.get() as i16);
        nbt.setShort("yTile", arrow.m_yTile.get() as i16);
        nbt.setShort("zTile", arrow.m_zTile.get() as i16);
        nbt.setByte("inTile", arrow.m_inTile.get() as i8);
        nbt.setByte("inData", arrow.m_blockData.get() as i8);
        nbt.setByte("shake", arrow.m_arrowShake.get() as i8);
        nbt.setByte("inGround", if arrow.m_inGround.get() { 1 } else { 0 });
        nbt.setBoolean("player", arrow.m_doesArrowBelongToPlayer.get());
    },
    readEntityFromNBT: |this, nbt: &NBTTagCompound| {
        let arrow: &ArrowFields = fields(this);
        arrow.m_xTile.set(nbt.getShort("xTile") as i32);
        arrow.m_yTile.set(nbt.getShort("yTile") as i32);
        arrow.m_zTile.set(nbt.getShort("zTile") as i32);
        arrow.m_inTile.set(nbt.getByte("inTile") as i32 & 0xFF);
        arrow.m_blockData.set(nbt.getByte("inData") as i32 & 0xFF);
        arrow.m_arrowShake.set(nbt.getByte("shake") as i32 & 0xFF);
        arrow.m_inGround.set(nbt.getByte("inGround") == 1);
        arrow.m_doesArrowBelongToPlayer.set(nbt.getBoolean("player"));
    },
    onCollideWithPlayer: |this, player| {
        let world: Rc<World> = this.world();
        if world.m_multiplayerWorld.get()
        {
            return;
        }
        let arrow: &ArrowFields = fields(this);
        if arrow.m_inGround.get()
            && arrow.m_doesArrowBelongToPlayer.get()
            && arrow.m_arrowShake.get() <= 0
            && entity_player::inventory(player).addItemStackToInventory(&ItemStack::withItemCount(&Item::arrow(), 1))
        {
            world.playSoundAtEntity(
                this,
                "random.pop",
                0.2,
                ((this.m_rand.nextFloat() - this.m_rand.nextFloat()) * 0.7 + 1.0) * 2.0,
            );
            player.onItemPickup(this, 1);
            this.setEntityDead();
        }
    },
    getShadowSize: |_| 0.0,
    ..ENTITY
};
