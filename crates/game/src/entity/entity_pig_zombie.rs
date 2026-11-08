use crate::entity::entity_mob::MobFields;
use crate::entity::{Entity, EntityClass, EntityExt, entity_creature, entity_living, entity_mob, entity_zombie};
use crate::item::{Item, ItemStack};
use crate::world::World;
use jvm::string::JString;
use std::cell::Cell;
use std::rc::Rc;

thread_local! {
    static DEFAULT_HELD_ITEM: Rc<ItemStack> = ItemStack::withItemCount(&Item::swordGold(), 1);
}

#[derive(Default)]
pub struct PigZombieExt
{
    pub m_living: entity_living::LivingFields,
    pub m_creature: entity_creature::CreatureFields,
    pub m_mob: MobFields,
    m_angerLevel: Cell<i32>,
    m_randomSoundDelay: Cell<i32>,
}

impl EntityExt for PigZombieExt
{
    fn asAny(&self) -> &dyn std::any::Any { self }

    fn living(&self) -> Option<&entity_living::LivingFields> { Some(&self.m_living) }

    fn creature(&self) -> Option<&entity_creature::CreatureFields> { Some(&self.m_creature) }

    fn mob(&self) -> Option<&MobFields> { Some(&self.m_mob) }
}

fn fields(this: &Entity) -> &PigZombieExt { this.ext::<PigZombieExt>() }

fn becomeAngryAt(this: &Rc<Entity>, target: &Rc<Entity>)
{
    entity_creature::fields(this).m_playerToAttack.set(Some(Rc::clone(target)));
    fields(this).m_angerLevel.set(400 + this.m_rand.nextIntBounded(400));
    fields(this).m_randomSoundDelay.set(this.m_rand.nextIntBounded(40));
}

fn onUpdate(this: &Rc<Entity>)
{
    let chasing: bool = entity_creature::fields(this).m_playerToAttack.with(Option::is_some);
    entity_living::fields(this).m_moveSpeed.set(if chasing { 0.95 } else { 0.5 });
    let pigZombie: &PigZombieExt = fields(this);
    if pigZombie.m_randomSoundDelay.get() > 0
    {
        pigZombie.m_randomSoundDelay.set(pigZombie.m_randomSoundDelay.get() - 1);
        if pigZombie.m_randomSoundDelay.get() == 0
        {
            let pitch: f32 = ((this.m_rand.nextFloat() - this.m_rand.nextFloat()) * 0.2 + 1.0) * 1.8;
            this.world().playSoundAtEntity(this, "mob.zombiepig.zpigangry", this.getSoundVolume() * 2.0, pitch);
        }
    }
    (entity_zombie::CLASS.onUpdate)(this);
}

fn attackEntityFrom(this: &Rc<Entity>, source: Option<&Rc<Entity>>, amount: i32) -> bool
{
    if let Some(player) = source.filter(|source| source.instanceOf("EntityPlayer"))
    {
        let bounds: Rc<crate::math::AxisAlignedBB> = this.m_boundingBox.expand(32.0, 32.0, 32.0);
        let list: Vec<Rc<Entity>> = this.world().getEntitiesWithinAABBExcludingEntity(Some(this), &bounds).toVec();
        for entity in list
        {
            if entity.instanceOf("EntityPigZombie")
            {
                becomeAngryAt(&entity, player);
            }
        }
        becomeAngryAt(this, player);
    }
    (entity_zombie::CLASS.attackEntityFrom)(this, source, amount)
}

pub const CLASS: EntityClass = EntityClass {
    m_name: "EntityPigZombie",
    m_parent: Some(&entity_zombie::CLASS),
    onUpdate,
    getCanSpawnHere: |this| {
        let world: Rc<World> = this.world();
        world.m_difficultySetting.get() > 0
            && world.checkIfAABBIsClear(&this.m_boundingBox)
            && world.getCollidingBoundingBoxes(this, &this.m_boundingBox).size() == 0
            && !world.getIsAnyLiquid(&this.m_boundingBox)
    },
    writeEntityToNBT: |this, nbt| {
        (entity_zombie::CLASS.writeEntityToNBT)(this, nbt);
        nbt.setShort("Anger", fields(this).m_angerLevel.get() as i16);
    },
    readEntityFromNBT: |this, nbt| {
        (entity_zombie::CLASS.readEntityFromNBT)(this, nbt);
        fields(this).m_angerLevel.set(nbt.getShort("Anger") as i32);
    },
    findPlayerToAttack: |this| {
        if fields(this).m_angerLevel.get() == 0
        {
            return None;
        }
        (entity_zombie::CLASS.findPlayerToAttack)(this)
    },
    onLivingUpdate: |this| (entity_zombie::CLASS.onLivingUpdate)(this),
    attackEntityFrom,
    getLivingSound: |_| Some("mob.zombiepig.zpig".to_string()),
    getHurtSound: |_| Some("mob.zombiepig.zpighurt".to_string()),
    getDeathSound: |_| Some("mob.zombiepig.zpigdeath".to_string()),
    getDropItemId: |_| Item::porkCooked().m_shiftedIndex,
    getHeldItem: |_| Some(DEFAULT_HELD_ITEM.with(Rc::clone)),
    ..entity_zombie::CLASS
};

pub fn new(world: Option<Rc<World>>) -> Rc<Entity>
{
    let entity: Rc<Entity> = Entity::alloc(&CLASS, Box::new(PigZombieExt::default()));
    entity_zombie::init(&entity, world);
    fields(&entity).m_angerLevel.set(0);
    fields(&entity).m_randomSoundDelay.set(0);
    let living: &entity_living::LivingFields = entity_living::fields(&entity);
    living.m_texture.set(Some(JString::from("/mob/pigzombie.png")));
    living.m_moveSpeed.set(0.5);
    entity_mob::fields(&entity).m_attackStrength.set(5);
    entity.m_isImmuneToFire.set(true);
    entity
}
