use super::{LivingFields, fields};
use crate::block::{Block, Material};
use crate::entity::{ENTITY, Entity};
use crate::math::{AxisAlignedBB, Vec3D, math_helper};
use crate::world::World;
use jvm::array_list::JArrayList;
use jvm::math;
use jvm::throwable::OrNpe;
use std::rc::Rc;

pub fn canEntityBeSeen(this: &Rc<Entity>, entity: &Rc<Entity>) -> bool
{
    this.world()
        .rayTraceBlocks(
            &Vec3D::createVector(this.posX(), this.posY() + this.getEyeHeight() as f64, this.posZ()),
            &Vec3D::createVector(entity.posX(), entity.posY() + entity.getEyeHeight() as f64, entity.posZ()),
        )
        .is_none()
}

pub fn onEntityUpdate(this: &Rc<Entity>)
{
    let living: &LivingFields = fields(this);
    let world: Rc<World> = this.world();
    living.m_prevSwingProgress.set(living.m_swingProgress.get());
    (ENTITY.onEntityUpdate)(this);
    let soundTime: i32 = living.m_livingSoundTime.get();
    living.m_livingSoundTime.set(soundTime + 1);
    if this.m_rand.nextIntBounded(1000) < soundTime
    {
        living.m_livingSoundTime.set(-this.getTalkInterval());
        super::playLivingSound(this);
    }
    if this.isEntityAlive() && this.isEntityInsideOpaqueBlock()
    {
        this.attackEntityFrom(None, 1);
    }
    if this.m_isImmuneToFire.get() || world.m_multiplayerWorld.get()
    {
        this.m_fire.set(0);
    }
    if this.isEntityAlive() && this.isInsideOfMaterial(&Material::water()) && !this.canBreatheUnderwater()
    {
        this.m_air.set(this.m_air.get() - 1);
        if this.m_air.get() == -20
        {
            this.m_air.set(0);
            for _ in 0..8
            {
                let f: f32 = this.m_rand.nextFloat() - this.m_rand.nextFloat();
                let f1: f32 = this.m_rand.nextFloat() - this.m_rand.nextFloat();
                let f2: f32 = this.m_rand.nextFloat() - this.m_rand.nextFloat();
                world.spawnParticle(
                    "bubble",
                    this.posX() + f as f64,
                    this.posY() + f1 as f64,
                    this.posZ() + f2 as f64,
                    this.m_motionX.get(),
                    this.m_motionY.get(),
                    this.m_motionZ.get(),
                );
            }
            this.attackEntityFrom(None, 2);
        }
        this.m_fire.set(0);
    }
    else
    {
        this.m_air.set(this.m_maxAir.get());
    }
    living.m_cameraPitch.set(living.m_tilt.get());
    if living.m_attackTime.get() > 0
    {
        living.m_attackTime.set(living.m_attackTime.get() - 1);
    }
    if living.m_hurtTime.get() > 0
    {
        living.m_hurtTime.set(living.m_hurtTime.get() - 1);
    }
    if this.m_heartsLife.get() > 0
    {
        this.m_heartsLife.set(this.m_heartsLife.get() - 1);
    }
    if living.m_health.get() <= 0
    {
        living.m_deathTime.set(living.m_deathTime.get() + 1);
        if living.m_deathTime.get() > 20
        {
            super::onEntityDeath(this);
            this.setEntityDead();
            for _ in 0..20
            {
                let d1: f64 = this.m_rand.nextGaussian() * 0.02;
                let d2: f64 = this.m_rand.nextGaussian() * 0.02;
                let d0: f64 = this.m_rand.nextGaussian() * 0.02;
                let width: f32 = this.m_width.get();
                let x: f64 = this.posX() + (this.m_rand.nextFloat() * width * 2.0) as f64 - width as f64;
                let y: f64 = this.posY() + (this.m_rand.nextFloat() * this.m_height.get()) as f64;
                let z: f64 = this.posZ() + (this.m_rand.nextFloat() * width * 2.0) as f64 - width as f64;
                world.spawnParticle("explode", x, y, z, d1, d2, d0);
            }
        }
    }
    living.m_lastTotalWalkDistance.set(living.m_totalWalkDistance.get());
    living.m_prevRenderYawOffset.set(living.m_renderYawOffset.get());
    this.m_prevRotationYaw.set(this.m_rotationYaw.get());
    this.m_prevRotationPitch.set(this.m_rotationPitch.get());
}

pub fn onUpdate(this: &Rc<Entity>)
{
    let living: &LivingFields = fields(this);
    (ENTITY.onUpdate)(this);
    this.onLivingUpdate();
    let d0: f64 = this.posX() - this.m_prevPosX.get();
    let d1: f64 = this.posZ() - this.m_prevPosZ.get();
    let f: f32 = math_helper::sqrt_double(d0 * d0 + d1 * d1);
    let mut f1: f32 = living.m_renderYawOffset.get();
    let mut f2: f32 = 0.0;
    living.m_lastWalkProgress.set(living.m_walkProgress.get());
    let mut f3: f32 = 0.0;
    if !(f <= 0.05)
    {
        f3 = 1.0;
        f2 = f * 3.0;
        f1 = math::atan2(d1, d0) as f32 * 180.0 / std::f64::consts::PI as f32 - 90.0;
    }
    if living.m_swingProgress.get() > 0.0
    {
        f1 = this.m_rotationYaw.get();
    }
    if !this.m_onGround.get()
    {
        f3 = 0.0;
    }
    living.m_walkProgress.set(living.m_walkProgress.get() + (f3 - living.m_walkProgress.get()) * 0.3);
    let mut f4: f32 = f1 - living.m_renderYawOffset.get();
    while f4 < -180.0
    {
        f4 += 360.0;
    }
    while f4 >= 180.0
    {
        f4 -= 360.0;
    }
    living.m_renderYawOffset.set(living.m_renderYawOffset.get() + f4 * 0.3);
    let mut f5: f32 = this.m_rotationYaw.get() - living.m_renderYawOffset.get();
    while f5 < -180.0
    {
        f5 += 360.0;
    }
    while f5 >= 180.0
    {
        f5 -= 360.0;
    }
    let flag: bool = f5 < -90.0 || f5 >= 90.0;
    if f5 < -75.0
    {
        f5 = -75.0;
    }
    if f5 >= 75.0
    {
        f5 = 75.0;
    }
    living.m_renderYawOffset.set(this.m_rotationYaw.get() - f5);
    if f5 * f5 > 2500.0
    {
        living.m_renderYawOffset.set(living.m_renderYawOffset.get() + f5 * 0.2);
    }
    if flag
    {
        f2 *= -1.0;
    }
    while this.m_rotationYaw.get() - this.m_prevRotationYaw.get() < -180.0
    {
        this.m_prevRotationYaw.set(this.m_prevRotationYaw.get() - 360.0);
    }
    while this.m_rotationYaw.get() - this.m_prevRotationYaw.get() >= 180.0
    {
        this.m_prevRotationYaw.set(this.m_prevRotationYaw.get() + 360.0);
    }
    while living.m_renderYawOffset.get() - living.m_prevRenderYawOffset.get() < -180.0
    {
        living.m_prevRenderYawOffset.set(living.m_prevRenderYawOffset.get() - 360.0);
    }
    while living.m_renderYawOffset.get() - living.m_prevRenderYawOffset.get() >= 180.0
    {
        living.m_prevRenderYawOffset.set(living.m_prevRenderYawOffset.get() + 360.0);
    }
    while this.m_rotationPitch.get() - this.m_prevRotationPitch.get() < -180.0
    {
        this.m_prevRotationPitch.set(this.m_prevRotationPitch.get() - 360.0);
    }
    while this.m_rotationPitch.get() - this.m_prevRotationPitch.get() >= 180.0
    {
        this.m_prevRotationPitch.set(this.m_prevRotationPitch.get() + 360.0);
    }
    living.m_totalWalkDistance.set(living.m_totalWalkDistance.get() + f2);
}

pub(crate) fn groundSlipperiness(this: &Rc<Entity>, world: &World) -> f32
{
    let mut f: f32 = 0.91;
    if this.m_onGround.get()
    {
        f = 0.54600006;
        let n: i32 = world.getBlockId(
            math_helper::floor_double(this.posX()),
            math_helper::floor_double(this.m_boundingBox.minY()) - 1,
            math_helper::floor_double(this.posZ()),
        );
        if n > 0
        {
            f = Block::byId(n).orNpe().m_slipperiness.get() * 0.91;
        }
    }
    f
}

fn liquidMove(this: &Rc<Entity>, sideways: f32, forwards: f32, drag: f64)
{
    let d: f64 = this.posY();
    this.moveFlying(sideways, forwards, 0.02);
    this.moveEntity(this.m_motionX.get(), this.m_motionY.get(), this.m_motionZ.get());
    this.m_motionX.set(this.m_motionX.get() * drag);
    this.m_motionY.set(this.m_motionY.get() * drag);
    this.m_motionZ.set(this.m_motionZ.get() * drag);
    this.m_motionY.set(this.m_motionY.get() - 0.02);
    if this.m_isCollidedHorizontally.get()
        && this.isOffsetPositionInLiquid(
            this.m_motionX.get(),
            this.m_motionY.get() + 0.6f32 as f64 - this.posY() + d,
            this.m_motionZ.get(),
        )
    {
        this.m_motionY.set(0.3f32 as f64);
    }
}

pub fn moveEntityWithHeading(this: &Rc<Entity>, sideways: f32, forwards: f32)
{
    let living: &LivingFields = fields(this);
    let world: Rc<World> = this.world();
    if this.isInWater()
    {
        liquidMove(this, sideways, forwards, 0.8f32 as f64);
    }
    else if this.handleLavaMovement()
    {
        liquidMove(this, sideways, forwards, 0.5);
    }
    else
    {
        let mut f: f32 = groundSlipperiness(this, &world);
        let f2: f32 = 0.16277136 / (f * f * f);
        this.moveFlying(
            sideways,
            forwards,
            if this.m_onGround.get()
            {
                0.1 * f2
            }
            else
            {
                0.02
            },
        );
        f = groundSlipperiness(this, &world);
        if this.isOnLadder()
        {
            let f3: f32 = 0.15;
            if this.m_motionX.get() < (-f3) as f64
            {
                this.m_motionX.set((-f3) as f64);
            }
            if this.m_motionX.get() > f3 as f64
            {
                this.m_motionX.set(f3 as f64);
            }
            if this.m_motionZ.get() < (-f3) as f64
            {
                this.m_motionZ.set((-f3) as f64);
            }
            if this.m_motionZ.get() > f3 as f64
            {
                this.m_motionZ.set(f3 as f64);
            }
            this.m_fallDistance.set(0.0);
            if this.m_motionY.get() < -0.15
            {
                this.m_motionY.set(-0.15);
            }
            if this.isSneaking() && this.m_motionY.get() < 0.0
            {
                this.m_motionY.set(0.0);
            }
        }
        this.moveEntity(this.m_motionX.get(), this.m_motionY.get(), this.m_motionZ.get());
        if this.m_isCollidedHorizontally.get() && this.isOnLadder()
        {
            this.m_motionY.set(0.2);
        }
        this.m_motionY.set(this.m_motionY.get() - 0.08);
        this.m_motionY.set(this.m_motionY.get() * 0.98f32 as f64);
        this.m_motionX.set(this.m_motionX.get() * f as f64);
        this.m_motionZ.set(this.m_motionZ.get() * f as f64);
    }
    living.m_lastWalkAnimationSpeed.set(living.m_walkAnimationSpeed.get());
    let d: f64 = this.posX() - this.m_prevPosX.get();
    let d2: f64 = this.posZ() - this.m_prevPosZ.get();
    let mut f: f32 = math_helper::sqrt_double(d * d + d2 * d2) * 4.0;
    if f > 1.0
    {
        f = 1.0;
    }
    living.m_walkAnimationSpeed.set(living.m_walkAnimationSpeed.get() + (f - living.m_walkAnimationSpeed.get()) * 0.4);
    living.m_walkAnimationProgress.set(living.m_walkAnimationProgress.get() + living.m_walkAnimationSpeed.get());
}

pub fn isOnLadder(this: &Rc<Entity>) -> bool
{
    let i: i32 = math_helper::floor_double(this.posX());
    let j: i32 = math_helper::floor_double(this.m_boundingBox.minY());
    let k: i32 = math_helper::floor_double(this.posZ());
    this.world().getBlockId(i, j, k) == Block::ladder().m_blockID
}

pub fn onLivingUpdate(this: &Rc<Entity>)
{
    let living: &LivingFields = fields(this);
    let world: Rc<World> = this.world();
    if living.m_newPosRotationIncrements.get() > 0
    {
        let steps: f64 = living.m_newPosRotationIncrements.get() as f64;
        let d0: f64 = this.posX() + (living.m_newPosX.get() - this.posX()) / steps;
        let mut d1: f64 = this.posY() + (living.m_newPosY.get() - this.posY()) / steps;
        let d2: f64 = this.posZ() + (living.m_newPosZ.get() - this.posZ()) / steps;
        let mut d3: f64 = living.m_newRotationYaw.get() - this.m_rotationYaw.get() as f64;
        while d3 < -180.0
        {
            d3 += 360.0;
        }
        while d3 >= 180.0
        {
            d3 -= 360.0;
        }
        this.m_rotationYaw.set((this.m_rotationYaw.get() as f64 + d3 / steps) as f32);
        this.m_rotationPitch.set(
            (this.m_rotationPitch.get() as f64
                + (living.m_newRotationPitch.get() - this.m_rotationPitch.get() as f64) / steps) as f32,
        );
        living.m_newPosRotationIncrements.set(living.m_newPosRotationIncrements.get() - 1);
        this.setPosition(d0, d1, d2);
        this.setRotation(this.m_rotationYaw.get(), this.m_rotationPitch.get());
        let list: Rc<JArrayList<Rc<AxisAlignedBB>>> =
            world.getCollidingBoundingBoxes(this, &this.m_boundingBox.contract(0.03125, 0.0, 0.03125));
        if list.size() > 0
        {
            let mut d4: f64 = 0.0;
            for j in 0..list.size()
            {
                let axisalignedbb: Rc<AxisAlignedBB> = list.get(j);
                if axisalignedbb.maxY() > d4
                {
                    d4 = axisalignedbb.maxY();
                }
            }
            d1 += d4 - this.m_boundingBox.minY();
            this.setPosition(d0, d1, d2);
        }
    }
    if this.isMovementBlocked()
    {
        living.m_isJumping.set(false);
        living.m_moveStrafing.set(0.0);
        living.m_moveForward.set(0.0);
        living.m_randomYawVelocity.set(0.0);
    }
    else if !living.m_isMultiplayerEntity.get()
    {
        this.updatePlayerActionState();
    }
    let flag1: bool = this.isInWater();
    let flag: bool = this.handleLavaMovement();
    if living.m_isJumping.get()
    {
        if flag1
        {
            this.m_motionY.set(this.m_motionY.get() + 0.04f32 as f64);
        }
        else if flag
        {
            this.m_motionY.set(this.m_motionY.get() + 0.04f32 as f64);
        }
        else if this.m_onGround.get()
        {
            this.jump();
        }
    }
    living.m_moveStrafing.set(living.m_moveStrafing.get() * 0.98);
    living.m_moveForward.set(living.m_moveForward.get() * 0.98);
    living.m_randomYawVelocity.set(living.m_randomYawVelocity.get() * 0.9);
    this.moveEntityWithHeading(living.m_moveStrafing.get(), living.m_moveForward.get());
    let list1: Rc<JArrayList<Rc<Entity>>> = world.getEntitiesWithinAABBExcludingEntity(
        Some(this),
        &this.m_boundingBox.expand(0.2f32 as f64, 0.0, 0.2f32 as f64),
    );
    if list1.size() > 0
    {
        let mut i: i32 = 0;
        while i < list1.size()
        {
            let entity: Rc<Entity> = list1.get(i);
            if entity.canBePushed()
            {
                entity.applyEntityCollision(this);
            }
            i += 1;
        }
    }
}
