use crate::render::particle::entity_fx::{self, FXFields, extra, fx};
use crate::render::particle::entity_heart_fx::InitialSize;
use game::entity::{Entity, EntityClass};
use game::world::World;
use jvm::math;
use std::cell::Cell;
use std::rc::Rc;

fn onUpdate(this: &Rc<Entity>)
{
    entity_fx::setPrevPosition(this);
    entity_fx::ageParticle(this);
    let fields: &FXFields = fx(this);
    let f: f32 = fields.m_particleAge.get() as f32 / fields.m_particleMaxAge.get() as f32;
    if this.m_rand.nextFloat() > f
    {
        this.world().spawnParticle(
            "smoke",
            this.posX(),
            this.posY(),
            this.posZ(),
            this.m_motionX.get(),
            this.m_motionY.get(),
            this.m_motionZ.get(),
        );
    }
    this.m_motionY.set(this.m_motionY.get() - 0.03);
    entity_fx::moveByMotion(this);
    entity_fx::scaleMotion(this, 0.999f32 as f64);
    entity_fx::groundFriction(this);
}

pub const CLASS: EntityClass = EntityClass {
    m_name: "EntityLavaFX",
    m_parent: Some(&entity_fx::CLASS),
    getEntityBrightness: |_, _| 1.0,
    renderParticle: |this, tessellator, tickDelta, dx, dy, dz, forwards, sideways| {
        let f: f32 = entity_fx::lifeFraction(this, tickDelta);
        fx(this).m_particleScale.set(extra::<InitialSize>(this).m_initialSize.get() * (1.0 - f * f));
        entity_fx::renderParticle(this, tessellator, tickDelta, dx, dy, dz, forwards, sideways);
    },
    onUpdate,
    ..entity_fx::CLASS
};

pub fn new(world: &Rc<World>, x: f64, y: f64, z: f64) -> Rc<Entity>
{
    let size: InitialSize = InitialSize { m_initialSize: Cell::new(0.0) };
    let this: Rc<Entity> = entity_fx::construct(&CLASS, Box::new(size), world, x, y, z, 0.0, 0.0, 0.0);
    entity_fx::scaleMotion(&this, 0.8f32 as f64);
    this.m_motionY.set((this.m_rand.nextFloat() * 0.4 + 0.05) as f64);
    let fields: &FXFields = fx(&this);
    fields.m_particleRed.set(1.0);
    fields.m_particleGreen.set(1.0);
    fields.m_particleBlue.set(1.0);
    fields.m_particleScale.set(fields.m_particleScale.get() * (this.m_rand.nextFloat() * 2.0 + 0.2));
    extra::<InitialSize>(&this).m_initialSize.set(fields.m_particleScale.get());
    fields.m_particleMaxAge.set((16.0 / (math::random() * 0.8 + 0.2)) as i32);
    this.m_noClip.set(false);
    fields.m_particleTextureIndex.set(49);
    this
}
