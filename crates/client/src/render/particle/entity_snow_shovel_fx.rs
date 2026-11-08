use crate::render::particle::entity_explode_fx::animateTexture;
use crate::render::particle::entity_fx::{self, FXFields, extra, fx};
use crate::render::particle::entity_heart_fx::{self, InitialSize};
use game::entity::{Entity, EntityClass};
use game::world::World;
use jvm::math;
use std::cell::Cell;
use std::rc::Rc;

fn onUpdate(this: &Rc<Entity>)
{
    entity_fx::setPrevPosition(this);
    entity_fx::ageParticle(this);
    animateTexture(this);
    this.m_motionY.set(this.m_motionY.get() - 0.03);
    entity_fx::moveByMotion(this);
    entity_fx::scaleMotion(this, 0.99f32 as f64);
    entity_fx::groundFriction(this);
}

pub const CLASS: EntityClass = EntityClass {
    m_name: "EntitySnowShovelFX",
    m_parent: Some(&entity_fx::CLASS),
    renderParticle: entity_heart_fx::renderGrowing,
    onUpdate,
    ..entity_fx::CLASS
};

pub fn new(world: &Rc<World>, x: f64, y: f64, z: f64, velocityX: f64, velocityY: f64, velocityZ: f64) -> Rc<Entity>
{
    newScaled(world, x, y, z, velocityX, velocityY, velocityZ, 1.0)
}

pub fn newScaled(
    world: &Rc<World>,
    x: f64,
    y: f64,
    z: f64,
    velocityX: f64,
    velocityY: f64,
    velocityZ: f64,
    scaleFactor: f32,
) -> Rc<Entity>
{
    let size: InitialSize = InitialSize { m_initialSize: Cell::new(0.0) };
    let this: Rc<Entity> =
        entity_fx::construct(&CLASS, Box::new(size), world, x, y, z, velocityX, velocityY, velocityZ);
    entity_fx::scaleMotion(&this, 0.1f32 as f64);
    this.m_motionX.set(this.m_motionX.get() + velocityX);
    this.m_motionY.set(this.m_motionY.get() + velocityY);
    this.m_motionZ.set(this.m_motionZ.get() + velocityZ);
    let fields: &FXFields = fx(&this);
    let color: f32 = 1.0 - (math::random() * 0.3f32 as f64) as f32;
    fields.m_particleRed.set(color);
    fields.m_particleGreen.set(color);
    fields.m_particleBlue.set(color);
    fields.m_particleScale.set(fields.m_particleScale.get() * 0.75);
    fields.m_particleScale.set(fields.m_particleScale.get() * scaleFactor);
    extra::<InitialSize>(&this).m_initialSize.set(fields.m_particleScale.get());
    fields.m_particleMaxAge.set((8.0 / (math::random() * 0.8 + 0.2)) as i32);
    fields.m_particleMaxAge.set((fields.m_particleMaxAge.get() as f32 * scaleFactor) as i32);
    this.m_noClip.set(false);
    this
}
