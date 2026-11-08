use crate::block::{Block, Material};
use crate::entity::{ENTITY, Entity, EntityClass, EntityExt};
use crate::item::Item;
use crate::math::AxisAlignedBB;
use crate::math::math_helper::floor_double;
use crate::world::World;
use std::cell::Cell;
use std::f64::consts::PI;
use std::rc::Rc;

pub struct LerpFields
{
    m_lerpSteps: Cell<i32>,
    m_lerpX: Cell<f64>,
    m_lerpY: Cell<f64>,
    m_lerpZ: Cell<f64>,
    m_lerpYaw: Cell<f64>,
    m_lerpPitch: Cell<f64>,
    m_lerpVelocityX: Cell<f64>,
    m_lerpVelocityY: Cell<f64>,
    m_lerpVelocityZ: Cell<f64>,
}

impl LerpFields
{
    pub(crate) fn new() -> LerpFields
    {
        LerpFields {
            m_lerpSteps: Cell::new(0),
            m_lerpX: Cell::new(0.0),
            m_lerpY: Cell::new(0.0),
            m_lerpZ: Cell::new(0.0),
            m_lerpYaw: Cell::new(0.0),
            m_lerpPitch: Cell::new(0.0),
            m_lerpVelocityX: Cell::new(0.0),
            m_lerpVelocityY: Cell::new(0.0),
            m_lerpVelocityZ: Cell::new(0.0),
        }
    }

    pub(crate) fn steps(&self) -> i32 { self.m_lerpSteps.get() }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn setTarget(&self, this: &Entity, x: f64, y: f64, z: f64, yaw: f32, pitch: f32, steps: i32)
    {
        self.m_lerpX.set(x);
        self.m_lerpY.set(y);
        self.m_lerpZ.set(z);
        self.m_lerpYaw.set(yaw as f64);
        self.m_lerpPitch.set(pitch as f64);
        self.m_lerpSteps.set(steps);
        this.m_motionX.set(self.m_lerpVelocityX.get());
        this.m_motionY.set(self.m_lerpVelocityY.get());
        this.m_motionZ.set(self.m_lerpVelocityZ.get());
    }

    pub(crate) fn setVelocity(&self, this: &Entity, x: f64, y: f64, z: f64)
    {
        this.m_motionX.set(x);
        self.m_lerpVelocityX.set(x);
        this.m_motionY.set(y);
        self.m_lerpVelocityY.set(y);
        this.m_motionZ.set(z);
        self.m_lerpVelocityZ.set(z);
    }

    pub(crate) fn step(&self, this: &Entity)
    {
        let steps: f64 = self.m_lerpSteps.get() as f64;
        let d8: f64 = this.posX() + (self.m_lerpX.get() - this.posX()) / steps;
        let d9: f64 = this.posY() + (self.m_lerpY.get() - this.posY()) / steps;
        let d10: f64 = this.posZ() + (self.m_lerpZ.get() - this.posZ()) / steps;
        let mut d7: f64 = self.m_lerpYaw.get() - this.m_rotationYaw.get() as f64;
        while d7 < -180.0
        {
            d7 += 360.0;
        }
        while d7 >= 180.0
        {
            d7 -= 360.0;
        }
        this.m_rotationYaw.set((this.m_rotationYaw.get() as f64 + d7 / steps) as f32);
        this.m_rotationPitch.set(
            (this.m_rotationPitch.get() as f64 + (self.m_lerpPitch.get() - this.m_rotationPitch.get() as f64) / steps)
                as f32,
        );
        self.m_lerpSteps.set(self.m_lerpSteps.get() - 1);
        this.setPosition(d8, d9, d10);
        this.setRotation(this.m_rotationYaw.get(), this.m_rotationPitch.get());
    }
}

pub struct BoatFields
{
    pub m_boatCurrentDamage: Cell<i32>,
    pub m_boatTimeSinceHit: Cell<i32>,
    pub m_boatRockDirection: Cell<i32>,
    m_lerp: LerpFields,
}

impl EntityExt for BoatFields
{
    fn asAny(&self) -> &dyn std::any::Any { self }
}

pub fn fields(this: &Entity) -> &BoatFields { this.ext::<BoatFields>() }

pub fn new(world: Option<Rc<World>>) -> Rc<Entity>
{
    let entity: Rc<Entity> = Entity::alloc(
        &CLASS,
        Box::new(BoatFields {
            m_boatCurrentDamage: Cell::new(0),
            m_boatTimeSinceHit: Cell::new(0),
            m_boatRockDirection: Cell::new(0),
            m_lerp: LerpFields::new(),
        }),
    );
    Entity::init(&entity, world);
    fields(&entity).m_boatRockDirection.set(1);
    entity.m_preventEntitySpawning.set(true);
    entity.setSize(1.5, 0.6);
    entity.m_yOffset.set(entity.m_height.get() / 2.0);
    entity
}

pub fn createAt(world: &Rc<World>, x: f64, y: f64, z: f64) -> Rc<Entity>
{
    let entity: Rc<Entity> = new(Some(Rc::clone(world)));
    entity.setPosition(x, y + entity.m_yOffset.get() as f64, z);
    entity.m_motionX.set(0.0);
    entity.m_motionY.set(0.0);
    entity.m_motionZ.set(0.0);
    entity.m_prevPosX.set(x);
    entity.m_prevPosY.set(y);
    entity.m_prevPosZ.set(z);
    entity
}

fn dropParts(this: &Rc<Entity>)
{
    for _ in 0..3
    {
        this.dropItemWithOffset(Block::planks().m_blockID, 1, 0.0);
    }
    for _ in 0..2
    {
        this.dropItemWithOffset(Item::stick().m_shiftedIndex, 1, 0.0);
    }
}

fn waterDepth(this: &Entity, world: &World) -> f64
{
    let n: i32 = 5;
    let mut d4: f64 = 0.0;
    let bounds: &AxisAlignedBB = &this.m_boundingBox;
    for i in 0..n
    {
        let d5: f64 = bounds.minY() + (bounds.maxY() - bounds.minY()) * (i + 0) as f64 / n as f64 - 0.125;
        let d6: f64 = bounds.minY() + (bounds.maxY() - bounds.minY()) * (i + 1) as f64 / n as f64 - 0.125;
        let axisAlignedBB: Rc<AxisAlignedBB> =
            AxisAlignedBB::getBoundingBoxFromPool(bounds.minX(), d5, bounds.minZ(), bounds.maxX(), d6, bounds.maxZ());
        if world.isAABBInMaterial(&axisAlignedBB, Material::water())
        {
            d4 += 1.0 / n as f64;
        }
    }
    d4
}

fn clampMotion(this: &Entity, limit: f64)
{
    if this.m_motionX.get() < -limit
    {
        this.m_motionX.set(-limit);
    }
    if this.m_motionX.get() > limit
    {
        this.m_motionX.set(limit);
    }
    if this.m_motionZ.get() < -limit
    {
        this.m_motionZ.set(-limit);
    }
    if this.m_motionZ.get() > limit
    {
        this.m_motionZ.set(limit);
    }
}

fn spawnWake(this: &Rc<Entity>, world: &World, d15: f64)
{
    let d16: f64 = jvm::math::cos(this.m_rotationYaw.get() as f64 * PI / 180.0);
    let d2: f64 = jvm::math::sin(this.m_rotationYaw.get() as f64 * PI / 180.0);
    let mut n2: i32 = 0;
    while (n2 as f64) < 1.0 + d15 * 60.0
    {
        let d19: f64 = (this.m_rand.nextFloat() * 2.0 - 1.0) as f64;
        let d20: f64 = (this.m_rand.nextIntBounded(2) * 2 - 1) as f64 * 0.7;
        let (d18, d17): (f64, f64) = if this.m_rand.nextBoolean()
        {
            (this.posX() - d16 * d19 * 0.8 + d2 * d20, this.posZ() - d2 * d19 * 0.8 - d16 * d20)
        }
        else
        {
            (this.posX() + d16 + d2 * d19 * 0.7, this.posZ() + d2 - d16 * d19 * 0.7)
        };
        world.spawnParticle(
            "splash",
            d18,
            this.posY() - 0.125,
            d17,
            this.m_motionX.get(),
            this.m_motionY.get(),
            this.m_motionZ.get(),
        );
        n2 += 1;
    }
}

fn isRider(this: &Entity, entity: &Rc<Entity>) -> bool
{
    this.m_riddenByEntity.get().is_some_and(|rider| Rc::ptr_eq(&rider, entity))
}

pub(crate) fn pushNearby(this: &Rc<Entity>, world: &World, className: &str)
{
    let list: Rc<jvm::array_list::JArrayList<Rc<Entity>>> = world.getEntitiesWithinAABBExcludingEntity(
        Some(this),
        &this.m_boundingBox.expand(0.2f32 as f64, 0.0, 0.2f32 as f64),
    );
    if list.size() <= 0
    {
        return;
    }
    let mut i: i32 = 0;
    while i < list.size()
    {
        let entity: Rc<Entity> = list.get(i);
        i += 1;
        if isRider(this, &entity) || !entity.canBePushed() || !entity.instanceOf(className)
        {
            continue;
        }
        entity.applyEntityCollision(this);
    }
}

pub(crate) fn clearDeadRider(this: &Entity)
{
    if this.m_riddenByEntity.get().is_some_and(|rider| rider.m_isDead.get())
    {
        this.m_riddenByEntity.set(None);
    }
}

fn onUpdate(this: &Rc<Entity>)
{
    (ENTITY.onUpdate)(this);
    let boat: &BoatFields = fields(this);
    if boat.m_boatTimeSinceHit.get() > 0
    {
        boat.m_boatTimeSinceHit.set(boat.m_boatTimeSinceHit.get() - 1);
    }
    if boat.m_boatCurrentDamage.get() > 0
    {
        boat.m_boatCurrentDamage.set(boat.m_boatCurrentDamage.get() - 1);
    }
    this.m_prevPosX.set(this.posX());
    this.m_prevPosY.set(this.posY());
    this.m_prevPosZ.set(this.posZ());
    let world: Rc<World> = this.world();
    let d4: f64 = waterDepth(this, &world);
    if world.m_multiplayerWorld.get()
    {
        if boat.m_lerp.steps() > 0
        {
            boat.m_lerp.step(this);
        }
        else
        {
            let d11: f64 = this.posX() + this.m_motionX.get();
            let d12: f64 = this.posY() + this.m_motionY.get();
            let d13: f64 = this.posZ() + this.m_motionZ.get();
            this.setPosition(d11, d12, d13);
            if this.m_onGround.get()
            {
                scale(this, 0.5, 0.5, 0.5);
            }
            scale(this, 0.99f32 as f64, 0.95f32 as f64, 0.99f32 as f64);
        }
        return;
    }

    if d4 < 1.0
    {
        let d14: f64 = d4 * 2.0 - 1.0;
        this.m_motionY.set(this.m_motionY.get() + 0.04f32 as f64 * d14);
    }
    else
    {
        if this.m_motionY.get() < 0.0
        {
            this.m_motionY.set(this.m_motionY.get() / 2.0);
        }
        this.m_motionY.set(this.m_motionY.get() + 0.007f32 as f64);
    }
    if let Some(rider) = this.m_riddenByEntity.get()
    {
        this.m_motionX.set(this.m_motionX.get() + rider.m_motionX.get() * 0.2);
        this.m_motionZ.set(this.m_motionZ.get() + rider.m_motionZ.get() * 0.2);
    }
    clampMotion(this, 0.4);
    if this.m_onGround.get()
    {
        scale(this, 0.5, 0.5, 0.5);
    }
    this.moveEntity(this.m_motionX.get(), this.m_motionY.get(), this.m_motionZ.get());
    let d15: f64 =
        jvm::math::sqrt(this.m_motionX.get() * this.m_motionX.get() + this.m_motionZ.get() * this.m_motionZ.get());
    if d15 > 0.15
    {
        spawnWake(this, &world, d15);
    }
    if this.m_isCollidedHorizontally.get() && d15 > 0.15
    {
        if !world.m_multiplayerWorld.get()
        {
            this.setEntityDead();
            dropParts(this);
        }
    }
    else
    {
        scale(this, 0.99f32 as f64, 0.95f32 as f64, 0.99f32 as f64);
    }

    this.m_rotationPitch.set(0.0);
    let mut d21: f64 = this.m_rotationYaw.get() as f64;
    let d2: f64 = this.m_prevPosX.get() - this.posX();
    let d22: f64 = this.m_prevPosZ.get() - this.posZ();
    if d2 * d2 + d22 * d22 > 0.001
    {
        d21 = (jvm::math::atan2(d22, d2) * 180.0 / PI) as f32 as f64;
    }
    let mut d: f64 = d21 - this.m_rotationYaw.get() as f64;
    while d >= 180.0
    {
        d -= 360.0;
    }
    while d < -180.0
    {
        d += 360.0;
    }
    d = d.clamp(-20.0, 20.0);
    this.m_rotationYaw.set((this.m_rotationYaw.get() as f64 + d) as f32);
    this.setRotation(this.m_rotationYaw.get(), this.m_rotationPitch.get());
    pushNearby(this, &world, "EntityBoat");
    for i in 0..4
    {
        let n6: i32 = floor_double(this.posX() + ((i % 2) as f64 - 0.5) * 0.8);
        let n5: i32 = floor_double(this.posY());
        let n4: i32 = floor_double(this.posZ() + ((i / 2) as f64 - 0.5) * 0.8);
        if world.getBlockId(n6, n5, n4) == Block::snow().m_blockID
        {
            world.setBlockWithNotify(n6, n5, n4, 0);
        }
    }
    clearDeadRider(this);
}

pub(crate) fn scale(this: &Entity, x: f64, y: f64, z: f64)
{
    this.m_motionX.set(this.m_motionX.get() * x);
    this.m_motionY.set(this.m_motionY.get() * y);
    this.m_motionZ.set(this.m_motionZ.get() * z);
}

pub(crate) fn mountBy(this: &Rc<Entity>, player: &Rc<Entity>) -> bool
{
    if let Some(rider) = this.m_riddenByEntity.get()
        && rider.instanceOf("EntityPlayer")
        && !Rc::ptr_eq(&rider, player)
    {
        return true;
    }
    if !this.world().m_multiplayerWorld.get()
    {
        player.mountEntity(Some(this));
    }
    true
}

pub const CLASS: EntityClass = EntityClass {
    m_name: "EntityBoat",
    m_parent: Some(&ENTITY),
    m_interfaces: &[],
    canTriggerWalking: |_| false,
    entityInit: |_| {},
    getCollisionBox: |_, other| Some(Rc::clone(&other.m_boundingBox)),
    getBoundingBox: |this| Some(Rc::clone(&this.m_boundingBox)),
    canBePushed: |_| true,
    getMountedYOffset: |this| this.m_height.get() as f64 * 0.0 - 0.3f32 as f64,
    attackEntityFrom: |this, _source, amount| {
        if this.world().m_multiplayerWorld.get() || this.m_isDead.get()
        {
            return true;
        }
        let boat: &BoatFields = fields(this);
        boat.m_boatRockDirection.set(-boat.m_boatRockDirection.get());
        boat.m_boatTimeSinceHit.set(10);
        boat.m_boatCurrentDamage.set(boat.m_boatCurrentDamage.get() + amount * 10);
        this.setBeenAttacked();
        if boat.m_boatCurrentDamage.get() > 40
        {
            if let Some(rider) = this.m_riddenByEntity.get()
            {
                rider.mountEntity(Some(this));
            }
            dropParts(this);
            this.setEntityDead();
        }
        true
    },
    performHurtAnimation: |this| {
        let boat: &BoatFields = fields(this);
        boat.m_boatRockDirection.set(-boat.m_boatRockDirection.get());
        boat.m_boatTimeSinceHit.set(10);
        boat.m_boatCurrentDamage.set(boat.m_boatCurrentDamage.get() + boat.m_boatCurrentDamage.get() * 10);
    },
    canBeCollidedWith: |this| !this.m_isDead.get(),
    setPositionAndRotation2: |this, x, y, z, yaw, pitch, steps| {
        fields(this).m_lerp.setTarget(this, x, y, z, yaw, pitch, steps + 4)
    },
    setVelocity: |this, x, y, z| fields(this).m_lerp.setVelocity(this, x, y, z),
    onUpdate,
    updateRiderPosition: |this| {
        let Some(rider) = this.m_riddenByEntity.get()
        else
        {
            return;
        };
        let d: f64 = jvm::math::cos(this.m_rotationYaw.get() as f64 * PI / 180.0) * 0.4;
        let d2: f64 = jvm::math::sin(this.m_rotationYaw.get() as f64 * PI / 180.0) * 0.4;
        rider.setPosition(
            this.posX() + d,
            this.posY() + this.getMountedYOffset() + rider.getYOffset(),
            this.posZ() + d2,
        );
    },
    writeEntityToNBT: |_, _| {},
    readEntityFromNBT: |_, _| {},
    getShadowSize: |_| 0.0,
    interact: mountBy,
    ..ENTITY
};
