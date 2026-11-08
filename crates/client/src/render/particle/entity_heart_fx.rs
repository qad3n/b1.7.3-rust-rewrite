use crate::render::particle::entity_fx::{self, FXFields, extra, fx};
use game::entity::{Entity, EntityClass};
use game::world::World;
use std::cell::Cell;
use std::rc::Rc;

pub struct InitialSize
{
    pub m_initialSize: Cell<f32>,
}

pub fn renderGrowing(
    this: &Rc<Entity>,
    target: &dyn std::any::Any,
    tickDelta: f32,
    dx: f32,
    dy: f32,
    dz: f32,
    forwards: f32,
    sideways: f32,
)
{
    // the game crate entity vtable passes the tessellator as Any so port downcasts it
    let tessellator: &crate::render::Tessellator = crate::port::tessellator(target);
    entity_fx::growScale(this, extra::<InitialSize>(this).m_initialSize.get(), tickDelta);
    entity_fx::renderParticle(this, tessellator, tickDelta, dx, dy, dz, forwards, sideways);
}

pub fn floatUpward(this: &Rc<Entity>, drag: f32)
{
    entity_fx::setPrevPosition(this);
    entity_fx::ageParticle(this);
    entity_fx::moveByMotion(this);
    if this.posY() == this.m_prevPosY.get()
    {
        this.m_motionX.set(this.m_motionX.get() * 1.1);
        this.m_motionZ.set(this.m_motionZ.get() * 1.1);
    }
    entity_fx::scaleMotion(this, drag as f64);
    entity_fx::groundFriction(this);
}

pub const CLASS: EntityClass = EntityClass {
    m_name: "EntityHeartFX",
    m_parent: Some(&entity_fx::CLASS),
    renderParticle: renderGrowing,
    onUpdate: |this| floatUpward(this, 0.86),
    ..entity_fx::CLASS
};

pub fn new(world: &Rc<World>, x: f64, y: f64, z: f64, velocityX: f64, velocityY: f64, velocityZ: f64) -> Rc<Entity>
{
    newScaled(world, x, y, z, velocityX, velocityY, velocityZ, 2.0)
}

pub fn newScaled(
    world: &Rc<World>,
    x: f64,
    y: f64,
    z: f64,
    _velocityX: f64,
    _velocityY: f64,
    _velocityZ: f64,
    scaleFactor: f32,
) -> Rc<Entity>
{
    let size: InitialSize = InitialSize { m_initialSize: Cell::new(0.0) };
    let this: Rc<Entity> = entity_fx::construct(&CLASS, Box::new(size), world, x, y, z, 0.0, 0.0, 0.0);
    entity_fx::scaleMotion(&this, 0.01f32 as f64);
    this.m_motionY.set(this.m_motionY.get() + 0.1);
    let fields: &FXFields = fx(&this);
    fields.m_particleScale.set(fields.m_particleScale.get() * 0.75);
    fields.m_particleScale.set(fields.m_particleScale.get() * scaleFactor);
    extra::<InitialSize>(&this).m_initialSize.set(fields.m_particleScale.get());
    fields.m_particleMaxAge.set(16);
    this.m_noClip.set(false);
    fields.m_particleTextureIndex.set(80);
    this
}
