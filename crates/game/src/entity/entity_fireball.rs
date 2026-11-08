use crate::entity::entity_arrow::{movePosition, rotateTowardsMotion, scaleMotion, spawnBubbles};
use crate::entity::entity_snowball::{self, ThrowableFields, tickInGround, traceHit};
use crate::entity::{ENTITY, Entity, EntityClass, EntityExt};
use crate::math::math_helper::sqrt_double;
use crate::world::World;
use std::cell::Cell;
use std::rc::Rc;

pub struct FireballFields
{
    pub m_throwable: ThrowableFields,
    pub m_accelerationX: Cell<f64>,
    pub m_accelerationY: Cell<f64>,
    pub m_accelerationZ: Cell<f64>,
}

impl EntityExt for FireballFields
{
    fn asAny(&self) -> &dyn std::any::Any { self }
}

pub fn fields(this: &Entity) -> &FireballFields { this.ext::<FireballFields>() }

fn alloc(world: Option<Rc<World>>) -> Rc<Entity>
{
    let entity: Rc<Entity> = Entity::alloc(
        &CLASS,
        Box::new(FireballFields {
            m_throwable: ThrowableFields::new(),
            m_accelerationX: Cell::new(0.0),
            m_accelerationY: Cell::new(0.0),
            m_accelerationZ: Cell::new(0.0),
        }),
    );
    Entity::init(&entity, world);
    fields(&entity).m_throwable.initTile();
    entity.setSize(1.0, 1.0);
    entity
}

fn setAcceleration(this: &Entity, x: f64, y: f64, z: f64)
{
    let d0: f64 = sqrt_double(x * x + y * y + z * z) as f64;
    let fireball: &FireballFields = fields(this);
    fireball.m_accelerationX.set(x / d0 * 0.1);
    fireball.m_accelerationY.set(y / d0 * 0.1);
    fireball.m_accelerationZ.set(z / d0 * 0.1);
}

pub fn new(world: Option<Rc<World>>) -> Rc<Entity> { alloc(world) }

pub fn createAt(
    world: &Rc<World>,
    x: f64,
    y: f64,
    z: f64,
    accelerationX: f64,
    accelerationY: f64,
    accelerationZ: f64,
) -> Rc<Entity>
{
    let entity: Rc<Entity> = alloc(Some(Rc::clone(world)));
    entity.setLocationAndAngles(x, y, z, entity.m_rotationYaw.get(), entity.m_rotationPitch.get());
    entity.setPosition(x, y, z);
    setAcceleration(&entity, accelerationX, accelerationY, accelerationZ);
    entity
}

pub fn createFromShooter(
    world: &Rc<World>,
    shooter: &Rc<Entity>,
    mut accelerationX: f64,
    mut accelerationY: f64,
    mut accelerationZ: f64,
) -> Rc<Entity>
{
    let entity: Rc<Entity> = alloc(Some(Rc::clone(world)));
    fields(&entity).m_throwable.m_thrower.set(Some(Rc::clone(shooter)));
    entity.setLocationAndAngles(
        shooter.posX(),
        shooter.posY(),
        shooter.posZ(),
        shooter.m_rotationYaw.get(),
        shooter.m_rotationPitch.get(),
    );
    entity.setPosition(entity.posX(), entity.posY(), entity.posZ());
    entity.m_yOffset.set(0.0);
    entity.m_motionX.set(0.0);
    entity.m_motionY.set(0.0);
    entity.m_motionZ.set(0.0);
    accelerationX += entity.m_rand.nextGaussian() * 0.4;
    accelerationY += entity.m_rand.nextGaussian() * 0.4;
    accelerationZ += entity.m_rand.nextGaussian() * 0.4;
    setAcceleration(&entity, accelerationX, accelerationY, accelerationZ);
    entity
}

fn onUpdate(this: &Rc<Entity>)
{
    (ENTITY.onUpdate)(this);
    this.m_fire.set(10);
    let fireball: &FireballFields = fields(this);
    if tickInGround(this, &fireball.m_throwable)
    {
        return;
    }
    let world: Rc<World> = this.world();
    if let Some(hit) = traceHit(this, &fireball.m_throwable, 25, true)
    {
        if !world.m_multiplayerWorld.get()
        {
            if let Some(entityHit) = &hit.m_entityHit
            {
                entityHit.attackEntityFrom(fireball.m_throwable.m_thrower.get().as_ref(), 0);
            }
            world.newExplosion(None, this.posX(), this.posY(), this.posZ(), 1.0, true);
        }
        this.setEntityDead();
    }

    movePosition(this);
    rotateTowardsMotion(this);
    let mut f2: f32 = 0.95;
    if this.isInWater()
    {
        spawnBubbles(this);
        f2 = 0.8;
    }
    this.m_motionX.set(this.m_motionX.get() + fireball.m_accelerationX.get());
    this.m_motionY.set(this.m_motionY.get() + fireball.m_accelerationY.get());
    this.m_motionZ.set(this.m_motionZ.get() + fireball.m_accelerationZ.get());
    scaleMotion(this, f2 as f64);
    world.spawnParticle("smoke", this.posX(), this.posY() + 0.5, this.posZ(), 0.0, 0.0, 0.0);
    this.setPosition(this.posX(), this.posY(), this.posZ());
}

pub const CLASS: EntityClass = EntityClass {
    m_name: "EntityFireball",
    m_parent: Some(&ENTITY),
    m_interfaces: &[],
    entityInit: |_| {},
    isInRangeToRenderDist: entity_snowball::isInRangeToRenderDist,
    onUpdate,
    writeEntityToNBT: entity_snowball::writeEntityToNBT,
    readEntityFromNBT: entity_snowball::readEntityFromNBT,
    canBeCollidedWith: |_| true,
    getCollisionBorderSize: |_| 1.0,
    attackEntityFrom: |this, source, _amount| {
        this.setBeenAttacked();
        let Some(source) = source
        else
        {
            return false;
        };
        if let Some(vec3d) = source.getLookVec()
        {
            this.m_motionX.set(vec3d.x());
            this.m_motionY.set(vec3d.y());
            this.m_motionZ.set(vec3d.z());
            let fireball: &FireballFields = fields(this);
            fireball.m_accelerationX.set(this.m_motionX.get() * 0.1);
            fireball.m_accelerationY.set(this.m_motionY.get() * 0.1);
            fireball.m_accelerationZ.set(this.m_motionZ.get() * 0.1);
        }
        true
    },
    getShadowSize: |_| 0.0,
    ..ENTITY
};
