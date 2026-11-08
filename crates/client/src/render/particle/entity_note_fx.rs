use crate::render::particle::entity_fx::{self, FXFields, extra, fx};
use crate::render::particle::entity_heart_fx::{self, InitialSize};
use game::entity::{Entity, EntityClass};
use game::math::math_helper;
use game::world::World;
use std::cell::Cell;
use std::f64::consts::PI;
use std::rc::Rc;

pub const CLASS: EntityClass = EntityClass {
    m_name: "EntityNoteFX",
    m_parent: Some(&entity_fx::CLASS),
    renderParticle: entity_heart_fx::renderGrowing,
    onUpdate: |this| entity_heart_fx::floatUpward(this, 0.66),
    ..entity_fx::CLASS
};

fn noteColor(velocityX: f64, offset: f32) -> f32
{
    math_helper::sin((velocityX as f32 + offset) * PI as f32 * 2.0) * 0.65 + 0.35
}

pub fn new(world: &Rc<World>, x: f64, y: f64, z: f64, velocityX: f64, velocityY: f64, velocityZ: f64) -> Rc<Entity>
{
    newScaled(world, x, y, z, velocityX, velocityY, velocityZ, 2.0)
}

pub fn newScaled(
    world: &Rc<World>,
    x: f64,
    y: f64,
    z: f64,
    velocityX: f64,
    _velocityY: f64,
    _velocityZ: f64,
    scaleFactor: f32,
) -> Rc<Entity>
{
    let size: InitialSize = InitialSize { m_initialSize: Cell::new(0.0) };
    let this: Rc<Entity> = entity_fx::construct(&CLASS, Box::new(size), world, x, y, z, 0.0, 0.0, 0.0);
    entity_fx::scaleMotion(&this, 0.01f32 as f64);
    this.m_motionY.set(this.m_motionY.get() + 0.2);
    let fields: &FXFields = fx(&this);
    fields.m_particleRed.set(noteColor(velocityX, 0.0));
    fields.m_particleGreen.set(noteColor(velocityX, 0.33333334));
    fields.m_particleBlue.set(noteColor(velocityX, 0.6666667));
    fields.m_particleScale.set(fields.m_particleScale.get() * 0.75);
    fields.m_particleScale.set(fields.m_particleScale.get() * scaleFactor);
    extra::<InitialSize>(&this).m_initialSize.set(fields.m_particleScale.get());
    fields.m_particleMaxAge.set(6);
    this.m_noClip.set(false);
    fields.m_particleTextureIndex.set(64);
    this
}
