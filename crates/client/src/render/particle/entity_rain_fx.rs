use crate::render::particle::entity_fx::{self, FXFields, fx};
use game::block::{Material, block_fluid};
use game::entity::{Entity, EntityClass};
use game::math::math_helper::floor_double;
use game::world::World;
use jvm::math;
use std::rc::Rc;

fn onUpdate(this: &Rc<Entity>)
{
    entity_fx::setPrevPosition(this);
    let fields: &FXFields = fx(this);
    this.m_motionY.set(this.m_motionY.get() - fields.m_particleGravity.get() as f64);
    entity_fx::moveByMotion(this);
    entity_fx::scaleMotion(this, 0.98f32 as f64);
    let maxAge: i32 = fields.m_particleMaxAge.get();
    fields.m_particleMaxAge.set(maxAge - 1);
    if maxAge <= 0
    {
        this.setEntityDead();
    }
    if this.m_onGround.get()
    {
        if math::random() < 0.5
        {
            this.setEntityDead();
        }
        this.m_motionX.set(this.m_motionX.get() * 0.7f32 as f64);
        this.m_motionZ.set(this.m_motionZ.get() * 0.7f32 as f64);
    }
    let world: Rc<World> = this.world();
    let x: i32 = floor_double(this.posX());
    let y: i32 = floor_double(this.posY());
    let z: i32 = floor_double(this.posZ());
    let material: Rc<Material> = world.getBlockMaterial(x, y, z);
    if material.getIsLiquid() || material.isSolid()
    {
        let d0: f64 = ((y + 1) as f32 - block_fluid::getPercentAir(world.getBlockMetadata(x, y, z))) as f64;
        if this.posY() < d0
        {
            this.setEntityDead();
        }
    }
}

pub const CLASS: EntityClass = EntityClass {
    m_name: "EntityRainFX",
    m_parent: Some(&entity_fx::CLASS),
    renderParticle: |this, tessellator, tickDelta, dx, dy, dz, forwards, sideways| {
        (entity_fx::CLASS.renderParticle)(this, tessellator, tickDelta, dx, dy, dz, forwards, sideways)
    },
    onUpdate,
    ..entity_fx::CLASS
};

pub fn construct(class: &'static EntityClass, world: &Rc<World>, x: f64, y: f64, z: f64) -> Rc<Entity>
{
    let this: Rc<Entity> = entity_fx::construct(class, Box::new(()), world, x, y, z, 0.0, 0.0, 0.0);
    this.m_motionX.set(this.m_motionX.get() * 0.3f32 as f64);
    this.m_motionY.set((math::random() as f32 * 0.2 + 0.1) as f64);
    this.m_motionZ.set(this.m_motionZ.get() * 0.3f32 as f64);
    let fields: &FXFields = fx(&this);
    fields.m_particleRed.set(1.0);
    fields.m_particleGreen.set(1.0);
    fields.m_particleBlue.set(1.0);
    fields.m_particleTextureIndex.set(19 + this.m_rand.nextIntBounded(4));
    this.setSize(0.01, 0.01);
    fields.m_particleGravity.set(0.06);
    fields.m_particleMaxAge.set((8.0 / (math::random() * 0.8 + 0.2)) as i32);
    this
}

pub fn new(world: &Rc<World>, x: f64, y: f64, z: f64) -> Rc<Entity> { construct(&CLASS, world, x, y, z) }
