use crate::render::particle::entity_fx::{self, FXFields, extra, fx};
use game::block::Block;
use game::entity::{Entity, EntityClass};
use game::world::World;
use std::rc::Rc;

pub struct DiggingFields
{
    m_block: Rc<Block>,
    pub m_face: i32,
}

pub const CLASS: EntityClass = EntityClass {
    m_name: "EntityDiggingFX",
    m_parent: Some(&entity_fx::CLASS),
    getFXLayer: |_| 1,
    renderParticle: entity_fx::renderJitteredParticle,
    ..entity_fx::CLASS
};

pub fn new(
    world: &Rc<World>,
    x: f64,
    y: f64,
    z: f64,
    velocityX: f64,
    velocityY: f64,
    velocityZ: f64,
    block: &Rc<Block>,
    face: i32,
    metadata: i32,
) -> Rc<Entity>
{
    let digging: DiggingFields = DiggingFields { m_block: Rc::clone(block), m_face: face };
    let this: Rc<Entity> =
        entity_fx::construct(&CLASS, Box::new(digging), world, x, y, z, velocityX, velocityY, velocityZ);
    let fields: &FXFields = fx(&this);
    fields.m_particleTextureIndex.set(block.getBlockTextureFromSideAndMetadata(0, metadata));
    fields.m_particleGravity.set(block.m_blockParticleGravity.get());
    fields.m_particleRed.set(0.6);
    fields.m_particleGreen.set(0.6);
    fields.m_particleBlue.set(0.6);
    fields.m_particleScale.set(fields.m_particleScale.get() / 2.0);
    this
}

pub fn init(this: &Rc<Entity>, dx: i32, dy: i32, dz: i32) -> Rc<Entity>
{
    let block: &Rc<Block> = &extra::<DiggingFields>(this).m_block;
    if Rc::ptr_eq(block, &Block::grass())
    {
        return Rc::clone(this);
    }
    let i: i32 = block.colorMultiplier(&*this.world(), dx, dy, dz);
    let fields: &FXFields = fx(this);
    fields.m_particleRed.set(fields.m_particleRed.get() * ((i >> 16 & 0xFF) as f32 / 255.0));
    fields.m_particleGreen.set(fields.m_particleGreen.get() * ((i >> 8 & 0xFF) as f32 / 255.0));
    fields.m_particleBlue.set(fields.m_particleBlue.get() * ((i & 0xFF) as f32 / 255.0));
    Rc::clone(this)
}
