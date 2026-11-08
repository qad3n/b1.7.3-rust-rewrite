use crate::entity::entity_cow::AnimalExt;
use crate::entity::{
    Entity, EntityClass, WatchedValue, entity_animal, entity_living, entity_pig_zombie, entity_player,
};
use crate::item::Item;
use crate::stats::AchievementList;
use crate::world::World;
use jvm::string::JString;
use std::rc::Rc;

pub fn getSaddled(this: &Rc<Entity>) -> bool { (this.m_dataWatcher.getWatchableObjectByte(16) & 1) != 0 }

pub fn setSaddled(this: &Rc<Entity>, saddled: bool)
{
    this.m_dataWatcher.updateObject(16, WatchedValue::Byte(if saddled { 1 } else { 0 }));
}

fn interact(this: &Rc<Entity>, player: &Rc<Entity>) -> bool
{
    let rider: Option<Rc<Entity>> = this.m_riddenByEntity.get();
    if getSaddled(this)
        && !this.world().m_multiplayerWorld.get()
        && rider.as_ref().is_none_or(|rider| Rc::ptr_eq(rider, player))
    {
        player.mountEntity(Some(this));
        return true;
    }
    false
}

fn onStruckByLightning(this: &Rc<Entity>, _lightning: &Rc<Entity>)
{
    let world: Rc<World> = this.world();
    if world.m_multiplayerWorld.get()
    {
        return;
    }
    let entityPigZombie: Rc<Entity> = entity_pig_zombie::new(Some(Rc::clone(&world)));
    entityPigZombie.setLocationAndAngles(
        this.posX(),
        this.posY(),
        this.posZ(),
        this.m_rotationYaw.get(),
        this.m_rotationPitch.get(),
    );
    world.entityJoinedWorld(&entityPigZombie);
    this.setEntityDead();
}

fn fall(this: &Rc<Entity>, distance: f32)
{
    (entity_animal::CLASS.fall)(this, distance);
    if distance > 5.0
        && let Some(rider) = this.m_riddenByEntity.get()
        && rider.instanceOf("EntityPlayer")
    {
        entity_player::triggerAchievement(&rider, AchievementList::flyPig());
    }
}

pub const CLASS: EntityClass = EntityClass {
    m_name: "EntityPig",
    m_parent: Some(&entity_animal::CLASS),
    entityInit: |this| this.m_dataWatcher.addObject(16, WatchedValue::Byte(0)),
    writeEntityToNBT: |this, nbt| {
        (entity_animal::CLASS.writeEntityToNBT)(this, nbt);
        nbt.setBoolean("Saddle", getSaddled(this));
    },
    readEntityFromNBT: |this, nbt| {
        (entity_animal::CLASS.readEntityFromNBT)(this, nbt);
        setSaddled(this, nbt.getBoolean("Saddle"));
    },
    getLivingSound: |_| Some("mob.pig".to_string()),
    getHurtSound: |_| Some("mob.pig".to_string()),
    getDeathSound: |_| Some("mob.pigdeath".to_string()),
    interact,
    getDropItemId: |this| {
        if this.m_fire.get() > 0
        {
            return Item::porkCooked().m_shiftedIndex;
        }
        Item::porkRaw().m_shiftedIndex
    },
    onStruckByLightning,
    fall,
    ..entity_animal::CLASS
};

pub fn new(world: Option<Rc<World>>) -> Rc<Entity>
{
    let entity: Rc<Entity> = Entity::alloc(&CLASS, Box::new(AnimalExt::default()));
    entity_animal::init(&entity, world);
    entity_living::fields(&entity).m_texture.set(Some(JString::from("/mob/pig.png")));
    entity.setSize(0.9, 0.9);
    entity
}
