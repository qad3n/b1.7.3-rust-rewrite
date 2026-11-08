use crate::render::particle::entity_fx::{self, FXFields, extra, fx};
use game::entity::{ENTITY, Entity, EntityClass};
use game::world::World;
use jvm::math;
use std::cell::Cell;
use std::rc::Rc;

pub struct PortalFields
{
    m_initialSize: Cell<f32>,
    m_initialX: Cell<f64>,
    m_initialY: Cell<f64>,
    m_initialZ: Cell<f64>,
}

fn onUpdate(this: &Rc<Entity>)
{
    entity_fx::setPrevPosition(this);
    let fields: &FXFields = fx(this);
    let portal: &PortalFields = extra::<PortalFields>(this);
    let mut f: f32 = fields.m_particleAge.get() as f32 / fields.m_particleMaxAge.get() as f32;
    let f1: f32 = f;
    f = -f + f * f * 2.0;
    f = 1.0 - f;
    this.m_posX.set(portal.m_initialX.get() + this.m_motionX.get() * f as f64);
    this.m_posY.set(portal.m_initialY.get() + this.m_motionY.get() * f as f64 + (1.0 - f1) as f64);
    this.m_posZ.set(portal.m_initialZ.get() + this.m_motionZ.get() * f as f64);
    entity_fx::ageParticle(this);
}

pub const CLASS: EntityClass = EntityClass {
    m_name: "EntityPortalFX",
    m_parent: Some(&entity_fx::CLASS),
    renderParticle: |this, tessellator, tickDelta, dx, dy, dz, forwards, sideways| {
        let mut f: f32 = entity_fx::lifeFraction(this, tickDelta);
        f = 1.0 - f;
        f *= f;
        f = 1.0 - f;
        fx(this).m_particleScale.set(extra::<PortalFields>(this).m_initialSize.get() * f);
        entity_fx::renderParticle(this, tessellator, tickDelta, dx, dy, dz, forwards, sideways);
    },
    getEntityBrightness: |this, tickDelta| {
        let f: f32 = (ENTITY.getEntityBrightness)(this, tickDelta);
        let fields: &FXFields = fx(this);
        let mut f1: f32 = fields.m_particleAge.get() as f32 / fields.m_particleMaxAge.get() as f32;
        f1 *= f1;
        f1 *= f1;
        f * (1.0 - f1) + f1
    },
    onUpdate,
    ..entity_fx::CLASS
};

pub fn new(world: &Rc<World>, x: f64, y: f64, z: f64, velocityX: f64, velocityY: f64, velocityZ: f64) -> Rc<Entity>
{
    let portal: PortalFields = PortalFields {
        m_initialSize: Cell::new(0.0),
        m_initialX: Cell::new(0.0),
        m_initialY: Cell::new(0.0),
        m_initialZ: Cell::new(0.0),
    };
    let this: Rc<Entity> =
        entity_fx::construct(&CLASS, Box::new(portal), world, x, y, z, velocityX, velocityY, velocityZ);
    this.m_motionX.set(velocityX);
    this.m_motionY.set(velocityY);
    this.m_motionZ.set(velocityZ);
    let portal: &PortalFields = extra::<PortalFields>(&this);
    this.m_posX.set(x);
    portal.m_initialX.set(x);
    this.m_posY.set(y);
    portal.m_initialY.set(y);
    this.m_posZ.set(z);
    portal.m_initialZ.set(z);
    let f: f32 = this.m_rand.nextFloat() * 0.6 + 0.4;
    let fields: &FXFields = fx(&this);
    fields.m_particleScale.set(this.m_rand.nextFloat() * 0.2 + 0.5);
    portal.m_initialSize.set(fields.m_particleScale.get());
    fields.m_particleRed.set(1.0 * f);
    fields.m_particleGreen.set(1.0 * f);
    fields.m_particleBlue.set(1.0 * f);
    fields.m_particleGreen.set(fields.m_particleGreen.get() * 0.3);
    fields.m_particleRed.set(fields.m_particleRed.get() * 0.9);
    fields.m_particleMaxAge.set((math::random() * 10.0) as i32 + 40);
    this.m_noClip.set(true);
    fields.m_particleTextureIndex.set((math::random() * 8.0) as i32);
    this
}
