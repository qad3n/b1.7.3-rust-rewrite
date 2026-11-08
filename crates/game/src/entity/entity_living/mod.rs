mod ai;
mod combat;
pub(crate) mod update;

use crate::entity::{ENTITY, Entity, EntityClass};
use crate::world::World;
pub use ai::{checkDespawn, faceEntity, getCurrentTarget, getLook, getPosition, hasCurrentTarget, rayTrace};
pub use combat::{knockBack, onEntityDeath, playLivingSound, spawnExplosionParticle};
use jvm::field::Field;
use jvm::math;
use jvm::string::JString;
use std::cell::Cell;
use std::rc::Rc;
pub use update::canEntityBeSeen;

#[derive(Default)]
pub struct LivingFields
{
    pub m_heartsHalvesLife: Cell<i32>,
    pub m_field_9365_p: Cell<f32>,
    pub m_field_9363_r: Cell<f32>,
    pub m_renderYawOffset: Cell<f32>,
    pub m_prevRenderYawOffset: Cell<f32>,
    pub m_lastWalkProgress: Cell<f32>,
    pub m_walkProgress: Cell<f32>,
    pub m_totalWalkDistance: Cell<f32>,
    pub m_lastTotalWalkDistance: Cell<f32>,
    pub m_field_9358_y: Cell<bool>,
    pub m_texture: Field<Option<JString>>,
    pub m_field_9355_A: Cell<bool>,
    pub m_rotationOffset: Cell<f32>,
    pub m_modelName: Field<Option<JString>>,
    pub m_field_9349_D: Cell<f32>,
    pub m_scoreValue: Cell<i32>,
    pub m_lastDamageTaken: Cell<f32>,
    pub m_isMultiplayerEntity: Cell<bool>,
    pub m_prevSwingProgress: Cell<f32>,
    pub m_swingProgress: Cell<f32>,
    pub m_health: Cell<i32>,
    pub m_prevHealth: Cell<i32>,
    pub m_livingSoundTime: Cell<i32>,
    pub m_hurtTime: Cell<i32>,
    pub m_maxHurtTime: Cell<i32>,
    pub m_attackedAtYaw: Cell<f32>,
    pub m_deathTime: Cell<i32>,
    pub m_attackTime: Cell<i32>,
    pub m_cameraPitch: Cell<f32>,
    pub m_tilt: Cell<f32>,
    pub m_unused_flag: Cell<bool>,
    pub m_field_9326_T: Cell<i32>,
    pub m_field_9325_U: Cell<f32>,
    pub m_lastWalkAnimationSpeed: Cell<f32>,
    pub m_walkAnimationSpeed: Cell<f32>,
    pub m_walkAnimationProgress: Cell<f32>,
    pub m_newPosRotationIncrements: Cell<i32>,
    pub m_newPosX: Cell<f64>,
    pub m_newPosY: Cell<f64>,
    pub m_newPosZ: Cell<f64>,
    pub m_newRotationYaw: Cell<f64>,
    pub m_newRotationPitch: Cell<f64>,
    pub m_field_9348_ae: Cell<f32>,
    pub m_prevDamageTaken: Cell<i32>,
    pub m_entityAge: Cell<i32>,
    pub m_moveStrafing: Cell<f32>,
    pub m_moveForward: Cell<f32>,
    pub m_randomYawVelocity: Cell<f32>,
    pub m_isJumping: Cell<bool>,
    pub m_defaultPitch: Cell<f32>,
    pub m_moveSpeed: Cell<f32>,
    pub m_currentTarget: Field<Option<Rc<Entity>>>,
    pub m_numTicksToChaseTarget: Cell<i32>,
}

pub fn fields(this: &Entity) -> &LivingFields
{
    match this.m_ext.living()
    {
        Some(fields) => fields,
        None => jvm::throwable::throw(
            &jvm::throwable::CLASS_CAST,
            &format!("net.minecraft.src.{} cannot be cast to net.minecraft.src.EntityLiving", this.className()),
        ),
    }
}

fn initFields(living: &LivingFields)
{
    living.m_heartsHalvesLife.set(20);
    living.m_renderYawOffset.set(0.0);
    living.m_prevRenderYawOffset.set(0.0);
    living.m_field_9358_y.set(true);
    living.m_texture.set(Some(JString::from("/mob/char.png")));
    living.m_field_9355_A.set(true);
    living.m_rotationOffset.set(0.0);
    living.m_modelName.set(None);
    living.m_field_9349_D.set(1.0);
    living.m_scoreValue.set(0);
    living.m_lastDamageTaken.set(0.0);
    living.m_isMultiplayerEntity.set(false);
    living.m_attackedAtYaw.set(0.0);
    living.m_deathTime.set(0);
    living.m_attackTime.set(0);
    living.m_unused_flag.set(false);
    living.m_field_9326_T.set(-1);
    living.m_field_9325_U.set((math::random() * 0.9f32 as f64 + 0.1f32 as f64) as f32);
    living.m_field_9348_ae.set(0.0);
    living.m_prevDamageTaken.set(0);
    living.m_entityAge.set(0);
    living.m_isJumping.set(false);
    living.m_defaultPitch.set(0.0);
    living.m_moveSpeed.set(0.7);
    living.m_currentTarget.set(None);
    living.m_numTicksToChaseTarget.set(0);
}

pub fn init(this: &Rc<Entity>, world: Option<Rc<World>>)
{
    Entity::init(this, world);
    let living: &LivingFields = fields(this);
    initFields(living);
    living.m_health.set(10);
    this.m_preventEntitySpawning.set(true);
    living.m_field_9363_r.set((math::random() + 1.0) as f32 * 0.01);
    this.setPosition(this.posX(), this.posY(), this.posZ());
    living.m_field_9365_p.set(math::random() as f32 * 12398.0);
    this.m_rotationYaw.set((math::random() * std::f64::consts::PI as f32 as f64 * 2.0) as f32);
    this.m_stepHeight.set(0.5);
}

pub fn getSwingProgress(this: &Entity, tickDelta: f32) -> f32
{
    let living: &LivingFields = fields(this);
    let mut f: f32 = living.m_swingProgress.get() - living.m_prevSwingProgress.get();
    if f < 0.0
    {
        f += 1.0;
    }
    living.m_prevSwingProgress.get() + f * tickDelta
}

pub const CLASS: EntityClass = EntityClass {
    m_name: "EntityLiving",
    m_parent: Some(&ENTITY),
    entityInit: |_| {},
    getEntityTexture: |this| fields(this).m_texture.get(),
    canBeCollidedWith: |this| !this.m_isDead.get(),
    canBePushed: |this| !this.m_isDead.get(),
    getEyeHeight: |this| this.m_height.get() * 0.85,
    getTalkInterval: |_| 80,
    onEntityUpdate: update::onEntityUpdate,
    updateRidden: |this| {
        (ENTITY.updateRidden)(this);
        let living: &LivingFields = fields(this);
        living.m_lastWalkProgress.set(living.m_walkProgress.get());
        living.m_walkProgress.set(0.0);
    },
    setPositionAndRotation2: |this, x, y, z, yaw, pitch, steps| {
        let living: &LivingFields = fields(this);
        this.m_yOffset.set(0.0);
        living.m_newPosX.set(x);
        living.m_newPosY.set(y);
        living.m_newPosZ.set(z);
        living.m_newRotationYaw.set(yaw as f64);
        living.m_newRotationPitch.set(pitch as f64);
        living.m_newPosRotationIncrements.set(steps);
    },
    onUpdate: update::onUpdate,
    setSize: |this, width, height| (ENTITY.setSize)(this, width, height),
    heal: combat::heal,
    attackEntityFrom: combat::attackEntityFrom,
    performHurtAnimation: |this| {
        let living: &LivingFields = fields(this);
        living.m_maxHurtTime.set(10);
        living.m_hurtTime.set(10);
        living.m_attackedAtYaw.set(0.0);
    },
    damageEntity: |this, amount| fields(this).m_health.set(fields(this).m_health.get() - amount),
    getSoundVolume: |_| 1.0,
    getLivingSound: |_| None,
    getHurtSound: |_| Some("random.hurt".to_string()),
    getDeathSound: |_| Some("random.hurt".to_string()),
    onDeath: combat::onDeath,
    dropFewItems: combat::dropFewItems,
    getDropItemId: |_| 0,
    fall: combat::fall,
    moveEntityWithHeading: update::moveEntityWithHeading,
    isOnLadder: update::isOnLadder,
    writeEntityToNBT: |this, nbt| {
        let living: &LivingFields = fields(this);
        nbt.setShort("Health", living.m_health.get() as i16);
        nbt.setShort("HurtTime", living.m_hurtTime.get() as i16);
        nbt.setShort("DeathTime", living.m_deathTime.get() as i16);
        nbt.setShort("AttackTime", living.m_attackTime.get() as i16);
    },
    readEntityFromNBT: |this, nbt| {
        let living: &LivingFields = fields(this);
        living.m_health.set(nbt.getShort("Health") as i32);
        if !nbt.hasKey("Health")
        {
            living.m_health.set(10);
        }
        living.m_hurtTime.set(nbt.getShort("HurtTime") as i32);
        living.m_deathTime.set(nbt.getShort("DeathTime") as i32);
        living.m_attackTime.set(nbt.getShort("AttackTime") as i32);
    },
    isEntityAlive: |this| !this.m_isDead.get() && fields(this).m_health.get() > 0,
    canBreatheUnderwater: |_| false,
    onLivingUpdate: update::onLivingUpdate,
    isMovementBlocked: |this| fields(this).m_health.get() <= 0,
    jump: |this| this.m_motionY.set(0.42f32 as f64),
    canDespawn: |_| true,
    updatePlayerActionState: ai::updatePlayerActionState,
    verticleLookSpeed: |_| 40,
    getCanSpawnHere: |this| {
        let world: Rc<World> = this.world();
        world.checkIfAABBIsClear(&this.m_boundingBox)
            && world.getCollidingBoundingBoxes(this, &this.m_boundingBox).size() == 0
            && !world.getIsAnyLiquid(&this.m_boundingBox)
    },
    kill: |this| {
        this.attackEntityFrom(None, 4);
    },
    getLookVec: |this| Some(getLook(this, 1.0)),
    getMaxSpawnedInChunk: |_| 4,
    getHeldItem: |_| None,
    handleHealthUpdate: combat::handleHealthUpdate,
    isPlayerSleeping: |_| false,
    getItemIcon: |_, item| item.getIconIndex(),
    ..ENTITY
};
