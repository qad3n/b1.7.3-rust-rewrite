use crate::entity::{ENTITY, Entity, EntityClass, EntityExt};
use crate::math::math_helper::{cos, sin};
use crate::world::World;
use std::cell::Cell;
use std::f32::consts::PI;
use std::rc::Rc;

pub struct TntFields
{
    pub m_fuse: Cell<i32>,
}

impl EntityExt for TntFields
{
    fn asAny(&self) -> &dyn std::any::Any { self }
}

pub fn fields(this: &Entity) -> &TntFields { this.ext::<TntFields>() }

pub fn new(world: Option<Rc<World>>) -> Rc<Entity>
{
    let entity: Rc<Entity> = Entity::alloc(&CLASS, Box::new(TntFields { m_fuse: Cell::new(0) }));
    Entity::init(&entity, world);
    entity.m_preventEntitySpawning.set(true);
    entity.setSize(0.98, 0.98);
    entity.m_yOffset.set(entity.m_height.get() / 2.0);
    entity
}

pub fn create(world: &Rc<World>, x: f64, y: f64, z: f64) -> Rc<Entity>
{
    let entity: Rc<Entity> = new(Some(Rc::clone(world)));
    entity.setPosition(x, y, z);
    let f: f32 = (jvm::math::random() * PI as f64 * 2.0) as f32;
    entity.m_motionX.set((-sin(f * PI / 180.0) * 0.02) as f64);
    entity.m_motionY.set(0.2f32 as f64);
    entity.m_motionZ.set((-cos(f * PI / 180.0) * 0.02) as f64);
    fields(&entity).m_fuse.set(80);
    entity.m_prevPosX.set(x);
    entity.m_prevPosY.set(y);
    entity.m_prevPosZ.set(z);
    entity
}

fn onUpdate(this: &Rc<Entity>)
{
    this.m_prevPosX.set(this.posX());
    this.m_prevPosY.set(this.posY());
    this.m_prevPosZ.set(this.posZ());
    this.m_motionY.set(this.m_motionY.get() - 0.04f32 as f64);
    this.moveEntity(this.m_motionX.get(), this.m_motionY.get(), this.m_motionZ.get());
    this.m_motionX.set(this.m_motionX.get() * 0.98f32 as f64);
    this.m_motionY.set(this.m_motionY.get() * 0.98f32 as f64);
    this.m_motionZ.set(this.m_motionZ.get() * 0.98f32 as f64);
    if this.m_onGround.get()
    {
        this.m_motionX.set(this.m_motionX.get() * 0.7f32 as f64);
        this.m_motionZ.set(this.m_motionZ.get() * 0.7f32 as f64);
        this.m_motionY.set(this.m_motionY.get() * -0.5);
    }
    let tnt: &TntFields = fields(this);
    let fuse: i32 = tnt.m_fuse.get();
    tnt.m_fuse.set(fuse - 1);
    let world: Rc<World> = this.world();
    if fuse <= 0
    {
        this.setEntityDead();
        if !world.m_multiplayerWorld.get()
        {
            explode(this, &world);
        }
    }
    else
    {
        world.spawnParticle("smoke", this.posX(), this.posY() + 0.5, this.posZ(), 0.0, 0.0, 0.0);
    }
}

fn explode(this: &Entity, world: &Rc<World>)
{
    let f: f32 = 4.0;
    world.createExplosion(None, this.posX(), this.posY(), this.posZ(), f);
}

pub const CLASS: EntityClass = EntityClass {
    m_name: "EntityTNTPrimed",
    m_parent: Some(&ENTITY),
    m_interfaces: &[],
    entityInit: |_| {},
    canTriggerWalking: |_| false,
    canBeCollidedWith: |this| !this.m_isDead.get(),
    onUpdate,
    writeEntityToNBT: |this, nbt| nbt.setByte("Fuse", fields(this).m_fuse.get() as i8),
    readEntityFromNBT: |this, nbt| fields(this).m_fuse.set(nbt.getByte("Fuse") as i32),
    getShadowSize: |_| 0.0,
    ..ENTITY
};
