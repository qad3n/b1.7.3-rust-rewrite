use crate::entity::{
    Entity, EntityClass, EntityExt, WatchedValue, entity_animal, entity_arrow, entity_creature, entity_living,
    entity_player,
};
use crate::inventory::{IInventory, InventoryPlayer};
use crate::item::{Item, ItemStack, item_food};
use crate::math::{AxisAlignedBB, math_helper};
use crate::pathfinding::PathEntity;
use crate::world::World;
use jvm::string::JString;
use jvm::throwable::OrNpe;
use std::cell::Cell;
use std::rc::Rc;

const PI_F: f32 = std::f64::consts::PI as f32;

mod flags
{
    pub const SITTING: i32 = 1;
    pub const ANGRY: i32 = 2;
    pub const TAMED: i32 = 4;
}

mod event
{
    pub const TAMED: i8 = 7;
    pub const TAME_FAILED: i8 = 6;
    pub const SHAKE: i8 = 8;
}

#[derive(Default)]
pub struct WolfExt
{
    pub m_living: entity_living::LivingFields,
    pub m_creature: entity_creature::CreatureFields,
    m_looksWithInterest: Cell<bool>,
    m_headRollAngle: Cell<f32>,
    m_lastHeadRollAngle: Cell<f32>,
    m_isWolfShaking: Cell<bool>,
    m_shaking: Cell<bool>,
    m_timeWolfIsShaking: Cell<f32>,
    m_prevTimeWolfIsShaking: Cell<f32>,
}

impl EntityExt for WolfExt
{
    fn asAny(&self) -> &dyn std::any::Any { self }

    fn living(&self) -> Option<&entity_living::LivingFields> { Some(&self.m_living) }

    fn creature(&self) -> Option<&entity_creature::CreatureFields> { Some(&self.m_creature) }
}

fn fields(this: &Entity) -> &WolfExt { this.ext::<WolfExt>() }

fn getFlag(this: &Entity, flag: i32) -> bool { (this.m_dataWatcher.getWatchableObjectByte(16) as i32 & flag) != 0 }

fn setFlag(this: &Entity, flag: i32, value: bool)
{
    let by: i32 = this.m_dataWatcher.getWatchableObjectByte(16) as i32;
    let updated: i32 = if value { by | flag } else { by & !flag };
    this.m_dataWatcher.updateObject(16, WatchedValue::Byte(updated as i8));
}

pub fn isWolfSitting(this: &Rc<Entity>) -> bool { getFlag(this, flags::SITTING) }

pub fn setWolfSitting(this: &Rc<Entity>, sitting: bool) { setFlag(this, flags::SITTING, sitting); }

pub fn isWolfAngry(this: &Rc<Entity>) -> bool { getFlag(this, flags::ANGRY) }

pub fn setWolfAngry(this: &Rc<Entity>, angry: bool) { setFlag(this, flags::ANGRY, angry); }

pub fn isWolfTamed(this: &Rc<Entity>) -> bool { getFlag(this, flags::TAMED) }

pub fn setWolfTamed(this: &Rc<Entity>, tame: bool) { setFlag(this, flags::TAMED, tame); }

pub fn getWolfOwner(this: &Rc<Entity>) -> Option<JString> { Some(this.m_dataWatcher.getWatchableObjectString(17)) }

pub fn setWolfOwner(this: &Rc<Entity>, name: JString)
{
    this.m_dataWatcher.updateObject(17, WatchedValue::String(name));
}

pub fn getWolfShaking(this: &Rc<Entity>) -> bool { fields(this).m_isWolfShaking.get() }

fn shakeTime(this: &Entity, tickDelta: f32) -> f32
{
    let wolf: &WolfExt = fields(this);
    wolf.m_prevTimeWolfIsShaking.get()
        + (wolf.m_timeWolfIsShaking.get() - wolf.m_prevTimeWolfIsShaking.get()) * tickDelta
}

pub fn getShadingWhileShaking(this: &Rc<Entity>, tickDelta: f32) -> f32
{
    0.75 + shakeTime(this, tickDelta) / 2.0 * 0.25
}

pub fn getShakeAngle(this: &Rc<Entity>, tickDelta: f32, offset: f32) -> f32
{
    let mut f: f32 = (shakeTime(this, tickDelta) + offset) / 1.8;
    if f < 0.0
    {
        f = 0.0;
    }
    else if f > 1.0
    {
        f = 1.0;
    }
    math_helper::sin(f * PI_F) * math_helper::sin(f * PI_F * 11.0) * 0.15 * PI_F
}

pub fn getInterestedAngle(this: &Rc<Entity>, tickDelta: f32) -> f32
{
    let wolf: &WolfExt = fields(this);
    (wolf.m_lastHeadRollAngle.get() + (wolf.m_headRollAngle.get() - wolf.m_lastHeadRollAngle.get()) * tickDelta)
        * 0.15
        * PI_F
}

pub fn setTailRotation(this: &Rc<Entity>) -> f32
{
    if isWolfAngry(this)
    {
        return 1.5393804;
    }
    if isWolfTamed(this)
    {
        return (0.55 - (20 - this.m_dataWatcher.getWatchableObjectInt(18)) as f32 * 0.02) * PI_F;
    }
    0.62831855
}

fn updatePlayerActionState(this: &Rc<Entity>)
{
    (entity_animal::CLASS.updatePlayerActionState)(this);
    let creature: &entity_creature::CreatureFields = entity_creature::fields(this);
    let world: Rc<World> = this.world();
    if !creature.m_hasAttacked.get()
        && !entity_creature::hasPathTo(this)
        && isWolfTamed(this)
        && this.m_ridingEntity.with(Option::is_none)
    {
        let entityplayer: Option<Rc<Entity>> = world.getPlayerEntityByName(&getWolfOwner(this).orNpe());
        if let Some(player) = entityplayer
        {
            let f: f32 = player.getDistanceToEntity(this);
            if f > 5.0
            {
                getPathOrWalkableBlock(this, &player, f);
            }
        }
        else if !this.isInWater()
        {
            setWolfSitting(this, true);
        }
    }
    else if creature.m_playerToAttack.with(Option::is_none)
        && !entity_creature::hasPathTo(this)
        && !isWolfTamed(this)
        && world.m_rand.nextIntBounded(100) == 0
    {
        let bounds: Rc<AxisAlignedBB> = AxisAlignedBB::getBoundingBoxFromPool(
            this.posX(),
            this.posY(),
            this.posZ(),
            this.posX() + 1.0,
            this.posY() + 1.0,
            this.posZ() + 1.0,
        )
        .expand(16.0, 4.0, 16.0);
        let list: Vec<Rc<Entity>> = world.getEntitiesWithinAABB("EntitySheep", &bounds);
        if !list.is_empty()
        {
            entity_creature::setTarget(
                this,
                Some(Rc::clone(&list[world.m_rand.nextIntBounded(list.len() as i32) as usize])),
            );
        }
    }
    if this.isInWater()
    {
        setWolfSitting(this, false);
    }
    if !world.m_multiplayerWorld.get()
    {
        this.m_dataWatcher.updateObject(18, WatchedValue::Int(entity_living::fields(this).m_health.get()));
    }
}

fn onLivingUpdate(this: &Rc<Entity>)
{
    (entity_animal::CLASS.onLivingUpdate)(this);
    let wolf: &WolfExt = fields(this);
    wolf.m_looksWithInterest.set(false);
    if entity_living::hasCurrentTarget(this)
        && !entity_creature::hasPathTo(this)
        && !isWolfAngry(this)
        && let Some(entity) = entity_living::getCurrentTarget(this)
        && entity.instanceOf("EntityPlayer")
        && let Some(itemstack) = entity_player::inventory(&entity).getCurrentItem()
    {
        let item: Option<Rc<Item>> = Item::byId(itemstack.m_itemID.get());
        if !isWolfTamed(this) && itemstack.m_itemID.get() == Item::bone().m_shiftedIndex
        {
            wolf.m_looksWithInterest.set(true);
        }
        else if isWolfTamed(this)
            && let Some(item) = item.filter(|item| item.instanceOf("ItemFood"))
        {
            wolf.m_looksWithInterest.set(item_food::getIsWolfsFavoriteMeat(&item));
        }
    }
    if !entity_living::fields(this).m_isMultiplayerEntity.get()
        && wolf.m_isWolfShaking.get()
        && !wolf.m_shaking.get()
        && !entity_creature::hasPathTo(this)
        && this.m_onGround.get()
    {
        wolf.m_shaking.set(true);
        wolf.m_timeWolfIsShaking.set(0.0);
        wolf.m_prevTimeWolfIsShaking.set(0.0);
        this.world().doEntityEvent(this, event::SHAKE);
    }
}

fn onUpdate(this: &Rc<Entity>)
{
    (entity_animal::CLASS.onUpdate)(this);
    let wolf: &WolfExt = fields(this);
    wolf.m_lastHeadRollAngle.set(wolf.m_headRollAngle.get());
    let goal: f32 = if wolf.m_looksWithInterest.get()
    {
        1.0
    }
    else
    {
        0.0
    };
    wolf.m_headRollAngle.set(wolf.m_headRollAngle.get() + (goal - wolf.m_headRollAngle.get()) * 0.4);
    if wolf.m_looksWithInterest.get()
    {
        entity_living::fields(this).m_numTicksToChaseTarget.set(10);
    }
    if this.isWet()
    {
        wolf.m_isWolfShaking.set(true);
        wolf.m_shaking.set(false);
        wolf.m_timeWolfIsShaking.set(0.0);
        wolf.m_prevTimeWolfIsShaking.set(0.0);
    }
    else if (wolf.m_isWolfShaking.get() || wolf.m_shaking.get()) && wolf.m_shaking.get()
    {
        let world: Rc<World> = this.world();
        if wolf.m_timeWolfIsShaking.get() == 0.0
        {
            let pitch: f32 = (this.m_rand.nextFloat() - this.m_rand.nextFloat()) * 0.2 + 1.0;
            world.playSoundAtEntity(this, "mob.wolf.shake", this.getSoundVolume(), pitch);
        }
        wolf.m_prevTimeWolfIsShaking.set(wolf.m_timeWolfIsShaking.get());
        wolf.m_timeWolfIsShaking.set(wolf.m_timeWolfIsShaking.get() + 0.05);
        if wolf.m_prevTimeWolfIsShaking.get() >= 2.0
        {
            wolf.m_isWolfShaking.set(false);
            wolf.m_shaking.set(false);
            wolf.m_prevTimeWolfIsShaking.set(0.0);
            wolf.m_timeWolfIsShaking.set(0.0);
        }
        if wolf.m_timeWolfIsShaking.get() > 0.4
        {
            let f: f32 = this.m_boundingBox.minY() as f32;
            let i: i32 = (math_helper::sin((wolf.m_timeWolfIsShaking.get() - 0.4) * PI_F) * 7.0) as i32;
            for _ in 0..i
            {
                let f1: f32 = (this.m_rand.nextFloat() * 2.0 - 1.0) * this.m_width.get() * 0.5;
                let f2: f32 = (this.m_rand.nextFloat() * 2.0 - 1.0) * this.m_width.get() * 0.5;
                world.spawnParticle(
                    "splash",
                    this.posX() + f1 as f64,
                    (f + 0.8) as f64,
                    this.posZ() + f2 as f64,
                    this.m_motionX.get(),
                    this.m_motionY.get(),
                    this.m_motionZ.get(),
                );
            }
        }
    }
}

fn getPathOrWalkableBlock(this: &Rc<Entity>, owner: &Rc<Entity>, distance: f32)
{
    let world: Rc<World> = this.world();
    let pathEntity: Option<Rc<PathEntity>> = world.getPathToEntity(this, owner, 16.0);
    if pathEntity.is_none() && distance > 12.0
    {
        let n: i32 = math_helper::floor_double(owner.posX()) - 2;
        let n2: i32 = math_helper::floor_double(owner.posZ()) - 2;
        let n3: i32 = math_helper::floor_double(owner.m_boundingBox.minY());
        for i in 0..=4
        {
            for j in 0..=4
            {
                if i >= 1 && j >= 1 && i <= 3 && j <= 3
                    || !world.isBlockNormalCube(n + i, n3 - 1, n2 + j)
                    || world.isBlockNormalCube(n + i, n3, n2 + j)
                    || world.isBlockNormalCube(n + i, n3 + 1, n2 + j)
                {
                    continue;
                }
                this.setLocationAndAngles(
                    ((n + i) as f32 + 0.5) as f64,
                    n3 as f64,
                    ((n2 + j) as f32 + 0.5) as f64,
                    this.m_rotationYaw.get(),
                    this.m_rotationPitch.get(),
                );
                return;
            }
        }
    }
    else
    {
        entity_creature::setPathToEntity(this, pathEntity);
    }
}

fn attackEntityFrom(this: &Rc<Entity>, source: Option<&Rc<Entity>>, mut amount: i32) -> bool
{
    setWolfSitting(this, false);
    if let Some(source) = source
        && !source.instanceOf("EntityPlayer")
        && !source.instanceOf("EntityArrow")
    {
        amount = (amount + 1) / 2;
    }
    if !(entity_animal::CLASS.attackEntityFrom)(this, source, amount)
    {
        return false;
    }
    let creature: &entity_creature::CreatureFields = entity_creature::fields(this);
    if !isWolfTamed(this) && !isWolfAngry(this)
    {
        let mut source: Option<Rc<Entity>> = source.cloned();
        if let Some(player) = source.as_ref().filter(|source| source.instanceOf("EntityPlayer"))
        {
            setWolfAngry(this, true);
            creature.m_playerToAttack.set(Some(Rc::clone(player)));
        }
        if let Some(arrow) = source.as_ref().filter(|source| source.instanceOf("EntityArrow"))
            && let Some(owner) = entity_arrow::fields(arrow).m_owner.get()
        {
            source = Some(owner);
        }
        if let Some(attacker) = source.filter(|source| source.instanceOf("EntityLiving"))
        {
            let bounds: Rc<AxisAlignedBB> = AxisAlignedBB::getBoundingBoxFromPool(
                this.posX(),
                this.posY(),
                this.posZ(),
                this.posX() + 1.0,
                this.posY() + 1.0,
                this.posZ() + 1.0,
            )
            .expand(16.0, 4.0, 16.0);
            for entityWolf in this.world().getEntitiesWithinAABB("EntityWolf", &bounds)
            {
                let wolfCreature: &entity_creature::CreatureFields = entity_creature::fields(&entityWolf);
                if isWolfTamed(&entityWolf) || wolfCreature.m_playerToAttack.with(Option::is_some)
                {
                    continue;
                }
                wolfCreature.m_playerToAttack.set(Some(Rc::clone(&attacker)));
                if !attacker.instanceOf("EntityPlayer")
                {
                    continue;
                }
                setWolfAngry(&entityWolf, true);
            }
        }
    }
    else if let Some(source) = source
        && !Rc::ptr_eq(source, this)
    {
        if isWolfTamed(this)
            && source.instanceOf("EntityPlayer")
            && entity_player::fields(source).m_username.get().orNpe().equalsIgnoreCase(getWolfOwner(this).as_ref())
        {
            return true;
        }
        creature.m_playerToAttack.set(Some(Rc::clone(source)));
    }
    true
}

fn attackEntity(this: &Rc<Entity>, target: &Rc<Entity>, distance: f32)
{
    if distance > 2.0 && distance < 6.0 && this.m_rand.nextIntBounded(10) == 0
    {
        if this.m_onGround.get()
        {
            let d: f64 = target.posX() - this.posX();
            let d2: f64 = target.posZ() - this.posZ();
            let f: f32 = math_helper::sqrt_double(d * d + d2 * d2);
            this.m_motionX.set(d / f as f64 * 0.5 * 0.8f32 as f64 + this.m_motionX.get() * 0.2f32 as f64);
            this.m_motionZ.set(d2 / f as f64 * 0.5 * 0.8f32 as f64 + this.m_motionZ.get() * 0.2f32 as f64);
            this.m_motionY.set(0.4f32 as f64);
        }
    }
    else if (distance as f64) < 1.5
        && target.m_boundingBox.maxY() > this.m_boundingBox.minY()
        && target.m_boundingBox.minY() < this.m_boundingBox.maxY()
    {
        entity_living::fields(this).m_attackTime.set(20);
        let n: i32 = if isWolfTamed(this) { 4 } else { 2 };
        target.attackEntityFrom(Some(this), n);
    }
}

fn consumeHeldItem(inventory: &InventoryPlayer, itemstack: &ItemStack)
{
    itemstack.m_stackSize.set(itemstack.m_stackSize.get() - 1);
    if itemstack.m_stackSize.get() <= 0
    {
        inventory.setInventorySlotContents(inventory.m_currentItem.get(), None);
    }
}

fn interact(this: &Rc<Entity>, player: &Rc<Entity>) -> bool
{
    let inventory: Rc<InventoryPlayer> = entity_player::inventory(player);
    let itemstack: Option<Rc<ItemStack>> = inventory.getCurrentItem();
    let world: Rc<World> = this.world();
    if !isWolfTamed(this)
    {
        if let Some(itemstack) = itemstack
            && itemstack.m_itemID.get() == Item::bone().m_shiftedIndex
            && !isWolfAngry(this)
        {
            consumeHeldItem(&inventory, &itemstack);
            if !world.m_multiplayerWorld.get()
            {
                if this.m_rand.nextIntBounded(3) == 0
                {
                    setWolfTamed(this, true);
                    entity_creature::setPathToEntity(this, None);
                    setWolfSitting(this, true);
                    entity_living::fields(this).m_health.set(20);
                    setWolfOwner(this, entity_player::fields(player).m_username.get().orNpe());
                    showHeartsOrSmokeFX(this, true);
                    world.doEntityEvent(this, event::TAMED);
                }
                else
                {
                    showHeartsOrSmokeFX(this, false);
                    world.doEntityEvent(this, event::TAME_FAILED);
                }
            }
            return true;
        }
    }
    else
    {
        if let Some(itemstack) = itemstack
            && let Some(itemfood) = Item::byId(itemstack.m_itemID.get()).filter(|item| item.instanceOf("ItemFood"))
            && item_food::getIsWolfsFavoriteMeat(&itemfood)
            && this.m_dataWatcher.getWatchableObjectInt(18) < 20
        {
            consumeHeldItem(&inventory, &itemstack);
            this.heal(item_food::getHealAmount(&Item::porkRaw()));
            return true;
        }
        if entity_player::fields(player).m_username.get().orNpe().equalsIgnoreCase(getWolfOwner(this).as_ref())
        {
            if !world.m_multiplayerWorld.get()
            {
                setWolfSitting(this, !isWolfSitting(this));
                entity_living::fields(this).m_isJumping.set(false);
                entity_creature::setPathToEntity(this, None);
            }
            return true;
        }
    }
    false
}

fn showHeartsOrSmokeFX(this: &Rc<Entity>, breeding: bool)
{
    let particle: &str = if breeding { "heart" } else { "smoke" };
    let world: Rc<World> = this.world();
    for _ in 0..7
    {
        let d: f64 = this.m_rand.nextGaussian() * 0.02;
        let d2: f64 = this.m_rand.nextGaussian() * 0.02;
        let d3: f64 = this.m_rand.nextGaussian() * 0.02;
        let width: f32 = this.m_width.get();
        let x: f64 = this.posX() + (this.m_rand.nextFloat() * width * 2.0) as f64 - width as f64;
        let y: f64 = this.posY() + 0.5 + (this.m_rand.nextFloat() * this.m_height.get()) as f64;
        let z: f64 = this.posZ() + (this.m_rand.nextFloat() * width * 2.0) as f64 - width as f64;
        world.spawnParticle(particle, x, y, z, d, d2, d3);
    }
}

fn handleHealthUpdate(this: &Rc<Entity>, kind: i8)
{
    match kind
    {
        event::TAMED => showHeartsOrSmokeFX(this, true),
        event::TAME_FAILED => showHeartsOrSmokeFX(this, false),
        event::SHAKE =>
        {
            let wolf: &WolfExt = fields(this);
            wolf.m_shaking.set(true);
            wolf.m_timeWolfIsShaking.set(0.0);
            wolf.m_prevTimeWolfIsShaking.set(0.0);
        }
        _ => (entity_animal::CLASS.handleHealthUpdate)(this, kind),
    }
}

pub const CLASS: EntityClass = EntityClass {
    m_name: "EntityWolf",
    m_parent: Some(&entity_animal::CLASS),
    entityInit: |this| {
        (entity_animal::CLASS.entityInit)(this);
        this.m_dataWatcher.addObject(16, WatchedValue::Byte(0));
        this.m_dataWatcher.addObject(17, WatchedValue::String(JString::from("")));
        this.m_dataWatcher.addObject(18, WatchedValue::Int(entity_living::fields(this).m_health.get()));
    },
    canTriggerWalking: |_| false,
    getEntityTexture: |this| {
        if isWolfTamed(this)
        {
            return Some(JString::from("/mob/wolf_tame.png"));
        }
        if isWolfAngry(this)
        {
            return Some(JString::from("/mob/wolf_angry.png"));
        }
        (entity_animal::CLASS.getEntityTexture)(this)
    },
    writeEntityToNBT: |this, nbt| {
        (entity_animal::CLASS.writeEntityToNBT)(this, nbt);
        nbt.setBoolean("Angry", isWolfAngry(this));
        nbt.setBoolean("Sitting", isWolfSitting(this));
        nbt.setString("Owner", Some(getWolfOwner(this).unwrap_or_default()));
    },
    readEntityFromNBT: |this, nbt| {
        (entity_animal::CLASS.readEntityFromNBT)(this, nbt);
        setWolfAngry(this, nbt.getBoolean("Angry"));
        setWolfSitting(this, nbt.getBoolean("Sitting"));
        let owner: JString = nbt.getString("Owner").orNpe();
        if owner.length() > 0
        {
            setWolfOwner(this, owner);
            setWolfTamed(this, true);
        }
    },
    canDespawn: |this| !isWolfTamed(this),
    getLivingSound: |this| {
        if isWolfAngry(this)
        {
            return Some("mob.wolf.growl".to_string());
        }
        if this.m_rand.nextIntBounded(3) == 0
        {
            if isWolfTamed(this) && this.m_dataWatcher.getWatchableObjectInt(18) < 10
            {
                return Some("mob.wolf.whine".to_string());
            }
            return Some("mob.wolf.panting".to_string());
        }
        Some("mob.wolf.bark".to_string())
    },
    getHurtSound: |_| Some("mob.wolf.hurt".to_string()),
    getDeathSound: |_| Some("mob.wolf.death".to_string()),
    getSoundVolume: |_| 0.4,
    getDropItemId: |_| -1,
    updatePlayerActionState,
    onLivingUpdate,
    onUpdate,
    getEyeHeight: |this| this.m_height.get() * 0.8,
    verticleLookSpeed: |this| {
        if isWolfSitting(this)
        {
            return 20;
        }
        (entity_animal::CLASS.verticleLookSpeed)(this)
    },
    isMovementCeased: |this| isWolfSitting(this) || fields(this).m_shaking.get(),
    attackEntityFrom,
    findPlayerToAttack: |this| {
        if isWolfAngry(this)
        {
            return this.world().getClosestPlayerToEntity(this, 16.0);
        }
        None
    },
    attackEntity,
    interact,
    handleHealthUpdate,
    getMaxSpawnedInChunk: |_| 8,
    ..entity_animal::CLASS
};

pub fn new(world: Option<Rc<World>>) -> Rc<Entity>
{
    let entity: Rc<Entity> = Entity::alloc(&CLASS, Box::new(WolfExt::default()));
    entity_animal::init(&entity, world);
    fields(&entity).m_looksWithInterest.set(false);
    let living: &entity_living::LivingFields = entity_living::fields(&entity);
    living.m_texture.set(Some(JString::from("/mob/wolf.png")));
    entity.setSize(0.8, 0.8);
    living.m_moveSpeed.set(1.1);
    living.m_health.set(8);
    entity
}
