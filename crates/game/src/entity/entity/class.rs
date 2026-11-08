use super::{movement, riding, update};
use crate::entity::Entity;
use crate::entity::entity_player::EnumStatus;
use crate::inventory::IInventory;
use crate::item::ItemStack;
use crate::math::{AxisAlignedBB, Vec3D};
use crate::stats::StatBase;
use crate::tileentity::TileEntity;
use crate::world::World;
use common::nbt::NBTTagCompound;
use jvm::string::JString;
use std::rc::Rc;

crate::vtable! {
    EntityClass for Entity {
        fn addToPlayerScore(victim: Option<&Rc<Entity>>, score: i32);
        fn addVelocity(dx: f64, dy: f64, dz: f64);
        fn applyEntityCollision(entity: &Rc<Entity>);
        fn attackEntityFrom(source: Option<&Rc<Entity>>, amount: i32) -> bool;
        fn canBeCollidedWith() -> bool;
        fn canBePushed() -> bool;
        fn canTriggerWalking() -> bool;
        fn dealFireDamage(amount: i32);
        fn entityInit();
        fn fall(distance: f32);
        fn getBoundingBox() -> Option<Rc<AxisAlignedBB>>;
        fn getCollisionBorderSize() -> f32;
        fn getCollisionBox(other: &Rc<Entity>) -> Option<Rc<AxisAlignedBB>>;
        fn getEntityBrightness(tickDelta: f32) -> f32;
        fn getEntityTexture() -> Option<JString>;
        fn getEyeHeight() -> f32;
        fn getInventory() -> Option<Vec<Option<Rc<ItemStack>>>>;
        fn getLookVec() -> Option<Rc<Vec3D>>;
        fn getMountedYOffset() -> f64;
        fn getShadowSize() -> f32;
        fn getYOffset() -> f64;
        fn handleHealthUpdate(event: i8);
        fn handleWaterMovement() -> bool;
        fn interact(player: &Rc<Entity>) -> bool;
        fn isEntityAlive() -> bool;
        fn isEntityInsideOpaqueBlock() -> bool;
        fn isInRangeToRenderDist(distance: f64) -> bool;
        fn isInRangeToRenderVec3D(cameraPos: &Vec3D) -> bool;
        fn isInWater() -> bool;
        fn isSneaking() -> bool;
        fn kill();
        fn moveEntity(dx: f64, dy: f64, dz: f64);
        fn onCollideWithPlayer(player: &Rc<Entity>);
        fn onEntityUpdate();
        fn onKillEntity(victim: &Rc<Entity>);
        fn onStruckByLightning(lightning: &Rc<Entity>);
        fn onUpdate();
        fn outfitWithItem(slot: i32, item: i32, metadata: i32);
        fn performHurtAnimation();
        fn preparePlayerToSpawn();
        fn pushOutOfBlocks(x: f64, y: f64, z: f64) -> bool;
        fn readEntityFromNBT(nbt: &NBTTagCompound);
        fn setEntityDead();
        fn setInPortal();
        fn setPositionAndRotation2(x: f64, y: f64, z: f64, yaw: f32, pitch: f32, steps: i32);
        fn setSize(width: f32, height: f32);
        fn setVelocity(x: f64, y: f64, z: f64);
        fn updateCloak();
        fn updateRidden();
        fn updateRiderPosition();
        fn writeEntityToNBT(nbt: &NBTTagCompound);
        fn mountEntity(entity: Option<&Rc<Entity>>);
        fn setWorld(world: Option<Rc<World>>);
        fn updateFallState(dy: f64, onGround: bool);
        fn addChatMessage(message: &str);
        fn addRespawnParticles();
        fn addStat(stat: Option<&Rc<StatBase>>, amount: i32);
        fn attackBlockedEntity(target: &Rc<Entity>, distance: f32);
        fn attackEntity(target: &Rc<Entity>, distance: f32);
        fn canBreatheUnderwater() -> bool;
        fn canDespawn() -> bool;
        fn closeScreen();
        fn damageEntity(amount: i32);
        fn displayGUIChest(inventory: Option<&Rc<dyn IInventory>>);
        fn displayGUIDispenser(dispenser: Option<&Rc<TileEntity>>);
        fn displayGUIEditSign(sign: &Rc<TileEntity>);
        fn displayGUIFurnace(furnace: Option<&Rc<TileEntity>>);
        fn displayWorkbenchGUI(x: i32, y: i32, z: i32);
        fn dropCurrentItem();
        fn dropFewItems();
        fn findPlayerToAttack() -> Option<Rc<Entity>>;
        fn getBlockPathWeight(x: i32, y: i32, z: i32) -> f32;
        fn getCanSpawnHere() -> bool;
        fn getDeathSound() -> Option<String>;
        fn getDropItemId() -> i32;
        fn getFXLayer() -> i32;
        fn getHeldItem() -> Option<Rc<ItemStack>>;
        fn getHurtSound() -> Option<String>;
        fn getItemIcon(stack: &Rc<ItemStack>) -> i32;
        fn getLivingSound() -> Option<String>;
        fn getMaxSpawnedInChunk() -> i32;
        fn getSoundVolume() -> f32;
        fn getTalkInterval() -> i32;
        fn heal(amount: i32);
        fn isMovementBlocked() -> bool;
        fn isMovementCeased() -> bool;
        fn isOnLadder() -> bool;
        fn isPlayerSleeping() -> bool;
        fn joinEntityItemWithWorld(item: &Rc<Entity>);
        fn jump();
        fn moveEntityWithHeading(strafe: f32, forward: f32);
        fn onDeath(killer: Option<&Rc<Entity>>);
        fn onItemPickup(item: &Rc<Entity>, count: i32);
        fn onItemStackChanged(item: Option<Rc<ItemStack>>);
        fn onLivingUpdate();
        fn pvpEnabled() -> bool;
        fn renderParticle(tessellator: &dyn std::any::Any, tickDelta: f32, rotationX: f32, rotationXZ: f32, rotationZ: f32, rotationYZ: f32, rotationXY: f32);
        fn resetHeight();
        fn respawnPlayer();
        fn sendChatMessage(message: &str);
        fn setHealth(health: i32);
        fn sleepInBedAt(x: i32, y: i32, z: i32) -> EnumStatus;
        fn swingItem();
        fn updatePlayerActionState();
        fn verticleLookSpeed() -> i32;
        fn wakeUpPlayer(resetSleepTimer: bool, updateAllPlayersSleeping: bool, setSpawnPoint: bool);
    }
}

use crate::port::abstractMethod;

// abstract java methods get a slot that throws the AbstractMethodError java would
pub const ENTITY: EntityClass = EntityClass {
    m_name: "Entity",
    m_parent: None,
    m_interfaces: &[],
    mountEntity: super::riding::mountEntity,
    setWorld: super::setWorld,
    updateFallState: movement::updateFallState,
    onItemStackChanged: |_, _| abstractMethod("onItemStackChanged"),
    pvpEnabled: |_| abstractMethod("pvpEnabled"),
    sleepInBedAt: |_, _, _, _| abstractMethod("sleepInBedAt"),
    wakeUpPlayer: |_, _, _, _| abstractMethod("wakeUpPlayer"),
    addChatMessage: |_, _| abstractMethod("addChatMessage"),
    addRespawnParticles: |_| abstractMethod("addRespawnParticles"),
    addStat: |_, _, _| abstractMethod("addStat"),
    attackBlockedEntity: |_, _, _| abstractMethod("attackBlockedEntity"),
    attackEntity: |_, _, _| abstractMethod("attackEntity"),
    canBreatheUnderwater: |_| abstractMethod("canBreatheUnderwater"),
    canDespawn: |_| abstractMethod("canDespawn"),
    closeScreen: |_| abstractMethod("closeScreen"),
    damageEntity: |_, _| abstractMethod("damageEntity"),
    displayGUIChest: |_, _| abstractMethod("displayGUIChest"),
    displayGUIDispenser: |_, _| abstractMethod("displayGUIDispenser"),
    displayGUIEditSign: |_, _| abstractMethod("displayGUIEditSign"),
    displayGUIFurnace: |_, _| abstractMethod("displayGUIFurnace"),
    displayWorkbenchGUI: |_, _, _, _| abstractMethod("displayWorkbenchGUI"),
    dropCurrentItem: |_| abstractMethod("dropCurrentItem"),
    dropFewItems: |_| abstractMethod("dropFewItems"),
    findPlayerToAttack: |_| abstractMethod("findPlayerToAttack"),
    getBlockPathWeight: |_, _, _, _| abstractMethod("getBlockPathWeight"),
    getCanSpawnHere: |_| abstractMethod("getCanSpawnHere"),
    getDeathSound: |_| abstractMethod("getDeathSound"),
    getDropItemId: |_| abstractMethod("getDropItemId"),
    getFXLayer: |_| abstractMethod("getFXLayer"),
    getHeldItem: |_| abstractMethod("getHeldItem"),
    getHurtSound: |_| abstractMethod("getHurtSound"),
    getItemIcon: |_, _| abstractMethod("getItemIcon"),
    getLivingSound: |_| abstractMethod("getLivingSound"),
    getMaxSpawnedInChunk: |_| abstractMethod("getMaxSpawnedInChunk"),
    getSoundVolume: |_| abstractMethod("getSoundVolume"),
    getTalkInterval: |_| abstractMethod("getTalkInterval"),
    heal: |_, _| abstractMethod("heal"),
    isMovementBlocked: |_| abstractMethod("isMovementBlocked"),
    isMovementCeased: |_| abstractMethod("isMovementCeased"),
    isOnLadder: |_| abstractMethod("isOnLadder"),
    isPlayerSleeping: |_| abstractMethod("isPlayerSleeping"),
    joinEntityItemWithWorld: |_, _| abstractMethod("joinEntityItemWithWorld"),
    jump: |_| abstractMethod("jump"),
    moveEntityWithHeading: |_, _, _| abstractMethod("moveEntityWithHeading"),
    onDeath: |_, _| abstractMethod("onDeath"),
    onItemPickup: |_, _, _| abstractMethod("onItemPickup"),
    onLivingUpdate: |_| abstractMethod("onLivingUpdate"),
    renderParticle: |_, _, _, _, _, _, _, _| abstractMethod("renderParticle"),
    resetHeight: |_| abstractMethod("resetHeight"),
    respawnPlayer: |_| abstractMethod("respawnPlayer"),
    sendChatMessage: |_, _| abstractMethod("sendChatMessage"),
    setHealth: |_, _| abstractMethod("setHealth"),
    swingItem: |_| abstractMethod("swingItem"),
    updatePlayerActionState: |_| abstractMethod("updatePlayerActionState"),
    verticleLookSpeed: |_| abstractMethod("verticleLookSpeed"),
    addToPlayerScore: |_, _, _| {},
    addVelocity: update::addVelocity,
    applyEntityCollision: update::applyEntityCollision,
    attackEntityFrom: update::attackEntityFrom,
    canBeCollidedWith: |_| false,
    canBePushed: |_| false,
    canTriggerWalking: |_| true,
    dealFireDamage: update::dealFireDamage,
    entityInit: |_| abstractMethod("entityInit"),
    fall: update::fall,
    getBoundingBox: |_| None,
    getCollisionBorderSize: |_| 0.1,
    getCollisionBox: |_, _| None,
    getEntityBrightness: update::getEntityBrightness,
    getEntityTexture: |_| None,
    getInventory: |_| None,
    getEyeHeight: |_| 0.0,
    getLookVec: |_| None,
    getMountedYOffset: riding::getMountedYOffset,
    getShadowSize: |this| this.m_height.get() / 2.0,
    getYOffset: |this| this.m_yOffset.get() as f64,
    handleHealthUpdate: |_, _| {},
    handleWaterMovement: update::handleWaterMovement,
    interact: |_, _| false,
    isEntityAlive: |this| !this.m_isDead.get(),
    isEntityInsideOpaqueBlock: update::isEntityInsideOpaqueBlock,
    isInRangeToRenderDist: update::isInRangeToRenderDist,
    isInRangeToRenderVec3D: update::isInRangeToRenderVec3D,
    isInWater: |this| this.m_inWater.get(),
    isSneaking: |this| this.getEntityFlag(1),
    kill: |this| this.setEntityDead(),
    moveEntity: movement::moveEntity,
    onCollideWithPlayer: |_, _| {},
    onEntityUpdate: update::onEntityUpdate,
    onKillEntity: |_, _| {},
    onStruckByLightning: update::onStruckByLightning,
    onUpdate: |this| this.onEntityUpdate(),
    outfitWithItem: |_, _, _, _| {},
    performHurtAnimation: |_| {},
    preparePlayerToSpawn: update::preparePlayerToSpawn,
    pushOutOfBlocks: movement::pushOutOfBlocks,
    readEntityFromNBT: |_, _| abstractMethod("readEntityFromNBT"),
    setEntityDead: |this| this.m_isDead.set(true),
    setInPortal: |_| {},
    setPositionAndRotation2: movement::setPositionAndRotation2,
    setSize: |this, width, height| {
        this.m_width.set(width);
        this.m_height.set(height);
    },
    setVelocity: |this, x, y, z| {
        this.m_motionX.set(x);
        this.m_motionY.set(y);
        this.m_motionZ.set(z);
    },
    updateCloak: |_| {},
    updateRidden: riding::updateRidden,
    updateRiderPosition: riding::updateRiderPosition,
    writeEntityToNBT: |_, _| abstractMethod("writeEntityToNBT"),
};
