mod combat;
mod sleep;
mod stats;

use crate::block::Block;
use crate::container::{Container, ContainerPlayer};
use crate::entity::{Entity, EntityClass, entity_item, entity_living};
use crate::inventory::{IInventory, InventoryPlayer};
use crate::item::{Item, ItemStack};
use crate::math::math_helper;
use crate::stats::{StatBase, StatList};
use crate::world::{ChunkCoordinates, World};
pub use combat::{alertWolves, attackTargetEntityWithCurrentItem, useCurrentItemOnEntity};
use jvm::field::Field;
use jvm::math;
use jvm::string::JString;
use jvm::throwable::OrNpe;
pub use sleep::{
    getBedOrientationInDegrees, getSleepingTime, isPlayerFullyAsleep, loadSpawnPoint, sleepInBedAt, wakeUpPlayer,
};
use std::cell::Cell;
use std::rc::Rc;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EnumStatus
{
    Ok,
    NotPossibleHere,
    NotPossibleNow,
    TooFarAway,
    OtherProblem,
}

#[derive(Default)]
pub struct PlayerFields
{
    pub m_inventory: Field<Option<Rc<InventoryPlayer>>>,
    pub m_inventorySlots: Field<Option<Rc<Container>>>,
    pub m_craftingInventory: Field<Option<Rc<Container>>>,
    pub m_userType: Cell<i8>,
    pub m_score: Cell<i32>,
    pub m_lastBob: Cell<f32>,
    pub m_bob: Cell<f32>,
    pub m_isSwinging: Cell<bool>,
    pub m_swingProgressInt: Cell<i32>,
    pub m_username: Field<Option<JString>>,
    pub m_dimension: Cell<i32>,
    pub m_playerCloakUrl: Field<Option<JString>>,
    pub m_lastCapeX: Cell<f64>,
    pub m_lastCapeY: Cell<f64>,
    pub m_lastCapeZ: Cell<f64>,
    pub m_capeX: Cell<f64>,
    pub m_capeY: Cell<f64>,
    pub m_capeZ: Cell<f64>,
    pub m_sleeping: Cell<bool>,
    pub m_bedChunkCoordinates: Cell<Option<ChunkCoordinates>>,
    pub m_sleepTimer: Cell<i32>,
    pub m_sleepingCameraOffsetX: Cell<f32>,
    pub m_sleepingCameraOffsetY: Cell<f32>,
    pub m_sleepingCameraOffsetZ: Cell<f32>,
    pub m_playerSpawnCoordinate: Cell<Option<ChunkCoordinates>>,
    pub m_startMinecartRidingCoordinate: Cell<Option<ChunkCoordinates>>,
    pub m_timeUntilPortal: Cell<i32>,
    pub m_inPortal: Cell<bool>,
    pub m_timeInPortal: Cell<f32>,
    pub m_prevTimeInPortal: Cell<f32>,
    pub m_damageRemainder: Cell<i32>,
    pub m_fishEntity: Field<Option<Rc<Entity>>>,
}

pub fn fields(this: &Entity) -> &PlayerFields
{
    match this.m_ext.player()
    {
        Some(fields) => fields,
        None => jvm::throwable::throw(
            &jvm::throwable::CLASS_CAST,
            &format!("net.minecraft.src.{} cannot be cast to net.minecraft.src.EntityPlayer", this.className()),
        ),
    }
}

pub fn inventory(this: &Entity) -> Rc<InventoryPlayer> { fields(this).m_inventory.get().orNpe() }

fn addStat(this: &Rc<Entity>, stat: Option<Rc<StatBase>>, amount: i32) { this.addStat(stat.as_ref(), amount); }

pub fn init(this: &Rc<Entity>, world: Rc<World>)
{
    entity_living::init(this, Some(Rc::clone(&world)));
    let player: &PlayerFields = fields(this);
    player.m_inventory.set(Some(Rc::new(InventoryPlayer::new(&Rc::downgrade(this)))));
    player.m_userType.set(0);
    player.m_score.set(0);
    player.m_isSwinging.set(false);
    player.m_swingProgressInt.set(0);
    player.m_timeUntilPortal.set(20);
    player.m_inPortal.set(false);
    player.m_damageRemainder.set(0);
    player.m_fishEntity.set(None);
    let slots: Rc<Container> = ContainerPlayer::new(&inventory(this), !world.m_multiplayerWorld.get());
    player.m_inventorySlots.set(Some(Rc::clone(&slots)));
    player.m_craftingInventory.set(Some(slots));
    this.m_yOffset.set(1.62);
    let chunkcoordinates: ChunkCoordinates = world.getSpawnPoint();
    this.setLocationAndAngles(
        chunkcoordinates.m_x as f64 + 0.5,
        (chunkcoordinates.m_y + 1) as f64,
        chunkcoordinates.m_z as f64 + 0.5,
        0.0,
        0.0,
    );
    let living: &entity_living::LivingFields = entity_living::fields(this);
    living.m_health.set(20);
    living.m_modelName.set(Some(JString::from("humanoid")));
    living.m_rotationOffset.set(180.0);
    this.m_fireResistance.set(20);
    living.m_texture.set(Some(JString::from("/mob/char.png")));
}

fn onUpdate(this: &Rc<Entity>)
{
    let player: &PlayerFields = fields(this);
    let world: Rc<World> = this.world();
    if this.isPlayerSleeping()
    {
        player.m_sleepTimer.set(player.m_sleepTimer.get() + 1);
        if player.m_sleepTimer.get() > 100
        {
            player.m_sleepTimer.set(100);
        }
        if !world.m_multiplayerWorld.get()
        {
            if !sleep::isInBed(this)
            {
                this.wakeUpPlayer(true, true, false);
            }
            else if world.isDaytime()
            {
                this.wakeUpPlayer(false, true, true);
            }
        }
    }
    else if player.m_sleepTimer.get() > 0
    {
        player.m_sleepTimer.set(player.m_sleepTimer.get() + 1);
        if player.m_sleepTimer.get() >= 110
        {
            player.m_sleepTimer.set(0);
        }
    }
    (entity_living::CLASS.onUpdate)(this);
    if !world.m_multiplayerWorld.get()
        && let Some(crafting) = player.m_craftingInventory.get()
        && !crafting.isUsableByPlayer(this)
    {
        this.closeScreen();
        player.m_craftingInventory.set(player.m_inventorySlots.get());
    }
    player.m_lastCapeX.set(player.m_capeX.get());
    player.m_lastCapeY.set(player.m_capeY.get());
    player.m_lastCapeZ.set(player.m_capeZ.get());
    let d0: f64 = this.posX() - player.m_capeX.get();
    let d1: f64 = this.posY() - player.m_capeY.get();
    let d2: f64 = this.posZ() - player.m_capeZ.get();
    let d3: f64 = 10.0;
    let snapX = || {
        player.m_capeX.set(this.posX());
        player.m_lastCapeX.set(this.posX());
    };
    let snapY = || {
        player.m_capeY.set(this.posY());
        player.m_lastCapeY.set(this.posY());
    };
    let snapZ = || {
        player.m_capeZ.set(this.posZ());
        player.m_lastCapeZ.set(this.posZ());
    };
    if d0 > d3
    {
        snapX();
    }
    if d2 > d3
    {
        snapZ();
    }
    if d1 > d3
    {
        snapY();
    }
    if d0 < -d3
    {
        snapX();
    }
    if d2 < -d3
    {
        snapZ();
    }
    if d1 < -d3
    {
        snapY();
    }
    player.m_capeX.set(player.m_capeX.get() + d0 * 0.25);
    player.m_capeZ.set(player.m_capeZ.get() + d2 * 0.25);
    player.m_capeY.set(player.m_capeY.get() + d1 * 0.25);
    addStat(this, StatList::minutesPlayedStat(), 1);
    if this.m_ridingEntity.with(Option::is_none)
    {
        player.m_startMinecartRidingCoordinate.set(None);
    }
}

fn onLivingUpdate(this: &Rc<Entity>)
{
    let player: &PlayerFields = fields(this);
    let living: &entity_living::LivingFields = entity_living::fields(this);
    let world: Rc<World> = this.world();
    if world.m_difficultySetting.get() == 0 && living.m_health.get() < 20 && this.m_ticksExisted.get() % 20 * 12 == 0
    {
        this.heal(1);
    }
    inventory(this).decrementAnimations();
    player.m_lastBob.set(player.m_bob.get());
    (entity_living::CLASS.onLivingUpdate)(this);
    let mut f: f32 = math_helper::sqrt_double(
        this.m_motionX.get() * this.m_motionX.get() + this.m_motionZ.get() * this.m_motionZ.get(),
    );
    let mut f1: f32 = math::atan(-this.m_motionY.get() * 0.2f32 as f64) as f32 * 15.0;
    if f > 0.1
    {
        f = 0.1;
    }
    if !this.m_onGround.get() || living.m_health.get() <= 0
    {
        f = 0.0;
    }
    if this.m_onGround.get() || living.m_health.get() <= 0
    {
        f1 = 0.0;
    }
    player.m_bob.set(player.m_bob.get() + (f - player.m_bob.get()) * 0.4);
    living.m_tilt.set(living.m_tilt.get() + (f1 - living.m_tilt.get()) * 0.8);
    if living.m_health.get() > 0
    {
        let list: Rc<jvm::array_list::JArrayList<Rc<Entity>>> =
            world.getEntitiesWithinAABBExcludingEntity(Some(this), &this.m_boundingBox.expand(1.0, 0.0, 1.0));
        let mut i: i32 = 0;
        while i < list.size()
        {
            let entity: Rc<Entity> = list.get(i);
            if !entity.m_isDead.get()
            {
                entity.onCollideWithPlayer(this);
            }
            i += 1;
        }
    }
}

pub fn getScore(this: &Entity) -> i32 { fields(this).m_score.get() }

pub fn dropCurrentItem(this: &Rc<Entity>)
{
    let inventory: Rc<InventoryPlayer> = inventory(this);
    dropPlayerItemWithRandomChoice(this, inventory.decrStackSize(inventory.m_currentItem.get(), 1), false);
}

pub fn dropPlayerItem(this: &Rc<Entity>, item: Option<Rc<ItemStack>>)
{
    dropPlayerItemWithRandomChoice(this, item, false);
}

const PI_F: f32 = std::f64::consts::PI as f32;

pub fn dropPlayerItemWithRandomChoice(this: &Rc<Entity>, item: Option<Rc<ItemStack>>, dead: bool)
{
    let Some(item) = item
    else
    {
        return;
    };
    let entityitem: Rc<Entity> = entity_item::create(
        &this.world(),
        this.posX(),
        this.posY() - 0.3f32 as f64 + this.getEyeHeight() as f64,
        this.posZ(),
        item,
    );
    entity_item::fields(&entityitem).m_delayBeforeCanPickup.set(40);
    let mut f: f32 = 0.1;
    if dead
    {
        let f1: f32 = this.m_rand.nextFloat() * 0.5;
        let f2: f32 = this.m_rand.nextFloat() * PI_F * 2.0;
        entityitem.m_motionX.set((-math_helper::sin(f2) * f1) as f64);
        entityitem.m_motionZ.set((math_helper::cos(f2) * f1) as f64);
        entityitem.m_motionY.set(0.2f32 as f64);
    }
    else
    {
        f = 0.3;
        let yaw: f32 = this.m_rotationYaw.get() / 180.0 * PI_F;
        let pitch: f32 = this.m_rotationPitch.get() / 180.0 * PI_F;
        entityitem.m_motionX.set((-math_helper::sin(yaw) * math_helper::cos(pitch) * f) as f64);
        entityitem.m_motionZ.set((math_helper::cos(yaw) * math_helper::cos(pitch) * f) as f64);
        entityitem.m_motionY.set((-math_helper::sin(pitch) * f + 0.1) as f64);
        f = 0.02;
        let f3: f32 = this.m_rand.nextFloat() * PI_F * 2.0;
        f *= this.m_rand.nextFloat();
        entityitem.m_motionX.set(entityitem.m_motionX.get() + math::cos(f3 as f64) * f as f64);
        entityitem
            .m_motionY
            .set(entityitem.m_motionY.get() + ((this.m_rand.nextFloat() - this.m_rand.nextFloat()) * 0.1) as f64);
        entityitem.m_motionZ.set(entityitem.m_motionZ.get() + math::sin(f3 as f64) * f as f64);
    }
    this.joinEntityItemWithWorld(&entityitem);
    addStat(this, StatList::dropStat(), 1);
}

pub fn getCurrentPlayerStrVsBlock(this: &Rc<Entity>, block: &Rc<Block>) -> f32
{
    let mut f: f32 = inventory(this).getStrVsBlock(block);
    if this.isInsideOfMaterial(&crate::block::Material::water())
    {
        f /= 5.0;
    }
    if !this.m_onGround.get()
    {
        f /= 5.0;
    }
    f
}

pub fn canHarvestBlock(this: &Rc<Entity>, block: &Rc<Block>) -> bool { inventory(this).canHarvestBlock(block) }

pub fn getCurrentEquippedItem(this: &Entity) -> Option<Rc<ItemStack>> { inventory(this).getCurrentItem() }

pub fn destroyCurrentEquippedItem(this: &Entity)
{
    let inventory: Rc<InventoryPlayer> = inventory(this);
    inventory.setInventorySlotContents(inventory.m_currentItem.get(), None);
}

fn onItemStackChanged(_this: &Rc<Entity>, _item: Option<Rc<ItemStack>>) {}

pub fn getPlayerSpawnCoordinate(this: &Entity) -> Option<ChunkCoordinates>
{
    fields(this).m_playerSpawnCoordinate.get()
}

pub fn setPlayerSpawnCoordinate(this: &Entity, pos: Option<ChunkCoordinates>)
{
    fields(this).m_playerSpawnCoordinate.set(pos.map(|pos| ChunkCoordinates::withCopy(&pos)));
}

pub fn triggerAchievement(this: &Rc<Entity>, stat: Option<Rc<StatBase>>) { this.addStat(stat.as_ref(), 1); }

pub const CLASS: EntityClass = EntityClass {
    m_name: "EntityPlayer",
    m_parent: Some(&entity_living::CLASS),
    entityInit: |this| {
        (entity_living::CLASS.entityInit)(this);
        this.m_dataWatcher.addObject(16, crate::entity::WatchedValue::Byte(0));
    },
    onUpdate,
    isMovementBlocked: |this| entity_living::fields(this).m_health.get() <= 0 || this.isPlayerSleeping(),
    closeScreen: |this| fields(this).m_craftingInventory.set(fields(this).m_inventorySlots.get()),
    updateCloak: |this| {
        let player: &PlayerFields = fields(this);
        let url: JString = JString::from(
            format!(
                "http://s3.amazonaws.com/MinecraftCloaks/{}.png",
                JString::display(player.m_username.get().as_ref())
            )
            .as_str(),
        );
        player.m_playerCloakUrl.set(Some(url));
        this.m_cloakUrl.set(player.m_playerCloakUrl.get());
    },
    updateRidden: |this| {
        let d0: f64 = this.posX();
        let d1: f64 = this.posY();
        let d2: f64 = this.posZ();
        (entity_living::CLASS.updateRidden)(this);
        let player: &PlayerFields = fields(this);
        player.m_lastBob.set(player.m_bob.get());
        player.m_bob.set(0.0);
        stats::addMountedMovementStat(this, this.posX() - d0, this.posY() - d1, this.posZ() - d2);
    },
    preparePlayerToSpawn: |this| {
        this.m_yOffset.set(1.62);
        this.setSize(0.6, 1.8);
        (entity_living::CLASS.preparePlayerToSpawn)(this);
        entity_living::fields(this).m_health.set(20);
        entity_living::fields(this).m_deathTime.set(0);
    },
    updatePlayerActionState: |this| {
        let player: &PlayerFields = fields(this);
        if player.m_isSwinging.get()
        {
            player.m_swingProgressInt.set(player.m_swingProgressInt.get() + 1);
            if player.m_swingProgressInt.get() >= 8
            {
                player.m_swingProgressInt.set(0);
                player.m_isSwinging.set(false);
            }
        }
        else
        {
            player.m_swingProgressInt.set(0);
        }
        entity_living::fields(this).m_swingProgress.set(player.m_swingProgressInt.get() as f32 / 8.0);
    },
    onLivingUpdate,
    onDeath: combat::onDeath,
    onItemStackChanged,
    pvpEnabled: combat::pvpEnabled,
    sleepInBedAt: sleep::sleepInBedAt,
    wakeUpPlayer: sleep::wakeUpPlayer,
    addToPlayerScore: |this, victim, score| {
        let player: &PlayerFields = fields(this);
        player.m_score.set(player.m_score.get() + score);
        if victim.is_some_and(|victim| victim.instanceOf("EntityPlayer"))
        {
            addStat(this, StatList::playerKillsStat(), 1);
        }
        else
        {
            addStat(this, StatList::mobKillsStat(), 1);
        }
    },
    dropCurrentItem,
    joinEntityItemWithWorld: |this, item| {
        this.world().entityJoinedWorld(item);
    },
    readEntityFromNBT: stats::readEntityFromNBT,
    writeEntityToNBT: stats::writeEntityToNBT,
    displayGUIChest: |_, _| {},
    displayWorkbenchGUI: |_, _, _, _| {},
    onItemPickup: |_, _, _| {},
    getEyeHeight: |_| 0.12,
    resetHeight: |this| this.m_yOffset.set(1.62),
    attackEntityFrom: combat::attackEntityFrom,
    damageEntity: combat::damageEntity,
    displayGUIFurnace: |_, _| {},
    displayGUIDispenser: |_, _| {},
    displayGUIEditSign: |_, _| {},
    getYOffset: |this| (this.m_yOffset.get() - 0.5) as f64,
    swingItem: |this| {
        fields(this).m_swingProgressInt.set(-1);
        fields(this).m_isSwinging.set(true);
    },
    respawnPlayer: |_| {},
    setEntityDead: |this| {
        (entity_living::CLASS.setEntityDead)(this);
        let player: &PlayerFields = fields(this);
        player.m_inventorySlots.get().orNpe().onCraftGuiClosed(this);
        if let Some(crafting) = player.m_craftingInventory.get()
        {
            crafting.onCraftGuiClosed(this);
        }
    },
    isEntityInsideOpaqueBlock: |this| {
        !fields(this).m_sleeping.get() && (entity_living::CLASS.isEntityInsideOpaqueBlock)(this)
    },
    isPlayerSleeping: |this| fields(this).m_sleeping.get(),
    addChatMessage: |_, _| {},
    addStat: |_, _, _| {},
    jump: |this| {
        (entity_living::CLASS.jump)(this);
        addStat(this, StatList::jumpStat(), 1);
    },
    moveEntityWithHeading: |this, sideways, forwards| {
        let d0: f64 = this.posX();
        let d1: f64 = this.posY();
        let d2: f64 = this.posZ();
        (entity_living::CLASS.moveEntityWithHeading)(this, sideways, forwards);
        stats::addMovementStat(this, this.posX() - d0, this.posY() - d1, this.posZ() - d2);
    },
    fall: |this, distance| {
        if distance >= 2.0
        {
            addStat(this, StatList::distanceFallenStat(), math::roundDouble(distance as f64 * 100.0) as i32);
        }
        (entity_living::CLASS.fall)(this, distance);
    },
    onKillEntity: |this, victim| {
        if victim.instanceOf("EntityMob")
        {
            triggerAchievement(this, crate::stats::AchievementList::killEnemy());
        }
    },
    getItemIcon: |this, item| {
        let mut i: i32 = (entity_living::CLASS.getItemIcon)(this, item);
        if item.m_itemID.get() == Item::fishingRod().m_shiftedIndex && fields(this).m_fishEntity.with(Option::is_some)
        {
            i = item.getIconIndex() + 16;
        }
        i
    },
    setInPortal: |this| {
        let player: &PlayerFields = fields(this);
        if player.m_timeUntilPortal.get() > 0
        {
            player.m_timeUntilPortal.set(10);
        }
        else
        {
            player.m_inPortal.set(true);
        }
    },
    ..entity_living::CLASS
};
