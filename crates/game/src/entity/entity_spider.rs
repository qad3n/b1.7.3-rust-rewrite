use crate::entity::entity_mob::MobExt;
use crate::entity::{Entity, EntityClass, entity_creature, entity_living, entity_mob};
use crate::item::Item;
use crate::math::math_helper;
use crate::world::World;
use jvm::string::JString;
use std::rc::Rc;

fn attackEntity(this: &Rc<Entity>, target: &Rc<Entity>, distance: f32)
{
    let f: f32 = this.getEntityBrightness(1.0);
    if f > 0.5 && this.m_rand.nextIntBounded(100) == 0
    {
        entity_creature::fields(this).m_playerToAttack.set(None);
        return;
    }
    if distance > 2.0 && distance < 6.0 && this.m_rand.nextIntBounded(10) == 0
    {
        if this.m_onGround.get()
        {
            let d: f64 = target.posX() - this.posX();
            let d2: f64 = target.posZ() - this.posZ();
            let f2: f32 = math_helper::sqrt_double(d * d + d2 * d2);
            this.m_motionX.set(d / f2 as f64 * 0.5 * 0.8f32 as f64 + this.m_motionX.get() * 0.2f32 as f64);
            this.m_motionZ.set(d2 / f2 as f64 * 0.5 * 0.8f32 as f64 + this.m_motionZ.get() * 0.2f32 as f64);
            this.m_motionY.set(0.4f32 as f64);
        }
    }
    else
    {
        (entity_mob::CLASS.attackEntity)(this, target, distance);
    }
}

pub const CLASS: EntityClass = EntityClass {
    m_name: "EntitySpider",
    m_parent: Some(&entity_mob::CLASS),
    getMountedYOffset: |this| this.m_height.get() as f64 * 0.75 - 0.5,
    canTriggerWalking: |_| false,
    findPlayerToAttack: |this| {
        let f: f32 = this.getEntityBrightness(1.0);
        if f < 0.5
        {
            let d: f64 = 16.0;
            return this.world().getClosestPlayerToEntity(this, d);
        }
        None
    },
    getLivingSound: |_| Some("mob.spider".to_string()),
    getHurtSound: |_| Some("mob.spider".to_string()),
    getDeathSound: |_| Some("mob.spiderdeath".to_string()),
    attackEntity,
    writeEntityToNBT: |this, nbt| (entity_mob::CLASS.writeEntityToNBT)(this, nbt),
    readEntityFromNBT: |this, nbt| (entity_mob::CLASS.readEntityFromNBT)(this, nbt),
    getDropItemId: |_| Item::silk().m_shiftedIndex,
    isOnLadder: |this| this.m_isCollidedHorizontally.get(),
    ..entity_mob::CLASS
};

pub fn new(world: Option<Rc<World>>) -> Rc<Entity>
{
    let entity: Rc<Entity> = Entity::alloc(&CLASS, Box::new(MobExt::default()));
    entity_mob::init(&entity, world);
    let living: &entity_living::LivingFields = entity_living::fields(&entity);
    living.m_texture.set(Some(JString::from("/mob/spider.png")));
    entity.setSize(1.4, 0.9);
    living.m_moveSpeed.set(0.8);
    entity
}
