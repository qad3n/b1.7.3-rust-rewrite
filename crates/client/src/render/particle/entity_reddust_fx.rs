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
    entity_fx::moveByMotion(this);
    if this.posY() == this.m_prevPosY.get()
    {
        this.m_motionX.set(this.m_motionX.get() * 1.1);
        this.m_motionZ.set(this.m_motionZ.get() * 1.1);
    }
    entity_fx::scaleMotion(this, 0.96f32 as f64);
    entity_fx::groundFriction(this);
}

pub const CLASS: EntityClass = EntityClass {
    m_name: "EntityReddustFX",
    m_parent: Some(&entity_fx::CLASS),
    renderParticle: entity_heart_fx::renderGrowing,
    onUpdate,
    ..entity_fx::CLASS
};

fn tint(channel: f32, f: f32) -> f32 { ((math::random() * 0.2f32 as f64) as f32 + 0.8) * channel * f }

pub fn new(world: &Rc<World>, x: f64, y: f64, z: f64, r: f32, g: f32, b: f32) -> Rc<Entity>
{
    newScaled(world, x, y, z, 1.0, r, g, b)
}

pub fn newScaled(world: &Rc<World>, x: f64, y: f64, z: f64, scale: f32, mut r: f32, g: f32, b: f32) -> Rc<Entity>
{
    let size: InitialSize = InitialSize { m_initialSize: Cell::new(0.0) };
    let this: Rc<Entity> = entity_fx::construct(&CLASS, Box::new(size), world, x, y, z, 0.0, 0.0, 0.0);
    entity_fx::scaleMotion(&this, 0.1f32 as f64);
    if r == 0.0
    {
        r = 1.0;
    }
    let f: f32 = math::random() as f32 * 0.4 + 0.6;
    let fields: &FXFields = fx(&this);
    fields.m_particleRed.set(tint(r, f));
    fields.m_particleGreen.set(tint(g, f));
    fields.m_particleBlue.set(tint(b, f));
    fields.m_particleScale.set(fields.m_particleScale.get() * 0.75);
    fields.m_particleScale.set(fields.m_particleScale.get() * scale);
    extra::<InitialSize>(&this).m_initialSize.set(fields.m_particleScale.get());
    fields.m_particleMaxAge.set((8.0 / (math::random() * 0.8 + 0.2)) as i32);
    fields.m_particleMaxAge.set((fields.m_particleMaxAge.get() as f32 * scale) as i32);
    this.m_noClip.set(false);
    this
}
