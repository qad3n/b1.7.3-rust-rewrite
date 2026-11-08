use crate::entity::{Entity, EntityClass, EntityExt, entity_animal, entity_creature, entity_living};
use crate::item::Item;
use crate::world::World;
use jvm::string::JString;
use std::cell::Cell;
use std::rc::Rc;

#[derive(Default)]
pub struct ChickenExt
{
    pub m_living: entity_living::LivingFields,
    pub m_creature: entity_creature::CreatureFields,
    pub m_sheared: Cell<bool>,
    pub m_flapProgress: Cell<f32>,
    pub m_destPos: Cell<f32>,
    pub m_lastFlapSpeed: Cell<f32>,
    pub m_lastFlapProgress: Cell<f32>,
    pub m_flapping: Cell<f32>,
    pub m_timeUntilNextEgg: Cell<i32>,
}

impl EntityExt for ChickenExt
{
    fn asAny(&self) -> &dyn std::any::Any { self }

    fn living(&self) -> Option<&entity_living::LivingFields> { Some(&self.m_living) }

    fn creature(&self) -> Option<&entity_creature::CreatureFields> { Some(&self.m_creature) }
}

pub fn fields(this: &Entity) -> &ChickenExt { this.ext::<ChickenExt>() }

fn onLivingUpdate(this: &Rc<Entity>)
{
    (entity_animal::CLASS.onLivingUpdate)(this);
    let chicken: &ChickenExt = fields(this);
    chicken.m_lastFlapProgress.set(chicken.m_flapProgress.get());
    chicken.m_lastFlapSpeed.set(chicken.m_destPos.get());
    let step: f64 = if this.m_onGround.get() { -1.0 } else { 4.0 };
    chicken.m_destPos.set((chicken.m_destPos.get() as f64 + step * 0.3) as f32);
    if chicken.m_destPos.get() < 0.0
    {
        chicken.m_destPos.set(0.0);
    }
    if chicken.m_destPos.get() > 1.0
    {
        chicken.m_destPos.set(1.0);
    }
    if !this.m_onGround.get() && chicken.m_flapping.get() < 1.0
    {
        chicken.m_flapping.set(1.0);
    }
    chicken.m_flapping.set((chicken.m_flapping.get() as f64 * 0.9) as f32);
    if !this.m_onGround.get() && this.m_motionY.get() < 0.0
    {
        this.m_motionY.set(this.m_motionY.get() * 0.6);
    }
    chicken.m_flapProgress.set(chicken.m_flapProgress.get() + chicken.m_flapping.get() * 2.0);
    let world: Rc<World> = this.world();
    if !world.m_multiplayerWorld.get()
    {
        chicken.m_timeUntilNextEgg.set(chicken.m_timeUntilNextEgg.get() - 1);
        if chicken.m_timeUntilNextEgg.get() <= 0
        {
            let pitch: f32 = (this.m_rand.nextFloat() - this.m_rand.nextFloat()) * 0.2 + 1.0;
            world.playSoundAtEntity(this, "mob.chickenplop", 1.0, pitch);
            this.dropItem(Item::egg().m_shiftedIndex, 1);
            chicken.m_timeUntilNextEgg.set(this.m_rand.nextIntBounded(6000) + 6000);
        }
    }
}

pub const CLASS: EntityClass = EntityClass {
    m_name: "EntityChicken",
    m_parent: Some(&entity_animal::CLASS),
    onLivingUpdate,
    fall: |_, _| {},
    writeEntityToNBT: |this, nbt| (entity_animal::CLASS.writeEntityToNBT)(this, nbt),
    readEntityFromNBT: |this, nbt| (entity_animal::CLASS.readEntityFromNBT)(this, nbt),
    getLivingSound: |_| Some("mob.chicken".to_string()),
    getHurtSound: |_| Some("mob.chickenhurt".to_string()),
    getDeathSound: |_| Some("mob.chickenhurt".to_string()),
    getDropItemId: |_| Item::feather().m_shiftedIndex,
    ..entity_animal::CLASS
};

pub fn new(world: Option<Rc<World>>) -> Rc<Entity>
{
    let entity: Rc<Entity> = Entity::alloc(&CLASS, Box::new(ChickenExt::default()));
    entity_animal::init(&entity, world);
    let chicken: &ChickenExt = fields(&entity);
    chicken.m_sheared.set(false);
    chicken.m_flapProgress.set(0.0);
    chicken.m_destPos.set(0.0);
    chicken.m_flapping.set(1.0);
    entity_living::fields(&entity).m_texture.set(Some(JString::from("/mob/chicken.png")));
    entity.setSize(0.3, 0.4);
    entity_living::fields(&entity).m_health.set(4);
    chicken.m_timeUntilNextEgg.set(entity.m_rand.nextIntBounded(6000) + 6000);
    entity
}
