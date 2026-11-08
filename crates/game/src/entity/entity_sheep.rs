use crate::block::Block;
use crate::entity::entity_cow::AnimalExt;
use crate::entity::{Entity, EntityClass, WatchedValue, entity_animal, entity_living, entity_player};
use crate::item::{Item, ItemStack};
use crate::world::World;
use jvm::random::Random;
use jvm::string::JString;
use std::rc::Rc;

pub const FLEECE_COLOR_TABLE: [[f32; 3]; 16] = [
    [1.0, 1.0, 1.0],
    [0.95, 0.7, 0.2],
    [0.9, 0.5, 0.85],
    [0.6, 0.7, 0.95],
    [0.9, 0.9, 0.2],
    [0.5, 0.8, 0.1],
    [0.95, 0.7, 0.8],
    [0.3, 0.3, 0.3],
    [0.6, 0.6, 0.6],
    [0.3, 0.6, 0.7],
    [0.7, 0.4, 0.9],
    [0.2, 0.4, 0.8],
    [0.5, 0.4, 0.3],
    [0.4, 0.5, 0.2],
    [0.8, 0.3, 0.3],
    [0.1, 0.1, 0.1],
];

mod wool
{
    pub const COLOR_MASK: i32 = 15;
    pub const SHEARED_BIT: i32 = 16;
}

pub fn getFleeceColor(this: &Rc<Entity>) -> i32
{
    this.m_dataWatcher.getWatchableObjectByte(16) as i32 & wool::COLOR_MASK
}

pub fn setFleeceColor(this: &Rc<Entity>, color: i32)
{
    let b0: i8 = this.m_dataWatcher.getWatchableObjectByte(16);
    this.m_dataWatcher.updateObject(16, WatchedValue::Byte((b0 as i32 & 240 | color & wool::COLOR_MASK) as i8));
}

pub fn getSheared(this: &Rc<Entity>) -> bool
{
    (this.m_dataWatcher.getWatchableObjectByte(16) as i32 & wool::SHEARED_BIT) != 0
}

pub fn setSheared(this: &Rc<Entity>, sheared: bool)
{
    let b0: i32 = this.m_dataWatcher.getWatchableObjectByte(16) as i32;
    let value: i32 = if sheared
    {
        b0 | wool::SHEARED_BIT
    }
    else
    {
        b0 & -17
    };
    this.m_dataWatcher.updateObject(16, WatchedValue::Byte(value as i8));
}

pub fn getRandomFleeceColor(random: &Random) -> i32
{
    let nextInt: i32 = random.nextIntBounded(100);
    if nextInt < 5
    {
        return 15;
    }
    if nextInt < 10
    {
        return 7;
    }
    if nextInt < 15
    {
        return 8;
    }
    if nextInt < 18
    {
        return 12;
    }
    if random.nextIntBounded(500) == 0
    {
        return 6;
    }
    0
}

fn woolStack(this: &Rc<Entity>) -> Rc<ItemStack>
{
    ItemStack::withBlockDamage(&Block::cloth(), 1, getFleeceColor(this))
}

fn interact(this: &Rc<Entity>, player: &Rc<Entity>) -> bool
{
    let itemstack: Option<Rc<ItemStack>> = entity_player::inventory(player).getCurrentItem();
    if let Some(itemstack) = itemstack
        && itemstack.m_itemID.get() == Item::shears().m_shiftedIndex
        && !getSheared(this)
    {
        if !this.world().m_multiplayerWorld.get()
        {
            setSheared(this, true);
            let i: i32 = 2 + this.m_rand.nextIntBounded(3);
            for _ in 0..i
            {
                let entityitem: Rc<Entity> = this.entityDropItem(woolStack(this), 1.0);
                entityitem.m_motionY.set(entityitem.m_motionY.get() + (this.m_rand.nextFloat() * 0.05) as f64);
                entityitem.m_motionX.set(
                    entityitem.m_motionX.get() + ((this.m_rand.nextFloat() - this.m_rand.nextFloat()) * 0.1) as f64,
                );
                entityitem.m_motionZ.set(
                    entityitem.m_motionZ.get() + ((this.m_rand.nextFloat() - this.m_rand.nextFloat()) * 0.1) as f64,
                );
            }
        }
        itemstack.damageItem(1, player);
    }
    false
}

pub const CLASS: EntityClass = EntityClass {
    m_name: "EntitySheep",
    m_parent: Some(&entity_animal::CLASS),
    entityInit: |this| {
        (entity_animal::CLASS.entityInit)(this);
        this.m_dataWatcher.addObject(16, WatchedValue::Byte(0));
    },
    attackEntityFrom: |this, source, amount| (entity_animal::CLASS.attackEntityFrom)(this, source, amount),
    dropFewItems: |this| {
        if !getSheared(this)
        {
            this.entityDropItem(woolStack(this), 0.0);
        }
    },
    getDropItemId: |_| Block::cloth().m_blockID,
    interact,
    writeEntityToNBT: |this, nbt| {
        (entity_animal::CLASS.writeEntityToNBT)(this, nbt);
        nbt.setBoolean("Sheared", getSheared(this));
        nbt.setByte("Color", getFleeceColor(this) as i8);
    },
    readEntityFromNBT: |this, nbt| {
        (entity_animal::CLASS.readEntityFromNBT)(this, nbt);
        setSheared(this, nbt.getBoolean("Sheared"));
        setFleeceColor(this, nbt.getByte("Color") as i32);
    },
    getLivingSound: |_| Some("mob.sheep".to_string()),
    getHurtSound: |_| Some("mob.sheep".to_string()),
    getDeathSound: |_| Some("mob.sheep".to_string()),
    ..entity_animal::CLASS
};

pub fn new(world: Option<Rc<World>>) -> Rc<Entity>
{
    let entity: Rc<Entity> = Entity::alloc(&CLASS, Box::new(AnimalExt::default()));
    entity_animal::init(&entity, world);
    entity_living::fields(&entity).m_texture.set(Some(JString::from("/mob/sheep.png")));
    entity.setSize(0.9, 1.3);
    entity
}
