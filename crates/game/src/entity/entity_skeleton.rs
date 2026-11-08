use crate::entity::entity_mob::MobExt;
use crate::entity::{Entity, EntityClass, entity_arrow, entity_creature, entity_living, entity_mob, entity_zombie};
use crate::item::{Item, ItemStack};
use crate::math::math_helper;
use crate::world::World;
use jvm::string::JString;
use std::rc::Rc;

thread_local! {
    static DEFAULT_HELD_ITEM: Rc<ItemStack> = ItemStack::withItemCount(&Item::bow(), 1);
}

fn attackEntity(this: &Rc<Entity>, target: &Rc<Entity>, distance: f32)
{
    if !(distance < 10.0)
    {
        return;
    }
    let living: &entity_living::LivingFields = entity_living::fields(this);
    let d0: f64 = target.posX() - this.posX();
    let d1: f64 = target.posZ() - this.posZ();
    if living.m_attackTime.get() == 0
    {
        let world: Rc<World> = this.world();
        let entityarrow: Rc<Entity> = entity_arrow::createFromShooter(&world, this);
        entityarrow.m_posY.set(entityarrow.posY() + 1.4f32 as f64);
        let d2: f64 = target.posY() + target.getEyeHeight() as f64 - 0.2f32 as f64 - entityarrow.posY();
        let f: f32 = math_helper::sqrt_double(d0 * d0 + d1 * d1) * 0.2;
        world.playSoundAtEntity(this, "random.bow", 1.0, 1.0 / (this.m_rand.nextFloat() * 0.4 + 0.8));
        world.entityJoinedWorld(&entityarrow);
        entity_arrow::setArrowHeading(&entityarrow, d0, d2 + f as f64, d1, 0.6, 12.0);
        living.m_attackTime.set(30);
    }
    this.m_rotationYaw.set((jvm::math::atan2(d1, d0) * 180.0 / std::f64::consts::PI as f32 as f64) as f32 - 90.0);
    entity_creature::fields(this).m_hasAttacked.set(true);
}

pub const CLASS: EntityClass = EntityClass {
    m_name: "EntitySkeleton",
    m_parent: Some(&entity_mob::CLASS),
    getLivingSound: |_| Some("mob.skeleton".to_string()),
    getHurtSound: |_| Some("mob.skeletonhurt".to_string()),
    getDeathSound: |_| Some("mob.skeletonhurt".to_string()),
    onLivingUpdate: |this| {
        entity_zombie::burnInDaylight(this);
        (entity_mob::CLASS.onLivingUpdate)(this);
    },
    attackEntity,
    writeEntityToNBT: |this, nbt| (entity_mob::CLASS.writeEntityToNBT)(this, nbt),
    readEntityFromNBT: |this, nbt| (entity_mob::CLASS.readEntityFromNBT)(this, nbt),
    getDropItemId: |_| Item::arrow().m_shiftedIndex,
    dropFewItems: |this| {
        let i: i32 = this.m_rand.nextIntBounded(3);
        for _ in 0..i
        {
            this.dropItem(Item::arrow().m_shiftedIndex, 1);
        }
        let i: i32 = this.m_rand.nextIntBounded(3);
        for _ in 0..i
        {
            this.dropItem(Item::bone().m_shiftedIndex, 1);
        }
    },
    getHeldItem: |_| Some(DEFAULT_HELD_ITEM.with(Rc::clone)),
    ..entity_mob::CLASS
};

pub fn new(world: Option<Rc<World>>) -> Rc<Entity>
{
    let entity: Rc<Entity> = Entity::alloc(&CLASS, Box::new(MobExt::default()));
    entity_mob::init(&entity, world);
    entity_living::fields(&entity).m_texture.set(Some(JString::from("/mob/skeleton.png")));
    entity
}
