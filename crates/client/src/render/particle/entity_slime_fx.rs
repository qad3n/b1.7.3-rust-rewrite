use crate::render::particle::entity_fx::{self, FXFields, fx};
use game::block::Block;
use game::entity::{Entity, EntityClass};
use game::item::Item;
use game::world::World;
use std::rc::Rc;

pub const CLASS: EntityClass = EntityClass {
    m_name: "EntitySlimeFX",
    m_parent: Some(&entity_fx::CLASS),
    getFXLayer: |_| 2,
    renderParticle: entity_fx::renderJitteredParticle,
    ..entity_fx::CLASS
};

pub fn new(world: &Rc<World>, x: f64, y: f64, z: f64, item: &Rc<Item>) -> Rc<Entity>
{
    let this: Rc<Entity> = entity_fx::construct(&CLASS, Box::new(()), world, x, y, z, 0.0, 0.0, 0.0);
    let fields: &FXFields = fx(&this);
    fields.m_particleTextureIndex.set(item.getIconFromDamage(0));
    fields.m_particleRed.set(1.0);
    fields.m_particleGreen.set(1.0);
    fields.m_particleBlue.set(1.0);
    fields.m_particleGravity.set(Block::blockSnow().m_blockParticleGravity.get());
    fields.m_particleScale.set(fields.m_particleScale.get() / 2.0);
    this
}
