use crate::render::particle::entity_explode_fx::animateTexture;
use crate::render::particle::entity_fx::{self, FXFields, extra, fx};
use crate::render::particle::entity_heart_fx::{self, InitialSize};
use game::entity::{Entity, EntityClass};
use game::world::World;
use jvm::math;
use std::cell::Cell;
use std::rc::Rc;

// rewrite: 1.7's EntityCritFX, a grey speck that browns as green and blue fade each tick
mod crit
{
    pub const KEEP_MOTION: f32 = 0.1;
    pub const PUSH: f64 = 0.4;
    pub const GREY_SPREAD: f32 = 0.3;
    pub const GREY_LOW: f32 = 0.6;
    pub const SIZE: f32 = 0.75;
    pub const AGE: f64 = 6.0;
    pub const AGE_SPREAD: f64 = 0.8;
    pub const AGE_LOW: f64 = 0.6;
    pub const GREEN_FADE: f64 = 0.96;
    pub const BLUE_FADE: f64 = 0.9;
    pub const DRAG: f32 = 0.7;
    pub const FALL: f32 = 0.02;
}

// beta's sheet has no crit sprite so it keeps the snow shovel frames
fn onUpdate(this: &Rc<Entity>)
{
    entity_fx::setPrevPosition(this);
    entity_fx::ageParticle(this);
    animateTexture(this);
    entity_fx::moveByMotion(this);

    let fields: &FXFields = fx(this);
    fields.m_particleGreen.set((f64::from(fields.m_particleGreen.get()) * crit::GREEN_FADE) as f32);
    fields.m_particleBlue.set((f64::from(fields.m_particleBlue.get()) * crit::BLUE_FADE) as f32);

    entity_fx::scaleMotion(this, f64::from(crit::DRAG));
    this.m_motionY.set(this.m_motionY.get() - f64::from(crit::FALL));
    entity_fx::groundFriction(this);
}

pub const CLASS: EntityClass = EntityClass
{
    m_name: "EntityCritFX",
    m_parent: Some(&entity_fx::CLASS),
    renderParticle: entity_heart_fx::renderGrowing,
    onUpdate,
    ..entity_fx::CLASS
};

pub fn new(world: &Rc<World>, x: f64, y: f64, z: f64, velocityX: f64, velocityY: f64, velocityZ: f64) -> Rc<Entity>
{
    let size: InitialSize = InitialSize { m_initialSize: Cell::new(0.0) };
    let this: Rc<Entity> = entity_fx::construct(&CLASS, Box::new(size), world, x, y, z, 0.0, 0.0, 0.0);
    entity_fx::scaleMotion(&this, f64::from(crit::KEEP_MOTION));
    this.m_motionX.set(this.m_motionX.get() + velocityX * crit::PUSH);
    this.m_motionY.set(this.m_motionY.get() + velocityY * crit::PUSH);
    this.m_motionZ.set(this.m_motionZ.get() + velocityZ * crit::PUSH);

    let fields: &FXFields = fx(&this);
    let grey: f32 = (math::random() * f64::from(crit::GREY_SPREAD) + f64::from(crit::GREY_LOW)) as f32;
    fields.m_particleRed.set(grey);
    fields.m_particleGreen.set(grey);
    fields.m_particleBlue.set(grey);

    fields.m_particleScale.set(fields.m_particleScale.get() * crit::SIZE);
    extra::<InitialSize>(&this).m_initialSize.set(fields.m_particleScale.get());
    fields.m_particleMaxAge.set((crit::AGE / (math::random() * crit::AGE_SPREAD + crit::AGE_LOW)) as i32);
    this.m_noClip.set(false);

    // the 1.7 constructor ends with one update
    onUpdate(&this);
    this
}
