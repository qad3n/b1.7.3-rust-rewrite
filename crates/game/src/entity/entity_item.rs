use crate::block::{Block, Material};
use crate::entity::{ENTITY, Entity, EntityClass, EntityExt, entity_player};
use crate::item::{Item, ItemStack};
use crate::math::math_helper::floor_double;
use crate::stats::AchievementList;
use crate::world::World;
use common::nbt::{NBTBase, NBTTagCompound};
use jvm::field::Field;
use jvm::throwable::OrNpe;
use std::cell::Cell;
use std::f64::consts::PI;
use std::rc::Rc;

pub struct ItemFields
{
    pub m_item: Field<Option<Rc<ItemStack>>>,
    m_renderTicks: Cell<i32>,
    pub m_age: Cell<i32>,
    pub m_delayBeforeCanPickup: Cell<i32>,
    m_health: Cell<i32>,
    pub m_bobOffset: Cell<f32>,
}

impl EntityExt for ItemFields
{
    fn asAny(&self) -> &dyn std::any::Any { self }
}

pub fn fields(this: &Entity) -> &ItemFields { this.ext::<ItemFields>() }

fn item(this: &Entity) -> Rc<ItemStack> { fields(this).m_item.get().orNpe() }

fn alloc(world: Option<Rc<World>>) -> Rc<Entity>
{
    let entity: Rc<Entity> = Entity::alloc(
        &CLASS,
        Box::new(ItemFields {
            m_item: Field::new(None),
            m_renderTicks: Cell::new(0),
            m_age: Cell::new(0),
            m_delayBeforeCanPickup: Cell::new(0),
            m_health: Cell::new(0),
            m_bobOffset: Cell::new(0.0),
        }),
    );
    Entity::init(&entity, world);
    let item: &ItemFields = fields(&entity);
    item.m_health.set(5);
    item.m_bobOffset.set((jvm::math::random() * PI * 2.0) as f32);
    entity.setSize(0.25, 0.25);
    entity.m_yOffset.set(entity.m_height.get() / 2.0);
    entity
}

pub fn create(world: &Rc<World>, x: f64, y: f64, z: f64, item: Rc<ItemStack>) -> Rc<Entity>
{
    let entity: Rc<Entity> = alloc(Some(Rc::clone(world)));
    entity.setPosition(x, y, z);
    fields(&entity).m_item.set(Some(item));
    entity.m_rotationYaw.set((jvm::math::random() * 360.0) as f32);
    entity.m_motionX.set((jvm::math::random() * 0.2f32 as f64 - 0.1f32 as f64) as f32 as f64);
    entity.m_motionY.set(0.2f32 as f64);
    entity.m_motionZ.set((jvm::math::random() * 0.2f32 as f64 - 0.1f32 as f64) as f32 as f64);
    entity
}

pub fn new(world: Option<Rc<World>>) -> Rc<Entity> { alloc(world) }

fn onUpdate(this: &Rc<Entity>)
{
    (ENTITY.onUpdate)(this);
    let fields: &ItemFields = fields(this);
    if fields.m_delayBeforeCanPickup.get() > 0
    {
        fields.m_delayBeforeCanPickup.set(fields.m_delayBeforeCanPickup.get() - 1);
    }
    this.m_prevPosX.set(this.posX());
    this.m_prevPosY.set(this.posY());
    this.m_prevPosZ.set(this.posZ());
    this.m_motionY.set(this.m_motionY.get() - 0.04f32 as f64);
    let world: Rc<World> = this.world();
    if Rc::ptr_eq(
        &world.getBlockMaterial(floor_double(this.posX()), floor_double(this.posY()), floor_double(this.posZ())),
        &Material::lava(),
    )
    {
        this.m_motionY.set(0.2f32 as f64);
        this.m_motionX.set(((this.m_rand.nextFloat() - this.m_rand.nextFloat()) * 0.2) as f64);
        this.m_motionZ.set(((this.m_rand.nextFloat() - this.m_rand.nextFloat()) * 0.2) as f64);
        world.playSoundAtEntity(this, "random.fizz", 0.4, 2.0 + this.m_rand.nextFloat() * 0.4);
    }
    this.pushOutOfBlocks(this.posX(), (this.m_boundingBox.minY() + this.m_boundingBox.maxY()) / 2.0, this.posZ());
    this.moveEntity(this.m_motionX.get(), this.m_motionY.get(), this.m_motionZ.get());
    let mut f: f32 = 0.98;
    if this.m_onGround.get()
    {
        f = 0.58800006;
        let n: i32 = world.getBlockId(
            floor_double(this.posX()),
            floor_double(this.m_boundingBox.minY()) - 1,
            floor_double(this.posZ()),
        );
        if n > 0
        {
            f = Block::byId(n).orNpe().m_slipperiness.get() * 0.98;
        }
    }
    this.m_motionX.set(this.m_motionX.get() * f as f64);
    this.m_motionY.set(this.m_motionY.get() * 0.98f32 as f64);
    this.m_motionZ.set(this.m_motionZ.get() * f as f64);
    if this.m_onGround.get()
    {
        this.m_motionY.set(this.m_motionY.get() * -0.5);
    }
    fields.m_renderTicks.set(fields.m_renderTicks.get() + 1);
    fields.m_age.set(fields.m_age.get() + 1);
    if fields.m_age.get() >= 6000
    {
        this.setEntityDead();
    }
}

fn onCollideWithPlayer(this: &Rc<Entity>, player: &Rc<Entity>)
{
    if this.world().m_multiplayerWorld.get()
    {
        return;
    }
    let fields: &ItemFields = fields(this);
    let stack: Rc<ItemStack> = item(this);
    let n: i32 = stack.m_stackSize.get();
    if fields.m_delayBeforeCanPickup.get() == 0 && entity_player::inventory(player).addItemStackToInventory(&stack)
    {
        if stack.m_itemID.get() == Block::wood().m_blockID
        {
            entity_player::triggerAchievement(player, AchievementList::mineWood());
        }
        if stack.m_itemID.get() == Item::leather().m_shiftedIndex
        {
            entity_player::triggerAchievement(player, AchievementList::killCow());
        }
        this.world().playSoundAtEntity(
            this,
            "random.pop",
            0.2,
            ((this.m_rand.nextFloat() - this.m_rand.nextFloat()) * 0.7 + 1.0) * 2.0,
        );
        player.onItemPickup(this, n);
        if stack.m_stackSize.get() <= 0
        {
            this.setEntityDead();
        }
    }
}

pub const CLASS: EntityClass = EntityClass {
    m_name: "EntityItem",
    m_parent: Some(&ENTITY),
    m_interfaces: &[],
    canTriggerWalking: |_| false,
    entityInit: |_| {},
    onUpdate,
    handleWaterMovement: |this| this.world().handleMaterialAcceleration(&this.m_boundingBox, Material::water(), this),
    dealFireDamage: |this, amount| {
        this.attackEntityFrom(None, amount);
    },
    attackEntityFrom: |this, _source, amount| {
        this.setBeenAttacked();
        let fields: &ItemFields = fields(this);
        fields.m_health.set(fields.m_health.get() - amount);
        if fields.m_health.get() <= 0
        {
            this.setEntityDead();
        }
        false
    },
    writeEntityToNBT: |this, nbt| {
        let fields: &ItemFields = fields(this);
        nbt.setShort("Health", fields.m_health.get() as i8 as i16);
        nbt.setShort("Age", fields.m_age.get() as i16);
        let tag: Rc<NBTBase> = NBTTagCompound::create();
        item(this).writeToNBT(tag.compound());
        nbt.setCompoundTag("Item", &tag);
    },
    readEntityFromNBT: |this, nbt| {
        let fields: &ItemFields = fields(this);
        fields.m_health.set(nbt.getShort("Health") as i32 & 0xFF);
        fields.m_age.set(nbt.getShort("Age") as i32);
        let tag: Rc<NBTBase> = nbt.getCompoundTag("Item");
        fields.m_item.set(Some(ItemStack::withNbt(tag.compound())));
    },
    onCollideWithPlayer,
    ..ENTITY
};
