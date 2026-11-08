use crate::render::particle::entity_fx::{self, FXFields, fx};
use game::entity::{Entity, EntityClass};
use game::world::World;
use jvm::math;
use std::rc::Rc;

pub fn animateTexture(this: &Entity)
{
    let fields: &FXFields = fx(this);
    fields.m_particleTextureIndex.set(7 - fields.m_particleAge.get() * 8 / fields.m_particleMaxAge.get());
}

fn onUpdate(this: &Rc<Entity>)
{
    entity_fx::setPrevPosition(this);
    entity_fx::ageParticle(this);
    animateTexture(this);
    this.m_motionY.set(this.m_motionY.get() + 0.004);
    entity_fx::moveByMotion(this);
    entity_fx::scaleMotion(this, 0.9f32 as f64);
    entity_fx::groundFriction(this);
}

pub const CLASS: EntityClass = EntityClass {
    m_name: "EntityExplodeFX",
    m_parent: Some(&entity_fx::CLASS),
    renderParticle: |this, tessellator, tickDelta, dx, dy, dz, forwards, sideways| {
        (entity_fx::CLASS.renderParticle)(this, tessellator, tickDelta, dx, dy, dz, forwards, sideways)
    },
    onUpdate,
    ..entity_fx::CLASS
};

fn velocity(value: f64) -> f64 { value + ((math::random() * 2.0 - 1.0) as f32 * 0.05) as f64 }

pub fn new(world: &Rc<World>, x: f64, y: f64, z: f64, velocityX: f64, velocityY: f64, velocityZ: f64) -> Rc<Entity>
{
    let this: Rc<Entity> = entity_fx::construct(&CLASS, Box::new(()), world, x, y, z, velocityX, velocityY, velocityZ);
    this.m_motionX.set(velocity(velocityX));
    this.m_motionY.set(velocity(velocityY));
    this.m_motionZ.set(velocity(velocityZ));
    let fields: &FXFields = fx(&this);
    let color: f32 = this.m_rand.nextFloat() * 0.3 + 0.7;
    fields.m_particleRed.set(color);
    fields.m_particleGreen.set(color);
    fields.m_particleBlue.set(color);
    fields.m_particleScale.set(this.m_rand.nextFloat() * this.m_rand.nextFloat() * 6.0 + 1.0);
    fields.m_particleMaxAge.set((16.0 / (this.m_rand.nextFloat() as f64 * 0.8 + 0.2)) as i32 + 2);
    this
}
