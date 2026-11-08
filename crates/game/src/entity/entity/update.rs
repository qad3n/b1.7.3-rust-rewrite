use crate::block::Material;
use crate::entity::Entity;
use crate::math::math_helper::{self, floor_double};
use crate::math::{AxisAlignedBB, Vec3D};
use crate::port::sameObject;
use crate::world::World;
use jvm::array_list::JArrayList;
use std::rc::Rc;

pub fn onEntityUpdate(this: &Rc<Entity>)
{
    if let Some(riding) = this.m_ridingEntity.get()
        && riding.m_isDead.get()
    {
        this.m_ridingEntity.set(None);
    }

    this.m_ticksExisted.set(this.m_ticksExisted.get() + 1);
    this.m_prevDistanceWalkedModified.set(this.m_distanceWalkedModified.get());
    this.m_prevPosX.set(this.posX());
    this.m_prevPosY.set(this.posY());
    this.m_prevPosZ.set(this.posZ());
    this.m_prevRotationPitch.set(this.m_rotationPitch.get());
    this.m_prevRotationYaw.set(this.m_rotationYaw.get());
    let world: Rc<World> = this.world();
    if this.handleWaterMovement()
    {
        if !this.m_inWater.get() && !this.m_isFirstUpdate.get()
        {
            let motionX: f64 = this.m_motionX.get();
            let motionY: f64 = this.m_motionY.get();
            let motionZ: f64 = this.m_motionZ.get();
            let mut f: f32 = math_helper::sqrt_double(
                motionX * motionX * 0.2f32 as f64 + motionY * motionY + motionZ * motionZ * 0.2f32 as f64,
            ) * 0.2;
            if f > 1.0
            {
                f = 1.0;
            }

            let pitch: f32 = 1.0 + (this.m_rand.nextFloat() - this.m_rand.nextFloat()) * 0.4;
            world.playSoundAtEntity(this, "random.splash", f, pitch);
            let f1: f32 = floor_double(this.m_boundingBox.minY()) as f32;

            let mut i: i32 = 0;
            while (i as f32) < 1.0 + this.m_width.get() * 20.0
            {
                let f2: f32 = (this.m_rand.nextFloat() * 2.0 - 1.0) * this.m_width.get();
                let f3: f32 = (this.m_rand.nextFloat() * 2.0 - 1.0) * this.m_width.get();
                let velocityY: f64 = this.m_motionY.get() - (this.m_rand.nextFloat() * 0.2) as f64;
                world.spawnParticle(
                    "bubble",
                    this.posX() + f2 as f64,
                    (f1 + 1.0) as f64,
                    this.posZ() + f3 as f64,
                    this.m_motionX.get(),
                    velocityY,
                    this.m_motionZ.get(),
                );
                i += 1;
            }

            let mut j: i32 = 0;
            while (j as f32) < 1.0 + this.m_width.get() * 20.0
            {
                let f4: f32 = (this.m_rand.nextFloat() * 2.0 - 1.0) * this.m_width.get();
                let f5: f32 = (this.m_rand.nextFloat() * 2.0 - 1.0) * this.m_width.get();
                world.spawnParticle(
                    "splash",
                    this.posX() + f4 as f64,
                    (f1 + 1.0) as f64,
                    this.posZ() + f5 as f64,
                    this.m_motionX.get(),
                    this.m_motionY.get(),
                    this.m_motionZ.get(),
                );
                j += 1;
            }
        }

        this.m_fallDistance.set(0.0);
        this.m_inWater.set(true);
        this.m_fire.set(0);
    }
    else
    {
        this.m_inWater.set(false);
    }

    if world.m_multiplayerWorld.get()
    {
        this.m_fire.set(0);
    }
    else if this.m_fire.get() > 0
    {
        if this.m_isImmuneToFire.get()
        {
            this.m_fire.set(this.m_fire.get() - 4);
            if this.m_fire.get() < 0
            {
                this.m_fire.set(0);
            }
        }
        else
        {
            if this.m_fire.get() % 20 == 0
            {
                this.attackEntityFrom(None, 1);
            }
            this.m_fire.set(this.m_fire.get() - 1);
        }
    }

    if this.handleLavaMovement()
    {
        setOnFireFromLava(this);
    }

    if this.posY() < -64.0
    {
        this.kill();
    }

    if !world.m_multiplayerWorld.get()
    {
        this.setEntityFlag(0, this.m_fire.get() > 0);
        this.setEntityFlag(2, this.m_ridingEntity.with(Option::is_some));
    }

    this.m_isFirstUpdate.set(false);
}

pub fn setOnFireFromLava(this: &Rc<Entity>)
{
    if !this.m_isImmuneToFire.get()
    {
        this.attackEntityFrom(None, 4);
        this.m_fire.set(600);
    }
}

pub fn addVelocity(this: &Rc<Entity>, dx: f64, dy: f64, dz: f64)
{
    this.m_motionX.set(this.m_motionX.get() + dx);
    this.m_motionY.set(this.m_motionY.get() + dy);
    this.m_motionZ.set(this.m_motionZ.get() + dz);
}

pub fn applyEntityCollision(this: &Rc<Entity>, entity: &Rc<Entity>)
{
    // java compares references here and either side may be null
    if sameObject(entity.m_riddenByEntity.get().as_ref(), Some(this))
        || sameObject(entity.m_ridingEntity.get().as_ref(), Some(this))
    {
        return;
    }

    let a: f64 = entity.posX() - this.posX();
    let b: f64 = entity.posZ() - this.posZ();
    let absMax: f64 = math_helper::abs_max(a, b);
    if absMax >= 0.009999999776482582
    {
        let n: f64 = math_helper::sqrt_double(absMax) as f64;
        let n2: f64 = a / n;
        let n3: f64 = b / n;
        let mut n4: f64 = 1.0 / n;
        if n4 > 1.0
        {
            n4 = 1.0;
        }
        let n5: f64 = n2 * n4;
        let n6: f64 = n3 * n4;
        let n7: f64 = n5 * 0.05000000074505806;
        let n8: f64 = n6 * 0.05000000074505806;
        let dx: f64 = n7 * (1.0 - this.m_entityCollisionReduction.get()) as f64;
        let dz: f64 = n8 * (1.0 - this.m_entityCollisionReduction.get()) as f64;
        this.addVelocity(-dx, 0.0, -dz);
        entity.addVelocity(dx, 0.0, dz);
    }
}

pub fn attackEntityFrom(this: &Rc<Entity>, _source: Option<&Rc<Entity>>, _amount: i32) -> bool
{
    this.setBeenAttacked();
    false
}

pub fn dealFireDamage(this: &Rc<Entity>, amount: i32)
{
    if !this.m_isImmuneToFire.get()
    {
        this.attackEntityFrom(None, amount);
    }
}

pub fn fall(this: &Rc<Entity>, distance: f32)
{
    if let Some(rider) = this.m_riddenByEntity.get()
    {
        rider.fall(distance);
    }
}

pub fn handleWaterMovement(this: &Rc<Entity>) -> bool
{
    let bb: Rc<AxisAlignedBB> = this.m_boundingBox.expand(0.0, -0.4f32 as f64, 0.0).contract(0.001, 0.001, 0.001);
    this.world().handleMaterialAcceleration(&bb, Material::water(), this)
}

pub fn getEntityBrightness(this: &Rc<Entity>, _tickDelta: f32) -> f32
{
    let i: i32 = floor_double(this.posX());
    let d0: f64 = (this.m_boundingBox.maxY() - this.m_boundingBox.minY()) * 0.66;
    let j: i32 = floor_double(this.posY() - this.m_yOffset.get() as f64 + d0);
    let k: i32 = floor_double(this.posZ());
    let world: Rc<World> = this.world();
    let bb: &AxisAlignedBB = &this.m_boundingBox;
    if world.checkChunksExist(
        floor_double(bb.minX()),
        floor_double(bb.minY()),
        floor_double(bb.minZ()),
        floor_double(bb.maxX()),
        floor_double(bb.maxY()),
        floor_double(bb.maxZ()),
    )
    {
        let mut f: f32 = world.getLightBrightness(i, j, k);
        if f < this.m_entityBrightness.get()
        {
            f = this.m_entityBrightness.get();
        }
        return f;
    }
    this.m_entityBrightness.get()
}

pub fn isEntityInsideOpaqueBlock(this: &Rc<Entity>) -> bool
{
    let world: Rc<World> = this.world();
    for i in 0..8
    {
        let f: f32 = (((i >> 0) % 2) as f32 - 0.5) * this.m_width.get() * 0.9;
        let f1: f32 = (((i >> 1) % 2) as f32 - 0.5) * 0.1;
        let f2: f32 = (((i >> 2) % 2) as f32 - 0.5) * this.m_width.get() * 0.9;
        let j: i32 = floor_double(this.posX() + f as f64);
        let k: i32 = floor_double(this.posY() + this.getEyeHeight() as f64 + f1 as f64);
        let l: i32 = floor_double(this.posZ() + f2 as f64);
        if world.isBlockNormalCube(j, k, l)
        {
            return true;
        }
    }
    false
}

pub fn isInRangeToRenderVec3D(this: &Rc<Entity>, cameraPos: &Vec3D) -> bool
{
    let d0: f64 = this.posX() - cameraPos.x();
    let d1: f64 = this.posY() - cameraPos.y();
    let d2: f64 = this.posZ() - cameraPos.z();
    let d3: f64 = d0 * d0 + d1 * d1 + d2 * d2;
    this.isInRangeToRenderDist(d3)
}

pub fn isInRangeToRenderDist(this: &Rc<Entity>, squaredDistanceToCamera: f64) -> bool
{
    let mut d0: f64 = this.m_boundingBox.getAverageEdgeLength();
    d0 *= 64.0 * this.m_renderDistanceWeight.get();
    squaredDistanceToCamera < d0 * d0
}

pub fn onStruckByLightning(this: &Rc<Entity>, _lightning: &Rc<Entity>)
{
    this.dealFireDamage(5);
    this.m_fire.set(this.m_fire.get() + 1);
    if this.m_fire.get() == 0
    {
        this.m_fire.set(300);
    }
}

pub fn preparePlayerToSpawn(this: &Rc<Entity>)
{
    let Some(world) = this.m_worldObj.get()
    else
    {
        return;
    };

    while this.posY() > 0.0
    {
        this.setPosition(this.posX(), this.posY(), this.posZ());
        if world.getCollidingBoundingBoxes(this, &this.m_boundingBox).isEmpty()
        {
            break;
        }
        this.m_posY.set(this.posY() + 1.0);
    }

    this.m_motionZ.set(0.0);
    this.m_motionY.set(0.0);
    this.m_motionX.set(0.0);
    this.m_rotationPitch.set(0.0);
}

impl Entity
{
    pub fn isOffsetPositionInLiquid(self: &Rc<Self>, dx: f64, dy: f64, dz: f64) -> bool
    {
        let axisalignedbb: Rc<AxisAlignedBB> = self.m_boundingBox.getOffsetBoundingBox(dx, dy, dz);
        let world: Rc<World> = self.world();
        let list: Rc<JArrayList<Rc<AxisAlignedBB>>> = world.getCollidingBoundingBoxes(self, &axisalignedbb);
        if !list.isEmpty()
        {
            false
        }
        else
        {
            !world.getIsAnyLiquid(&axisalignedbb)
        }
    }
}
