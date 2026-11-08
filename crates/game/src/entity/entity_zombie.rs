use crate::entity::entity_mob::MobExt;
use crate::entity::{Entity, EntityClass, entity_living, entity_mob};
use crate::item::Item;
use crate::math::math_helper;
use crate::world::World;
use jvm::string::JString;
use std::rc::Rc;

pub fn burnInDaylight(this: &Rc<Entity>)
{
    let world: Rc<World> = this.world();
    if world.isDaytime()
    {
        let f: f32 = this.getEntityBrightness(1.0);
        if f > 0.5
            && world.canBlockSeeTheSky(
                math_helper::floor_double(this.posX()),
                math_helper::floor_double(this.posY()),
                math_helper::floor_double(this.posZ()),
            )
            && this.m_rand.nextFloat() * 30.0 < (f - 0.4) * 2.0
        {
            this.m_fire.set(300);
        }
    }
}

pub const CLASS: EntityClass = EntityClass {
    m_name: "EntityZombie",
    m_parent: Some(&entity_mob::CLASS),
    onLivingUpdate: |this| {
        burnInDaylight(this);
        (entity_mob::CLASS.onLivingUpdate)(this);
    },
    getLivingSound: |_| Some("mob.zombie".to_string()),
    getHurtSound: |_| Some("mob.zombiehurt".to_string()),
    getDeathSound: |_| Some("mob.zombiedeath".to_string()),
    getDropItemId: |_| Item::feather().m_shiftedIndex,
    ..entity_mob::CLASS
};

pub fn init(this: &Rc<Entity>, world: Option<Rc<World>>)
{
    entity_mob::init(this, world);
    entity_living::fields(this).m_texture.set(Some(JString::from("/mob/zombie.png")));
    entity_living::fields(this).m_moveSpeed.set(0.5);
    entity_mob::fields(this).m_attackStrength.set(5);
}

pub fn new(world: Option<Rc<World>>) -> Rc<Entity>
{
    let entity: Rc<Entity> = Entity::alloc(&CLASS, Box::new(MobExt::default()));
    init(&entity, world);
    entity
}
