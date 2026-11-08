use super::{LivingFields, fields};
use crate::entity::Entity;
use crate::math::{MovingObjectPosition, Vec3D, math_helper};
use crate::world::World;
use jvm::math;
use std::rc::Rc;

const DEG_TO_RAD: f32 = (std::f64::consts::PI / 180.0) as f32;
const PI_F: f32 = std::f64::consts::PI as f32;

pub fn checkDespawn(this: &Rc<Entity>)
{
    let living: &LivingFields = fields(this);
    let entityplayer: Option<Rc<Entity>> = this.world().getClosestPlayerToEntity(this, -1.0);
    if this.canDespawn()
        && let Some(player) = entityplayer
    {
        let d0: f64 = player.posX() - this.posX();
        let d1: f64 = player.posY() - this.posY();
        let d2: f64 = player.posZ() - this.posZ();
        let d3: f64 = d0 * d0 + d1 * d1 + d2 * d2;
        if d3 > 16384.0
        {
            this.setEntityDead();
        }
        if living.m_entityAge.get() > 600 && this.m_rand.nextIntBounded(800) == 0
        {
            if d3 < 1024.0
            {
                living.m_entityAge.set(0);
            }
            else
            {
                this.setEntityDead();
            }
        }
    }
}

pub fn updatePlayerActionState(this: &Rc<Entity>)
{
    let living: &LivingFields = fields(this);
    let world: Rc<World> = this.world();
    living.m_entityAge.set(living.m_entityAge.get() + 1);
    let mut entityPlayer: Option<Rc<Entity>> = world.getClosestPlayerToEntity(this, -1.0);
    checkDespawn(this);
    living.m_moveStrafing.set(0.0);
    living.m_moveForward.set(0.0);
    let f: f32 = 8.0;
    if this.m_rand.nextFloat() < 0.02
    {
        entityPlayer = world.getClosestPlayerToEntity(this, f as f64);
        if let Some(player) = &entityPlayer
        {
            living.m_currentTarget.set(Some(Rc::clone(player)));
            living.m_numTicksToChaseTarget.set(10 + this.m_rand.nextIntBounded(20));
        }
        else
        {
            living.m_randomYawVelocity.set((this.m_rand.nextFloat() - 0.5) * 20.0);
        }
    }
    if let Some(target) = living.m_currentTarget.get()
    {
        faceEntity(this, &target, 10.0, this.verticleLookSpeed() as f32);
        let ticks: i32 = living.m_numTicksToChaseTarget.get();
        living.m_numTicksToChaseTarget.set(ticks - 1);
        if ticks <= 0 || target.m_isDead.get() || target.getDistanceSqToEntity(this) > (f * f) as f64
        {
            living.m_currentTarget.set(None);
        }
    }
    else
    {
        if this.m_rand.nextFloat() < 0.05
        {
            living.m_randomYawVelocity.set((this.m_rand.nextFloat() - 0.5) * 20.0);
        }
        this.m_rotationYaw.set(this.m_rotationYaw.get() + living.m_randomYawVelocity.get());
        this.m_rotationPitch.set(living.m_defaultPitch.get());
    }
    let bl: bool = this.isInWater();
    let bl2: bool = this.handleLavaMovement();
    if bl || bl2
    {
        living.m_isJumping.set(this.m_rand.nextFloat() < 0.8);
    }
}

pub fn faceEntity(this: &Rc<Entity>, target: &Rc<Entity>, maxPitch: f32, maxYaw: f32)
{
    let d0: f64 = target.posX() - this.posX();
    let d2: f64 = target.posZ() - this.posZ();
    let d1: f64 = if target.instanceOf("EntityLiving")
    {
        this.posY() + this.getEyeHeight() as f64 - (target.posY() + target.getEyeHeight() as f64)
    }
    else
    {
        (target.m_boundingBox.minY() + target.m_boundingBox.maxY()) / 2.0 - (this.posY() + this.getEyeHeight() as f64)
    };
    let d3: f64 = math_helper::sqrt_double(d0 * d0 + d2 * d2) as f64;
    let f: f32 = (math::atan2(d2, d0) * 180.0 / PI_F as f64) as f32 - 90.0;
    let f1: f32 = (-(math::atan2(d1, d3) * 180.0 / PI_F as f64)) as f32;
    this.m_rotationPitch.set(-updateRotation(this.m_rotationPitch.get(), f1, maxYaw));
    this.m_rotationYaw.set(updateRotation(this.m_rotationYaw.get(), f, maxPitch));
}

pub fn hasCurrentTarget(this: &Entity) -> bool { fields(this).m_currentTarget.with(Option::is_some) }

pub fn getCurrentTarget(this: &Entity) -> Option<Rc<Entity>> { fields(this).m_currentTarget.get() }

fn updateRotation(from: f32, to: f32, maxChange: f32) -> f32
{
    let mut f: f32 = to - from;
    while f < -180.0
    {
        f += 360.0;
    }
    while f >= 180.0
    {
        f -= 360.0;
    }
    if f > maxChange
    {
        f = maxChange;
    }
    if f < -maxChange
    {
        f = -maxChange;
    }
    from + f
}

pub fn getPosition(this: &Entity, tickDelta: f32) -> Rc<Vec3D>
{
    if tickDelta == 1.0
    {
        return Vec3D::createVector(this.posX(), this.posY(), this.posZ());
    }
    let d0: f64 = this.m_prevPosX.get() + (this.posX() - this.m_prevPosX.get()) * tickDelta as f64;
    let d1: f64 = this.m_prevPosY.get() + (this.posY() - this.m_prevPosY.get()) * tickDelta as f64;
    let d2: f64 = this.m_prevPosZ.get() + (this.posZ() - this.m_prevPosZ.get()) * tickDelta as f64;
    Vec3D::createVector(d0, d1, d2)
}

pub fn getLook(this: &Entity, tickDelta: f32) -> Rc<Vec3D>
{
    if tickDelta == 1.0
    {
        let f6: f32 = math_helper::cos(-this.m_rotationYaw.get() * DEG_TO_RAD - PI_F);
        let f7: f32 = math_helper::sin(-this.m_rotationYaw.get() * DEG_TO_RAD - PI_F);
        let f8: f32 = -math_helper::cos(-this.m_rotationPitch.get() * DEG_TO_RAD);
        let f9: f32 = math_helper::sin(-this.m_rotationPitch.get() * DEG_TO_RAD);
        return Vec3D::createVector((f7 * f8) as f64, f9 as f64, (f6 * f8) as f64);
    }
    let f: f32 =
        this.m_prevRotationPitch.get() + (this.m_rotationPitch.get() - this.m_prevRotationPitch.get()) * tickDelta;
    let f1: f32 = this.m_prevRotationYaw.get() + (this.m_rotationYaw.get() - this.m_prevRotationYaw.get()) * tickDelta;
    let f2: f32 = math_helper::cos(-f1 * DEG_TO_RAD - PI_F);
    let f3: f32 = math_helper::sin(-f1 * DEG_TO_RAD - PI_F);
    let f4: f32 = -math_helper::cos(-f * DEG_TO_RAD);
    let f5: f32 = math_helper::sin(-f * DEG_TO_RAD);
    Vec3D::createVector((f3 * f4) as f64, f5 as f64, (f2 * f4) as f64)
}

pub fn rayTrace(this: &Rc<Entity>, distance: f64, tickDelta: f32) -> Option<MovingObjectPosition>
{
    let vec3d: Rc<Vec3D> = getPosition(this, tickDelta);
    let vec3d1: Rc<Vec3D> = getLook(this, tickDelta);
    let vec3d2: Rc<Vec3D> = vec3d.addVector(vec3d1.x() * distance, vec3d1.y() * distance, vec3d1.z() * distance);
    this.world().rayTraceBlocks(&vec3d, &vec3d2)
}
