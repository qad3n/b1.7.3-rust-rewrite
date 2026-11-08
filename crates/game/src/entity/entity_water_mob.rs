use crate::entity::{Entity, EntityClass, entity_creature};
use crate::world::World;
use std::rc::Rc;

pub fn init(this: &Rc<Entity>, world: Option<Rc<World>>) { entity_creature::init(this, world); }

pub const CLASS: EntityClass = EntityClass {
    m_name: "EntityWaterMob",
    m_parent: Some(&entity_creature::CLASS),
    m_interfaces: &["IAnimals"],
    canBreatheUnderwater: |_| true,
    writeEntityToNBT: |this, nbt| (entity_creature::CLASS.writeEntityToNBT)(this, nbt),
    readEntityFromNBT: |this, nbt| (entity_creature::CLASS.readEntityFromNBT)(this, nbt),
    getCanSpawnHere: |this| this.world().checkIfAABBIsClear(&this.m_boundingBox),
    getTalkInterval: |_| 120,
    ..entity_creature::CLASS
};
