use crate::block::Block;
use crate::entity::{Entity, EntityClass, entity_creature};
use crate::math::math_helper;
use crate::world::World;
use std::rc::Rc;

pub fn init(this: &Rc<Entity>, world: Option<Rc<World>>) { entity_creature::init(this, world); }

pub const CLASS: EntityClass = EntityClass {
    m_name: "EntityAnimal",
    m_parent: Some(&entity_creature::CLASS),
    m_interfaces: &["IAnimals"],
    getBlockPathWeight: |this, x, y, z| {
        let world: Rc<World> = this.world();
        if world.getBlockId(x, y - 1, z) == Block::grass().m_blockID
        {
            return 10.0;
        }
        world.getLightBrightness(x, y, z) - 0.5
    },
    writeEntityToNBT: |this, nbt| (entity_creature::CLASS.writeEntityToNBT)(this, nbt),
    readEntityFromNBT: |this, nbt| (entity_creature::CLASS.readEntityFromNBT)(this, nbt),
    getCanSpawnHere: |this| {
        let world: Rc<World> = this.world();
        let x: i32 = math_helper::floor_double(this.posX());
        let y: i32 = math_helper::floor_double(this.m_boundingBox.minY());
        let z: i32 = math_helper::floor_double(this.posZ());
        world.getBlockId(x, y - 1, z) == Block::grass().m_blockID
            && world.getFullBlockLightValue(x, y, z) > 8
            && (entity_creature::CLASS.getCanSpawnHere)(this)
    },
    getTalkInterval: |_| 120,
    ..entity_creature::CLASS
};
