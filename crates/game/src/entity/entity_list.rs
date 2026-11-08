use crate::entity::{
    Entity, entity_arrow, entity_boat, entity_chicken, entity_cow, entity_creeper, entity_falling_sand, entity_ghast,
    entity_giant_zombie, entity_item, entity_minecart, entity_mob, entity_painting, entity_pig, entity_pig_zombie,
    entity_sheep, entity_skeleton, entity_slime, entity_snowball, entity_spider, entity_squid, entity_tnt_primed,
    entity_wolf, entity_zombie,
};
use crate::world::World;
use common::nbt::NBTTagCompound;
use jvm::string::JString;
use jvm::throwable::{self, OrNpe, Throwable};
use std::rc::Rc;

type Constructor = fn(Option<Rc<World>>) -> Rc<Entity>;

struct EntityRow
{
    m_className: &'static str,
    m_key: &'static str,
    m_id: i32,
    m_create: Option<Constructor>,
}

const MAPPINGS: [EntityRow; 24] = [
    EntityRow { m_className: "EntityArrow", m_key: "Arrow", m_id: 10, m_create: Some(entity_arrow::new) },
    EntityRow { m_className: "EntitySnowball", m_key: "Snowball", m_id: 11, m_create: Some(entity_snowball::new) },
    EntityRow { m_className: "EntityItem", m_key: "Item", m_id: 1, m_create: Some(entity_item::new) },
    EntityRow { m_className: "EntityPainting", m_key: "Painting", m_id: 9, m_create: Some(entity_painting::new) },
    EntityRow { m_className: "EntityLiving", m_key: "Mob", m_id: 48, m_create: None },
    EntityRow { m_className: "EntityMob", m_key: "Monster", m_id: 49, m_create: Some(entity_mob::new) },
    EntityRow { m_className: "EntityCreeper", m_key: "Creeper", m_id: 50, m_create: Some(entity_creeper::new) },
    EntityRow { m_className: "EntitySkeleton", m_key: "Skeleton", m_id: 51, m_create: Some(entity_skeleton::new) },
    EntityRow { m_className: "EntitySpider", m_key: "Spider", m_id: 52, m_create: Some(entity_spider::new) },
    EntityRow { m_className: "EntityGiantZombie", m_key: "Giant", m_id: 53, m_create: Some(entity_giant_zombie::new) },
    EntityRow { m_className: "EntityZombie", m_key: "Zombie", m_id: 54, m_create: Some(entity_zombie::new) },
    EntityRow { m_className: "EntitySlime", m_key: "Slime", m_id: 55, m_create: Some(entity_slime::new) },
    EntityRow { m_className: "EntityGhast", m_key: "Ghast", m_id: 56, m_create: Some(entity_ghast::new) },
    EntityRow { m_className: "EntityPigZombie", m_key: "PigZombie", m_id: 57, m_create: Some(entity_pig_zombie::new) },
    EntityRow { m_className: "EntityPig", m_key: "Pig", m_id: 90, m_create: Some(entity_pig::new) },
    EntityRow { m_className: "EntitySheep", m_key: "Sheep", m_id: 91, m_create: Some(entity_sheep::new) },
    EntityRow { m_className: "EntityCow", m_key: "Cow", m_id: 92, m_create: Some(entity_cow::new) },
    EntityRow { m_className: "EntityChicken", m_key: "Chicken", m_id: 93, m_create: Some(entity_chicken::new) },
    EntityRow { m_className: "EntitySquid", m_key: "Squid", m_id: 94, m_create: Some(entity_squid::new) },
    EntityRow { m_className: "EntityWolf", m_key: "Wolf", m_id: 95, m_create: Some(entity_wolf::new) },
    EntityRow { m_className: "EntityTNTPrimed", m_key: "PrimedTnt", m_id: 20, m_create: Some(entity_tnt_primed::new) },
    EntityRow {
        m_className: "EntityFallingSand",
        m_key: "FallingSand",
        m_id: 21,
        m_create: Some(entity_falling_sand::new),
    },
    EntityRow { m_className: "EntityMinecart", m_key: "Minecart", m_id: 40, m_create: Some(entity_minecart::new) },
    EntityRow { m_className: "EntityBoat", m_key: "Boat", m_id: 41, m_create: Some(entity_boat::new) },
];

pub struct EntityList;

fn instantiate(row: Option<&EntityRow>, world: &Rc<World>) -> Option<Rc<Entity>>
{
    let row: &EntityRow = row?;
    let result: Result<Rc<Entity>, Throwable> =
        throwable::catchThrowable(&throwable::EXCEPTION, || match row.m_create
        {
            Some(create) => create(Some(Rc::clone(world))),
            None => throwable::throwBare(&throwable::INSTANTIATION),
        });
    match result
    {
        Ok(entity) => Some(entity),
        Err(error) =>
        {
            error.printStackTrace();
            None
        }
    }
}

fn byKey(key: Option<&JString>) -> Option<&'static EntityRow>
{
    let key: &JString = key?;
    MAPPINGS.iter().find(|row| JString::from(row.m_key) == *key)
}

fn byClass(entity: &Entity) -> Option<&'static EntityRow>
{
    MAPPINGS.iter().find(|row| row.m_className == entity.className())
}

impl EntityList
{
    pub fn createEntityInWorld(key: Option<&JString>, world: &Rc<World>) -> Option<Rc<Entity>>
    {
        instantiate(byKey(key), world)
    }

    pub fn createEntityFromNBT(nbt: &NBTTagCompound, world: &Rc<World>) -> Option<Rc<Entity>>
    {
        let id: Option<JString> = nbt.getString("id");
        let entity: Option<Rc<Entity>> = instantiate(byKey(id.as_ref()), world);
        match &entity
        {
            Some(entity) => entity.readFromNBT(nbt),
            None => jvm::system::println(&format!(
                "Skipping Entity with id {}",
                JString::display(nbt.getString("id").as_ref())
            )),
        }
        entity
    }

    pub fn createEntity(id: i32, world: &Rc<World>) -> Option<Rc<Entity>>
    {
        let entity: Option<Rc<Entity>> = instantiate(MAPPINGS.iter().find(|row| row.m_id == id), world);
        if entity.is_none()
        {
            jvm::system::println(&format!("Skipping Entity with id {id}"));
        }
        entity
    }

    pub fn getEntityID(entity: &Entity) -> i32 { byClass(entity).map(|row| row.m_id).orNpe() }

    pub fn getEntityString(entity: &Entity) -> Option<JString> { byClass(entity).map(|row| JString::from(row.m_key)) }

    pub fn objectString(entity: &Entity) -> String
    {
        format!("net.minecraft.src.{}@{:x}", entity.className(), entity.hashCode())
    }
}
