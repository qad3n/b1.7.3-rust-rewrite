use crate::entity::{ENTITY, Entity, EntityClass};
use crate::world::World;
use std::rc::Rc;

pub fn init(this: &Rc<Entity>, world: Option<Rc<World>>) { Entity::init(this, world); }

pub const CLASS: EntityClass =
    EntityClass { m_name: "EntityWeatherEffect", m_parent: Some(&ENTITY), m_interfaces: &[], ..ENTITY };
