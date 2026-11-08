use crate::entity::{Entity, EntityClass, EntityExt, entity_creature, entity_living};
use crate::math::math_helper;
use crate::port::sameObject;
use crate::world::{EnumSkyBlock, World};
use std::cell::Cell;
use std::rc::Rc;

#[derive(Default)]
pub struct MobFields
{
    pub m_attackStrength: Cell<i32>,
}

pub fn fields(this: &Entity) -> &MobFields
{
    match this.m_ext.mob()
    {
        Some(fields) => fields,
        None => jvm::throwable::throw(
            &jvm::throwable::CLASS_CAST,
            &format!("net.minecraft.src.{} cannot be cast to net.minecraft.src.EntityMob", this.className()),
        ),
    }
}

pub fn init(this: &Rc<Entity>, world: Option<Rc<World>>)
{
    entity_creature::init(this, world);
    fields(this).m_attackStrength.set(2);
    entity_living::fields(this).m_health.set(20);
}

fn getCanSpawnHere(this: &Rc<Entity>) -> bool
{
    let world: Rc<World> = this.world();
    let i: i32 = math_helper::floor_double(this.posX());
    let j: i32 = math_helper::floor_double(this.m_boundingBox.minY());
    let k: i32 = math_helper::floor_double(this.posZ());
    if world.getSavedLightValue(EnumSkyBlock::Sky, i, j, k) > this.m_rand.nextIntBounded(32)
    {
        return false;
    }
    let mut l: i32 = world.getBlockLightValue(i, j, k);
    if world.isThundering()
    {
        let i1: i32 = world.m_skylightSubtracted.get();
        world.m_skylightSubtracted.set(10);
        l = world.getBlockLightValue(i, j, k);
        world.m_skylightSubtracted.set(i1);
    }
    l <= this.m_rand.nextIntBounded(8) && (entity_creature::CLASS.getCanSpawnHere)(this)
}

pub const CLASS: EntityClass = EntityClass {
    m_name: "EntityMob",
    m_parent: Some(&entity_creature::CLASS),
    m_interfaces: &["IMob", "IAnimals"],
    onLivingUpdate: |this| {
        let f: f32 = this.getEntityBrightness(1.0);
        if f > 0.5
        {
            let living: &entity_living::LivingFields = entity_living::fields(this);
            living.m_entityAge.set(living.m_entityAge.get() + 2);
        }
        (entity_creature::CLASS.onLivingUpdate)(this);
    },
    onUpdate: |this| {
        (entity_creature::CLASS.onUpdate)(this);
        let world: Rc<World> = this.world();
        if !world.m_multiplayerWorld.get() && world.m_difficultySetting.get() == 0
        {
            this.setEntityDead();
        }
    },
    findPlayerToAttack: |this| {
        let entityPlayer: Option<Rc<Entity>> = this.world().getClosestPlayerToEntity(this, 16.0);
        entityPlayer.filter(|player| entity_living::canEntityBeSeen(this, player))
    },
    attackEntityFrom: |this, source, amount| {
        if (entity_creature::CLASS.attackEntityFrom)(this, source, amount)
        {
            // java compares references here and either side may be null
            if sameObject(this.m_riddenByEntity.get().as_ref(), source)
                || sameObject(this.m_ridingEntity.get().as_ref(), source)
            {
                return true;
            }
            if !source.is_some_and(|source| Rc::ptr_eq(source, this))
            {
                entity_creature::fields(this).m_playerToAttack.set(source.cloned());
            }
            return true;
        }
        false
    },
    attackEntity: |this, target, distance| {
        let living: &entity_living::LivingFields = entity_living::fields(this);
        if living.m_attackTime.get() <= 0
            && distance < 2.0
            && target.m_boundingBox.maxY() > this.m_boundingBox.minY()
            && target.m_boundingBox.minY() < this.m_boundingBox.maxY()
        {
            living.m_attackTime.set(20);
            target.attackEntityFrom(Some(this), fields(this).m_attackStrength.get());
        }
    },
    getBlockPathWeight: |this, x, y, z| 0.5 - this.world().getLightBrightness(x, y, z),
    writeEntityToNBT: |this, nbt| (entity_creature::CLASS.writeEntityToNBT)(this, nbt),
    readEntityFromNBT: |this, nbt| (entity_creature::CLASS.readEntityFromNBT)(this, nbt),
    getCanSpawnHere,
    ..entity_creature::CLASS
};

#[derive(Default)]
pub struct MobExt
{
    pub m_living: entity_living::LivingFields,
    pub m_creature: entity_creature::CreatureFields,
    pub m_mob: MobFields,
}

impl EntityExt for MobExt
{
    fn asAny(&self) -> &dyn std::any::Any { self }

    fn living(&self) -> Option<&entity_living::LivingFields> { Some(&self.m_living) }

    fn creature(&self) -> Option<&entity_creature::CreatureFields> { Some(&self.m_creature) }

    fn mob(&self) -> Option<&MobFields> { Some(&self.m_mob) }
}

pub fn new(world: Option<Rc<World>>) -> Rc<Entity>
{
    let entity: Rc<Entity> = Entity::alloc(&CLASS, Box::new(MobExt::default()));
    init(&entity, world);
    entity
}
