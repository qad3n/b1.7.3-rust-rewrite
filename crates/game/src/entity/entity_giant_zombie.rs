use crate::entity::entity_mob::MobExt;
use crate::entity::{Entity, EntityClass, entity_living, entity_mob};
use crate::world::World;
use jvm::string::JString;
use std::rc::Rc;

pub const CLASS: EntityClass = EntityClass {
    m_name: "EntityGiantZombie",
    m_parent: Some(&entity_mob::CLASS),
    getBlockPathWeight: |this, x, y, z| this.world().getLightBrightness(x, y, z) - 0.5,
    ..entity_mob::CLASS
};

pub fn new(world: Option<Rc<World>>) -> Rc<Entity>
{
    let entity: Rc<Entity> = Entity::alloc(&CLASS, Box::new(MobExt::default()));
    entity_mob::init(&entity, world);
    let living: &entity_living::LivingFields = entity_living::fields(&entity);
    living.m_texture.set(Some(JString::from("/mob/zombie.png")));
    living.m_moveSpeed.set(0.5);
    entity_mob::fields(&entity).m_attackStrength.set(50);
    living.m_health.set(living.m_health.get() * 10);
    entity.m_yOffset.set(entity.m_yOffset.get() * 6.0);
    entity.setSize(entity.m_width.get() * 6.0, entity.m_height.get() * 6.0);
    entity
}
