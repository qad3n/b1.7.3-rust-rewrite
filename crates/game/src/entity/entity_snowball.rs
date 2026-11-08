use crate::entity::entity_arrow::{
    applyHeading, faceVelocity, findEntityHit, launchFrom, movePosition, rotateTowardsMotion, scaleMotion, shakeLoose,
    spawnBubbles,
};
use crate::entity::{ENTITY, Entity, EntityClass, EntityExt, entity_player};
use crate::item::{Item, ItemStack};
use crate::math::{MovingObjectPosition, Vec3D};
use crate::world::World;
use common::nbt::NBTTagCompound;
use jvm::field::Field;
use std::cell::Cell;
use std::rc::Rc;

pub struct ThrowableFields
{
    m_xTile: Cell<i32>,
    m_yTile: Cell<i32>,
    m_zTile: Cell<i32>,
    m_inTile: Cell<i32>,
    pub(crate) m_inGround: Cell<bool>,
    pub m_shake: Cell<i32>,
    pub m_thrower: Field<Option<Rc<Entity>>>,
    m_ticksInGround: Cell<i32>,
    m_ticksInAir: Cell<i32>,
}

impl ThrowableFields
{
    pub(crate) fn new() -> ThrowableFields
    {
        ThrowableFields {
            m_xTile: Cell::new(0),
            m_yTile: Cell::new(0),
            m_zTile: Cell::new(0),
            m_inTile: Cell::new(0),
            m_inGround: Cell::new(false),
            m_shake: Cell::new(0),
            m_thrower: Field::new(None),
            m_ticksInGround: Cell::new(0),
            m_ticksInAir: Cell::new(0),
        }
    }

    pub(crate) fn initTile(&self)
    {
        self.m_xTile.set(-1);
        self.m_yTile.set(-1);
        self.m_zTile.set(-1);
    }

    pub(crate) fn resetTicksInGround(&self) { self.m_ticksInGround.set(0); }
}

impl EntityExt for ThrowableFields
{
    fn asAny(&self) -> &dyn std::any::Any { self }
}

pub fn throwable(this: &Entity) -> &ThrowableFields
{
    if let Some(fields) = this.m_ext.asAny().downcast_ref::<ThrowableFields>()
    {
        return fields;
    }
    match this.className()
    {
        "EntityFireball" => &crate::entity::entity_fireball::fields(this).m_throwable,
        _ => &crate::entity::entity_fish::fields(this).m_throwable,
    }
}

pub(crate) fn tickInGround(this: &Rc<Entity>, fields: &ThrowableFields) -> bool
{
    if fields.m_shake.get() > 0
    {
        fields.m_shake.set(fields.m_shake.get() - 1);
    }
    if !fields.m_inGround.get()
    {
        fields.m_ticksInAir.set(fields.m_ticksInAir.get() + 1);
        return false;
    }
    let i: i32 = this.world().getBlockId(fields.m_xTile.get(), fields.m_yTile.get(), fields.m_zTile.get());
    if i == fields.m_inTile.get()
    {
        fields.m_ticksInGround.set(fields.m_ticksInGround.get() + 1);
        if fields.m_ticksInGround.get() == 1200
        {
            this.setEntityDead();
        }
        return true;
    }
    fields.m_inGround.set(false);
    shakeLoose(this);
    fields.m_ticksInGround.set(0);
    fields.m_ticksInAir.set(0);
    false
}

pub(crate) fn traceHit(
    this: &Rc<Entity>,
    fields: &ThrowableFields,
    throwerGraceTicks: i32,
    searchEntities: bool,
) -> Option<MovingObjectPosition>
{
    let world: Rc<World> = this.world();
    let mut vec3d1: Rc<Vec3D> = Vec3D::createVector(this.posX(), this.posY(), this.posZ());
    let mut vec3d: Rc<Vec3D> = Vec3D::createVector(
        this.posX() + this.m_motionX.get(),
        this.posY() + this.m_motionY.get(),
        this.posZ() + this.m_motionZ.get(),
    );
    let mut movingobjectposition: Option<MovingObjectPosition> = world.rayTraceBlocks(&vec3d1, &vec3d);
    vec3d1 = Vec3D::createVector(this.posX(), this.posY(), this.posZ());
    vec3d = Vec3D::createVector(
        this.posX() + this.m_motionX.get(),
        this.posY() + this.m_motionY.get(),
        this.posZ() + this.m_motionZ.get(),
    );
    if let Some(hit) = &movingobjectposition
    {
        vec3d = Vec3D::createVector(hit.m_hitVec.x(), hit.m_hitVec.y(), hit.m_hitVec.z());
    }
    if searchEntities
    {
        let thrower: Option<Rc<Entity>> = fields.m_thrower.get();
        let ignoreThrower: bool = fields.m_ticksInAir.get() < throwerGraceTicks;
        if let Some(entity) = findEntityHit(this, thrower.as_ref(), ignoreThrower, &vec3d1, &vec3d)
        {
            movingobjectposition = Some(MovingObjectPosition::newEntity(&entity));
        }
    }
    movingobjectposition
}

pub(crate) fn initFromThrower(this: &Rc<Entity>, thrower: &Rc<Entity>)
{
    throwable(this).m_thrower.set(Some(Rc::clone(thrower)));
    this.setSize(0.25, 0.25);
    launchFrom(this, thrower, thrower.posY() + thrower.getEyeHeight() as f64, 0.4);
    setSnowballHeading(this, this.m_motionX.get(), this.m_motionY.get(), this.m_motionZ.get(), 1.5, 1.0);
}

pub(crate) fn initAt(this: &Rc<Entity>, x: f64, y: f64, z: f64)
{
    throwable(this).m_ticksInGround.set(0);
    this.setSize(0.25, 0.25);
    this.setPosition(x, y, z);
    this.m_yOffset.set(0.0);
}

pub(crate) fn alloc(class: &'static EntityClass, world: Option<Rc<World>>) -> Rc<Entity>
{
    let entity: Rc<Entity> = Entity::alloc(class, Box::new(ThrowableFields::new()));
    Entity::init(&entity, world);
    throwable(&entity).initTile();
    entity
}

pub fn new(world: Option<Rc<World>>) -> Rc<Entity>
{
    let entity: Rc<Entity> = alloc(&CLASS, world);
    entity.setSize(0.25, 0.25);
    entity
}

pub fn create(world: &Rc<World>, x: f64, y: f64, z: f64) -> Rc<Entity>
{
    let entity: Rc<Entity> = alloc(&CLASS, Some(Rc::clone(world)));
    initAt(&entity, x, y, z);
    entity
}

pub fn createFromThrower(world: &Rc<World>, thrower: &Rc<Entity>) -> Rc<Entity>
{
    let entity: Rc<Entity> = alloc(&CLASS, Some(Rc::clone(world)));
    initFromThrower(&entity, thrower);
    entity
}

pub fn setSnowballHeading(this: &Rc<Entity>, x: f64, y: f64, z: f64, speed: f32, spread: f32)
{
    applyHeading(this, x, y, z, speed, spread);
    throwable(this).m_ticksInGround.set(0);
}

pub(crate) fn updateThrown(this: &Rc<Entity>, onImpact: fn(&Rc<Entity>))
{
    this.m_lastTickPosX.set(this.posX());
    this.m_lastTickPosY.set(this.posY());
    this.m_lastTickPosZ.set(this.posZ());
    (ENTITY.onUpdate)(this);
    let fields: &ThrowableFields = throwable(this);
    if tickInGround(this, fields)
    {
        return;
    }
    let world: Rc<World> = this.world();
    let movingobjectposition: Option<MovingObjectPosition> = traceHit(this, fields, 5, !world.m_multiplayerWorld.get());
    if let Some(hit) = movingobjectposition
    {
        if let Some(entityHit) = &hit.m_entityHit
        {
            entityHit.attackEntityFrom(fields.m_thrower.get().as_ref(), 0);
        }
        onImpact(this);
        for _ in 0..8
        {
            world.spawnParticle("snowballpoof", this.posX(), this.posY(), this.posZ(), 0.0, 0.0, 0.0);
        }
        this.setEntityDead();
    }

    movePosition(this);
    rotateTowardsMotion(this);
    let mut f2: f32 = 0.99;
    let f3: f32 = 0.03;
    if this.isInWater()
    {
        spawnBubbles(this);
        f2 = 0.8;
    }
    scaleMotion(this, f2 as f64);
    this.m_motionY.set(this.m_motionY.get() - f3 as f64);
    this.setPosition(this.posX(), this.posY(), this.posZ());
}

pub(crate) fn setVelocity(this: &Rc<Entity>, x: f64, y: f64, z: f64) { faceVelocity(this, x, y, z); }

pub(crate) fn onCollideWithPlayer(this: &Rc<Entity>, player: &Rc<Entity>)
{
    let fields: &ThrowableFields = throwable(this);
    if fields.m_inGround.get()
        && fields.m_thrower.get().is_some_and(|thrower| Rc::ptr_eq(&thrower, player))
        && fields.m_shake.get() <= 0
        && entity_player::inventory(player).addItemStackToInventory(&ItemStack::withItemCount(&Item::arrow(), 1))
    {
        this.world().playSoundAtEntity(
            this,
            "random.pop",
            0.2,
            ((this.m_rand.nextFloat() - this.m_rand.nextFloat()) * 0.7 + 1.0) * 2.0,
        );
        player.onItemPickup(this, 1);
        this.setEntityDead();
    }
}

pub(crate) fn isInRangeToRenderDist(this: &Rc<Entity>, squaredDistanceToCamera: f64) -> bool
{
    let mut d0: f64 = this.m_boundingBox.getAverageEdgeLength() * 4.0;
    d0 *= 64.0;
    squaredDistanceToCamera < d0 * d0
}

pub(crate) fn writeEntityToNBT(this: &Rc<Entity>, nbt: &NBTTagCompound)
{
    let fields: &ThrowableFields = throwable(this);
    nbt.setShort("xTile", fields.m_xTile.get() as i16);
    nbt.setShort("yTile", fields.m_yTile.get() as i16);
    nbt.setShort("zTile", fields.m_zTile.get() as i16);
    nbt.setByte("inTile", fields.m_inTile.get() as i8);
    nbt.setByte("shake", fields.m_shake.get() as i8);
    nbt.setByte("inGround", if fields.m_inGround.get() { 1 } else { 0 });
}

pub(crate) fn readEntityFromNBT(this: &Rc<Entity>, nbt: &NBTTagCompound)
{
    let fields: &ThrowableFields = throwable(this);
    fields.m_xTile.set(nbt.getShort("xTile") as i32);
    fields.m_yTile.set(nbt.getShort("yTile") as i32);
    fields.m_zTile.set(nbt.getShort("zTile") as i32);
    fields.m_inTile.set(nbt.getByte("inTile") as i32 & 255);
    fields.m_shake.set(nbt.getByte("shake") as i32 & 255);
    fields.m_inGround.set(nbt.getByte("inGround") == 1);
}

pub const CLASS: EntityClass = EntityClass {
    m_name: "EntitySnowball",
    m_parent: Some(&ENTITY),
    m_interfaces: &[],
    entityInit: |_| {},
    isInRangeToRenderDist,
    setVelocity,
    onUpdate: |this| updateThrown(this, |_| {}),
    writeEntityToNBT,
    readEntityFromNBT,
    onCollideWithPlayer,
    getShadowSize: |_| 0.0,
    ..ENTITY
};
