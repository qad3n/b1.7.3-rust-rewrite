use crate::render::particle::entity_fx::{FXFields, fx};
use crate::render::particle::entity_rain_fx;
use game::entity::{Entity, EntityClass};
use game::world::World;
use std::rc::Rc;

pub const CLASS: EntityClass =
    EntityClass { m_name: "EntitySplashFX", m_parent: Some(&entity_rain_fx::CLASS), ..entity_rain_fx::CLASS };

pub fn new(world: &Rc<World>, x: f64, y: f64, z: f64, velocityX: f64, velocityY: f64, velocityZ: f64) -> Rc<Entity>
{
    let this: Rc<Entity> = entity_rain_fx::construct(&CLASS, world, x, y, z);
    let fields: &FXFields = fx(&this);
    fields.m_particleGravity.set(0.04);
    fields.m_particleTextureIndex.set(fields.m_particleTextureIndex.get() + 1);
    if velocityY == 0.0 && (velocityX != 0.0 || velocityZ != 0.0)
    {
        this.m_motionX.set(velocityX);
        this.m_motionY.set(velocityY + 0.1);
        this.m_motionZ.set(velocityZ);
    }
    this
}
