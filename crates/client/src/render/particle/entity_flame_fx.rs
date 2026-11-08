use crate::render::particle::entity_fx::{self, FXFields, extra, fx};
use game::entity::{ENTITY, Entity, EntityClass};
use game::world::World;
use jvm::math;
use std::cell::Cell;
use std::rc::Rc;

pub struct FlameFields
{
    m_initialSize: Cell<f32>,
}

fn onUpdate(this: &Rc<Entity>)
{
    entity_fx::setPrevPosition(this);
    entity_fx::ageParticle(this);
    entity_fx::moveByMotion(this);
    entity_fx::scaleMotion(this, 0.96f32 as f64);
    entity_fx::groundFriction(this);
}

pub const CLASS: EntityClass = EntityClass {
    m_name: "EntityFlameFX",
    m_parent: Some(&entity_fx::CLASS),
    renderParticle: |this, tessellator, tickDelta, dx, dy, dz, forwards, sideways| {
        let f: f32 = entity_fx::lifeFraction(this, tickDelta);
        fx(this).m_particleScale.set(extra::<FlameFields>(this).m_initialSize.get() * (1.0 - f * f * 0.5));
        entity_fx::renderParticle(this, tessellator, tickDelta, dx, dy, dz, forwards, sideways);
    },
    getEntityBrightness: |this, tickDelta| {
        let mut f: f32 = entity_fx::lifeFraction(this, tickDelta);
        if f < 0.0
        {
            f = 0.0;
        }
        if f > 1.0
        {
            f = 1.0;
        }
        let f1: f32 = (ENTITY.getEntityBrightness)(this, tickDelta);
        f1 * f + (1.0 - f)
    },
    onUpdate,
    ..entity_fx::CLASS
};

pub fn new(world: &Rc<World>, x: f64, y: f64, z: f64, velocityX: f64, velocityY: f64, velocityZ: f64) -> Rc<Entity>
{
    let flame: FlameFields = FlameFields { m_initialSize: Cell::new(0.0) };
    let this: Rc<Entity> =
        entity_fx::construct(&CLASS, Box::new(flame), world, x, y, z, velocityX, velocityY, velocityZ);
    this.m_motionX.set(this.m_motionX.get() * 0.01f32 as f64 + velocityX);
    this.m_motionY.set(this.m_motionY.get() * 0.01f32 as f64 + velocityY);
    this.m_motionZ.set(this.m_motionZ.get() * 0.01f32 as f64 + velocityZ);
    for _ in 0..3
    {
        let _offset: f32 = (this.m_rand.nextFloat() - this.m_rand.nextFloat()) * 0.05;
    }
    let fields: &FXFields = fx(&this);
    extra::<FlameFields>(&this).m_initialSize.set(fields.m_particleScale.get());
    fields.m_particleRed.set(1.0);
    fields.m_particleGreen.set(1.0);
    fields.m_particleBlue.set(1.0);
    fields.m_particleMaxAge.set((8.0 / (math::random() * 0.8 + 0.2)) as i32 + 4);
    this.m_noClip.set(true);
    fields.m_particleTextureIndex.set(48);
    this
}
