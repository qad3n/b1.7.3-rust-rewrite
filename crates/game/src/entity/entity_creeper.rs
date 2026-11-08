use crate::entity::entity_mob::MobFields;
use crate::entity::{Entity, EntityClass, EntityExt, WatchedValue, entity_creature, entity_living, entity_mob};
use crate::item::Item;
use crate::world::World;
use jvm::string::JString;
use std::cell::Cell;
use std::rc::Rc;

const FUSE_TICKS: i32 = 30;

#[derive(Default)]
pub struct CreeperExt
{
    pub m_living: entity_living::LivingFields,
    pub m_creature: entity_creature::CreatureFields,
    pub m_mob: MobFields,
    m_timeSinceIgnited: Cell<i32>,
    m_lastActiveTime: Cell<i32>,
}

impl EntityExt for CreeperExt
{
    fn asAny(&self) -> &dyn std::any::Any { self }

    fn living(&self) -> Option<&entity_living::LivingFields> { Some(&self.m_living) }

    fn creature(&self) -> Option<&entity_creature::CreatureFields> { Some(&self.m_creature) }

    fn mob(&self) -> Option<&MobFields> { Some(&self.m_mob) }
}

fn fields(this: &Entity) -> &CreeperExt { this.ext::<CreeperExt>() }

fn getCreeperState(this: &Entity) -> i32 { this.m_dataWatcher.getWatchableObjectByte(16) as i32 }

fn setCreeperState(this: &Entity, dir: i32) { this.m_dataWatcher.updateObject(16, WatchedValue::Byte(dir as i8)); }

pub fn getPowered(this: &Rc<Entity>) -> bool { this.m_dataWatcher.getWatchableObjectByte(17) == 1 }

pub fn setCreeperFlashTime(this: &Rc<Entity>, tickDelta: f32) -> f32
{
    let creeper: &CreeperExt = fields(this);
    (creeper.m_lastActiveTime.get() as f32
        + (creeper.m_timeSinceIgnited.get() - creeper.m_lastActiveTime.get()) as f32 * tickDelta)
        / 28.0
}

fn coolDown(this: &Entity)
{
    let creeper: &CreeperExt = fields(this);
    setCreeperState(this, -1);
    creeper.m_timeSinceIgnited.set(creeper.m_timeSinceIgnited.get() - 1);
    if creeper.m_timeSinceIgnited.get() < 0
    {
        creeper.m_timeSinceIgnited.set(0);
    }
}

fn onUpdate(this: &Rc<Entity>)
{
    let creeper: &CreeperExt = fields(this);
    creeper.m_lastActiveTime.set(creeper.m_timeSinceIgnited.get());
    let world: Rc<World> = this.world();
    if world.m_multiplayerWorld.get()
    {
        let n: i32 = getCreeperState(this);
        if n > 0 && creeper.m_timeSinceIgnited.get() == 0
        {
            world.playSoundAtEntity(this, "random.fuse", 1.0, 0.5);
        }
        creeper.m_timeSinceIgnited.set(creeper.m_timeSinceIgnited.get() + n);
        if creeper.m_timeSinceIgnited.get() < 0
        {
            creeper.m_timeSinceIgnited.set(0);
        }
        if creeper.m_timeSinceIgnited.get() >= FUSE_TICKS
        {
            creeper.m_timeSinceIgnited.set(FUSE_TICKS);
        }
    }
    (entity_mob::CLASS.onUpdate)(this);
    if entity_creature::fields(this).m_playerToAttack.with(Option::is_none) && creeper.m_timeSinceIgnited.get() > 0
    {
        coolDown(this);
    }
}

fn attackEntity(this: &Rc<Entity>, _target: &Rc<Entity>, distance: f32)
{
    let world: Rc<World> = this.world();
    if world.m_multiplayerWorld.get()
    {
        return;
    }
    let creeper: &CreeperExt = fields(this);
    let n: i32 = getCreeperState(this);
    if n <= 0 && distance < 3.0 || n > 0 && distance < 7.0
    {
        if creeper.m_timeSinceIgnited.get() == 0
        {
            world.playSoundAtEntity(this, "random.fuse", 1.0, 0.5);
        }
        setCreeperState(this, 1);
        creeper.m_timeSinceIgnited.set(creeper.m_timeSinceIgnited.get() + 1);
        if creeper.m_timeSinceIgnited.get() >= FUSE_TICKS
        {
            let power: f32 = if getPowered(this) { 6.0 } else { 3.0 };
            world.createExplosion(Some(this), this.posX(), this.posY(), this.posZ(), power);
            this.setEntityDead();
        }
        entity_creature::fields(this).m_hasAttacked.set(true);
    }
    else
    {
        coolDown(this);
    }
}

pub const CLASS: EntityClass = EntityClass {
    m_name: "EntityCreeper",
    m_parent: Some(&entity_mob::CLASS),
    entityInit: |this| {
        (entity_mob::CLASS.entityInit)(this);
        this.m_dataWatcher.addObject(16, WatchedValue::Byte(-1));
        this.m_dataWatcher.addObject(17, WatchedValue::Byte(0));
    },
    writeEntityToNBT: |this, nbt| {
        (entity_mob::CLASS.writeEntityToNBT)(this, nbt);
        if this.m_dataWatcher.getWatchableObjectByte(17) == 1
        {
            nbt.setBoolean("powered", true);
        }
    },
    readEntityFromNBT: |this, nbt| {
        (entity_mob::CLASS.readEntityFromNBT)(this, nbt);
        this.m_dataWatcher.updateObject(17, WatchedValue::Byte(if nbt.getBoolean("powered") { 1 } else { 0 }));
    },
    attackBlockedEntity: |this, _, _| {
        if this.world().m_multiplayerWorld.get()
        {
            return;
        }
        if fields(this).m_timeSinceIgnited.get() > 0
        {
            coolDown(this);
        }
    },
    onUpdate,
    getHurtSound: |_| Some("mob.creeper".to_string()),
    getDeathSound: |_| Some("mob.creeperdeath".to_string()),
    onDeath: |this, killer| {
        (entity_mob::CLASS.onDeath)(this, killer);
        if killer.is_some_and(|killer| killer.instanceOf("EntitySkeleton"))
        {
            this.dropItem(Item::record13().m_shiftedIndex + this.m_rand.nextIntBounded(2), 1);
        }
    },
    attackEntity,
    getDropItemId: |_| Item::gunpowder().m_shiftedIndex,
    onStruckByLightning: |this, lightning| {
        (entity_mob::CLASS.onStruckByLightning)(this, lightning);
        this.m_dataWatcher.updateObject(17, WatchedValue::Byte(1));
    },
    ..entity_mob::CLASS
};

pub fn new(world: Option<Rc<World>>) -> Rc<Entity>
{
    let entity: Rc<Entity> = Entity::alloc(&CLASS, Box::new(CreeperExt::default()));
    entity_mob::init(&entity, world);
    entity_living::fields(&entity).m_texture.set(Some(JString::from("/mob/creeper.png")));
    entity
}
