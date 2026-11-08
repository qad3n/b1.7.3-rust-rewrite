mod class;
mod movement;
mod nbt;
mod riding;
mod update;

use crate::entity::DataWatcher;
use crate::math::AxisAlignedBB;
use crate::world::World;
pub use class::{ENTITY, EntityClass};
use jvm::field::Field;
use jvm::object::JavaObject;
use jvm::random::Random;
use jvm::string::JString;
use jvm::throwable::OrNpe;
use std::any::Any;
use std::cell::Cell;
use std::rc::Rc;

thread_local! {
    static NEXT_ENTITY_ID: Cell<i32> = const { Cell::new(0) };
}

pub trait EntityExt: Any
{
    fn asAny(&self) -> &dyn Any;

    fn living(&self) -> Option<&crate::entity::entity_living::LivingFields> { None }

    fn creature(&self) -> Option<&crate::entity::entity_creature::CreatureFields> { None }

    fn mob(&self) -> Option<&crate::entity::entity_mob::MobFields> { None }

    fn player(&self) -> Option<&crate::entity::entity_player::PlayerFields> { None }
}

pub struct NoExt;

impl EntityExt for NoExt
{
    fn asAny(&self) -> &dyn Any { self }
}

pub struct Entity
{
    pub m_class: &'static EntityClass,
    pub m_ext: Box<dyn EntityExt>,
    pub m_entityId: Cell<i32>,
    pub m_renderDistanceWeight: Cell<f64>,
    pub m_preventEntitySpawning: Cell<bool>,
    pub m_riddenByEntity: Field<Option<Rc<Entity>>>,
    pub m_ridingEntity: Field<Option<Rc<Entity>>>,
    pub m_worldObj: Field<Option<Rc<World>>>,
    pub m_prevPosX: Cell<f64>,
    pub m_prevPosY: Cell<f64>,
    pub m_prevPosZ: Cell<f64>,
    pub m_posX: Cell<f64>,
    pub m_posY: Cell<f64>,
    pub m_posZ: Cell<f64>,
    pub m_motionX: Cell<f64>,
    pub m_motionY: Cell<f64>,
    pub m_motionZ: Cell<f64>,
    pub m_rotationYaw: Cell<f32>,
    pub m_rotationPitch: Cell<f32>,
    pub m_prevRotationYaw: Cell<f32>,
    pub m_prevRotationPitch: Cell<f32>,
    pub m_boundingBox: Rc<AxisAlignedBB>,
    pub m_onGround: Cell<bool>,
    pub m_isCollidedHorizontally: Cell<bool>,
    pub m_isCollidedVertically: Cell<bool>,
    pub m_isCollided: Cell<bool>,
    pub m_beenAttacked: Cell<bool>,
    pub m_isInWeb: Cell<bool>,
    pub m_immovable: Cell<bool>,
    pub m_isDead: Cell<bool>,
    pub m_yOffset: Cell<f32>,
    pub m_width: Cell<f32>,
    pub m_height: Cell<f32>,
    pub m_prevDistanceWalkedModified: Cell<f32>,
    pub m_distanceWalkedModified: Cell<f32>,
    pub m_fallDistance: Cell<f32>,
    pub m_nextStepDistance: Cell<i32>,
    pub m_lastTickPosX: Cell<f64>,
    pub m_lastTickPosY: Cell<f64>,
    pub m_lastTickPosZ: Cell<f64>,
    pub m_ySize: Cell<f32>,
    pub m_stepHeight: Cell<f32>,
    pub m_noClip: Cell<bool>,
    pub m_entityCollisionReduction: Cell<f32>,
    pub m_rand: Random,
    pub m_ticksExisted: Cell<i32>,
    pub m_fireResistance: Cell<i32>,
    pub m_fire: Cell<i32>,
    pub m_maxAir: Cell<i32>,
    pub m_inWater: Cell<bool>,
    pub m_heartsLife: Cell<i32>,
    pub m_air: Cell<i32>,
    pub m_isFirstUpdate: Cell<bool>,
    pub m_skinUrl: Field<Option<JString>>,
    pub m_cloakUrl: Field<Option<JString>>,
    pub m_isImmuneToFire: Cell<bool>,
    pub m_dataWatcher: DataWatcher,
    pub m_entityBrightness: Cell<f32>,
    pub m_entityRiderPitchDelta: Cell<f64>,
    pub m_entityRiderYawDelta: Cell<f64>,
    pub m_addedToChunk: Cell<bool>,
    pub m_chunkCoordX: Cell<i32>,
    pub m_chunkCoordY: Cell<i32>,
    pub m_chunkCoordZ: Cell<i32>,
    pub m_serverPosX: Cell<i32>,
    pub m_serverPosY: Cell<i32>,
    pub m_serverPosZ: Cell<i32>,
    pub m_ignoreFrustumCheck: Cell<bool>,
}

impl Entity
{
    pub fn alloc(class: &'static EntityClass, ext: Box<dyn EntityExt>) -> Rc<Self>
    {
        let entityId: i32 = NEXT_ENTITY_ID.with(|next| {
            let id: i32 = next.get();
            next.set(id + 1);
            id
        });
        Rc::new(Self {
            m_class: class,
            m_ext: ext,
            m_entityId: Cell::new(entityId),
            m_renderDistanceWeight: Cell::new(1.0),
            m_preventEntitySpawning: Cell::new(false),
            m_riddenByEntity: Field::new(None),
            m_ridingEntity: Field::new(None),
            m_worldObj: Field::new(None),
            m_prevPosX: Cell::new(0.0),
            m_prevPosY: Cell::new(0.0),
            m_prevPosZ: Cell::new(0.0),
            m_posX: Cell::new(0.0),
            m_posY: Cell::new(0.0),
            m_posZ: Cell::new(0.0),
            m_motionX: Cell::new(0.0),
            m_motionY: Cell::new(0.0),
            m_motionZ: Cell::new(0.0),
            m_rotationYaw: Cell::new(0.0),
            m_rotationPitch: Cell::new(0.0),
            m_prevRotationYaw: Cell::new(0.0),
            m_prevRotationPitch: Cell::new(0.0),
            m_boundingBox: AxisAlignedBB::getBoundingBox(0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
            m_onGround: Cell::new(false),
            m_isCollidedHorizontally: Cell::new(false),
            m_isCollidedVertically: Cell::new(false),
            m_isCollided: Cell::new(false),
            m_beenAttacked: Cell::new(false),
            m_isInWeb: Cell::new(false),
            m_immovable: Cell::new(true),
            m_isDead: Cell::new(false),
            m_yOffset: Cell::new(0.0),
            m_width: Cell::new(0.6),
            m_height: Cell::new(1.8),
            m_prevDistanceWalkedModified: Cell::new(0.0),
            m_distanceWalkedModified: Cell::new(0.0),
            m_fallDistance: Cell::new(0.0),
            m_nextStepDistance: Cell::new(1),
            m_lastTickPosX: Cell::new(0.0),
            m_lastTickPosY: Cell::new(0.0),
            m_lastTickPosZ: Cell::new(0.0),
            m_ySize: Cell::new(0.0),
            m_stepHeight: Cell::new(0.0),
            m_noClip: Cell::new(false),
            m_entityCollisionReduction: Cell::new(0.0),
            m_rand: Random::new(),
            m_ticksExisted: Cell::new(0),
            m_fireResistance: Cell::new(1),
            m_fire: Cell::new(0),
            m_maxAir: Cell::new(300),
            m_inWater: Cell::new(false),
            m_heartsLife: Cell::new(0),
            m_air: Cell::new(300),
            m_isFirstUpdate: Cell::new(true),
            m_skinUrl: Field::new(None),
            m_cloakUrl: Field::new(None),
            m_isImmuneToFire: Cell::new(false),
            m_dataWatcher: DataWatcher::new(),
            m_entityBrightness: Cell::new(0.0),
            m_entityRiderPitchDelta: Cell::new(0.0),
            m_entityRiderYawDelta: Cell::new(0.0),
            m_addedToChunk: Cell::new(false),
            m_chunkCoordX: Cell::new(0),
            m_chunkCoordY: Cell::new(0),
            m_chunkCoordZ: Cell::new(0),
            m_serverPosX: Cell::new(0),
            m_serverPosY: Cell::new(0),
            m_serverPosZ: Cell::new(0),
            m_ignoreFrustumCheck: Cell::new(false),
        })
    }

    pub fn init(this: &Rc<Self>, world: Option<Rc<World>>)
    {
        this.m_worldObj.set(world);
        this.setPosition(0.0, 0.0, 0.0);
        this.m_dataWatcher.addObject(0, crate::entity::WatchedValue::Byte(0));
        this.entityInit();
    }

    pub fn ext<T: 'static>(&self) -> &T
    {
        self.m_ext.asAny().downcast_ref::<T>().expect("entity ext matches the class that allocated it")
    }

    pub fn world(&self) -> Rc<World> { self.m_worldObj.get().orNpe() }

    pub fn posX(&self) -> f64 { self.m_posX.get() }

    pub fn posY(&self) -> f64 { self.m_posY.get() }

    pub fn posZ(&self) -> f64 { self.m_posZ.get() }

    pub fn getDataWatcher(&self) -> &DataWatcher { &self.m_dataWatcher }

    pub fn equals(&self, other: &Entity) -> bool { other.m_entityId.get() == self.m_entityId.get() }

    pub fn hashCode(&self) -> i32 { self.m_entityId.get() }

    pub fn setRotation(&self, yaw: f32, pitch: f32)
    {
        self.m_rotationYaw.set(yaw % 360.0);
        self.m_rotationPitch.set(pitch % 360.0);
    }

    pub fn setPosition(&self, x: f64, y: f64, z: f64)
    {
        self.m_posX.set(x);
        self.m_posY.set(y);
        self.m_posZ.set(z);
        let f: f32 = self.m_width.get() / 2.0;
        let f1: f32 = self.m_height.get();
        let base: f64 = y - self.m_yOffset.get() as f64 + self.m_ySize.get() as f64;
        self.m_boundingBox.setBounds(x - f as f64, base, z - f as f64, x + f as f64, base + f1 as f64, z + f as f64);
    }

    pub fn updateLocalPlayerCamera(&self, yaw: f32, pitch: f32)
    {
        let f: f32 = self.m_rotationPitch.get();
        let f1: f32 = self.m_rotationYaw.get();
        self.m_rotationYaw.set((self.m_rotationYaw.get() as f64 + yaw as f64 * 0.15) as f32);
        self.m_rotationPitch.set((self.m_rotationPitch.get() as f64 - pitch as f64 * 0.15) as f32);
        if self.m_rotationPitch.get() < -90.0
        {
            self.m_rotationPitch.set(-90.0);
        }
        if self.m_rotationPitch.get() > 90.0
        {
            self.m_rotationPitch.set(90.0);
        }
        self.m_prevRotationPitch.set(self.m_prevRotationPitch.get() + (self.m_rotationPitch.get() - f));
        self.m_prevRotationYaw.set(self.m_prevRotationYaw.get() + (self.m_rotationYaw.get() - f1));
    }

    pub fn setPositionAndRotation(&self, x: f64, y: f64, z: f64, yaw: f32, pitch: f32)
    {
        self.m_posX.set(x);
        self.m_prevPosX.set(x);
        self.m_posY.set(y);
        self.m_prevPosY.set(y);
        self.m_posZ.set(z);
        self.m_prevPosZ.set(z);
        self.m_rotationYaw.set(yaw);
        self.m_prevRotationYaw.set(yaw);
        self.m_rotationPitch.set(pitch);
        self.m_prevRotationPitch.set(pitch);
        self.m_ySize.set(0.0);
        let d0: f64 = (self.m_prevRotationYaw.get() - yaw) as f64;
        if d0 < -180.0
        {
            self.m_prevRotationYaw.set(self.m_prevRotationYaw.get() + 360.0);
        }
        if d0 >= 180.0
        {
            self.m_prevRotationYaw.set(self.m_prevRotationYaw.get() - 360.0);
        }
        self.setPosition(self.posX(), self.posY(), self.posZ());
        self.setRotation(yaw, pitch);
    }

    pub fn setLocationAndAngles(&self, x: f64, y: f64, z: f64, yaw: f32, pitch: f32)
    {
        self.m_posX.set(x);
        self.m_prevPosX.set(x);
        self.m_lastTickPosX.set(x);
        let y: f64 = y + self.m_yOffset.get() as f64;
        self.m_posY.set(y);
        self.m_prevPosY.set(y);
        self.m_lastTickPosY.set(y);
        self.m_posZ.set(z);
        self.m_prevPosZ.set(z);
        self.m_lastTickPosZ.set(z);
        self.m_rotationYaw.set(yaw);
        self.m_rotationPitch.set(pitch);
        self.setPosition(self.posX(), self.posY(), self.posZ());
    }

    pub fn getDistanceToEntity(&self, entity: &Entity) -> f32
    {
        let f: f32 = (self.posX() - entity.posX()) as f32;
        let f1: f32 = (self.posY() - entity.posY()) as f32;
        let f2: f32 = (self.posZ() - entity.posZ()) as f32;
        crate::math::math_helper::sqrt_float(f * f + f1 * f1 + f2 * f2)
    }

    pub fn getDistanceSq(&self, x: f64, y: f64, z: f64) -> f64
    {
        let d0: f64 = self.posX() - x;
        let d1: f64 = self.posY() - y;
        let d2: f64 = self.posZ() - z;
        d0 * d0 + d1 * d1 + d2 * d2
    }

    pub fn getDistance(&self, x: f64, y: f64, z: f64) -> f64
    {
        let d0: f64 = self.posX() - x;
        let d1: f64 = self.posY() - y;
        let d2: f64 = self.posZ() - z;
        crate::math::math_helper::sqrt_double(d0 * d0 + d1 * d1 + d2 * d2) as f64
    }

    pub fn getDistanceSqToEntity(&self, entity: &Entity) -> f64
    {
        let d0: f64 = self.posX() - entity.posX();
        let d1: f64 = self.posY() - entity.posY();
        let d2: f64 = self.posZ() - entity.posZ();
        d0 * d0 + d1 * d1 + d2 * d2
    }

    pub fn setBeenAttacked(&self) { self.m_beenAttacked.set(true); }

    pub fn getEntityString(&self) -> Option<JString> { crate::entity::EntityList::getEntityString(self) }

    pub fn isBurning(&self) -> bool { self.m_fire.get() > 0 || self.getEntityFlag(0) }

    pub fn isRiding(&self) -> bool { self.m_ridingEntity.with(Option::is_some) || self.getEntityFlag(2) }

    pub fn getEntityFlag(&self, index: i32) -> bool
    {
        (self.m_dataWatcher.getWatchableObjectByte(0) as i32 & 1 << index) != 0
    }

    pub fn setSneaking(&self, sneaking: bool) { self.setEntityFlag(1, sneaking); }

    pub fn setEntityFlag(&self, index: i32, value: bool)
    {
        let b0: i8 = self.m_dataWatcher.getWatchableObjectByte(0);
        let flags: i8 = if value
        {
            (b0 as i32 | 1 << index) as i8
        }
        else
        {
            (b0 as i32 & !(1 << index)) as i8
        };
        self.m_dataWatcher.updateObject(0, crate::entity::WatchedValue::Byte(flags));
    }

    pub fn isWet(self: &Rc<Self>) -> bool
    {
        use crate::math::math_helper::floor_double;
        self.m_inWater.get()
            || self.world().canBlockBeRainedOn(
                floor_double(self.posX()),
                floor_double(self.posY()),
                floor_double(self.posZ()),
            )
    }

    pub fn handleLavaMovement(self: &Rc<Self>) -> bool
    {
        let bb: Rc<AxisAlignedBB> = self.m_boundingBox.expand(-0.1f32 as f64, -0.4f32 as f64, -0.1f32 as f64);
        self.world().isMaterialInBB(&bb, crate::block::Material::lava())
    }

    pub fn isInsideOfMaterial(self: &Rc<Self>, liquid: &Rc<crate::block::Material>) -> bool
    {
        use crate::math::math_helper::{floor_double, floor_float};
        let d0: f64 = self.posY() + self.getEyeHeight() as f64;
        let i: i32 = floor_double(self.posX());
        let j: i32 = floor_float(floor_double(d0) as f32);
        let k: i32 = floor_double(self.posZ());
        let world: Rc<World> = self.world();
        let l: i32 = world.getBlockId(i, j, k);
        if l != 0 && Rc::ptr_eq(&crate::block::Block::byId(l).orNpe().m_blockMaterial, liquid)
        {
            let f: f32 = crate::block::block_fluid::getPercentAir(world.getBlockMetadata(i, j, k)) - 0.11111111;
            let f1: f32 = (j + 1) as f32 - f;
            return d0 < f1 as f64;
        }
        false
    }

    pub fn moveFlying(&self, sideways: f32, forwards: f32, scale: f32)
    {
        use crate::math::math_helper;
        let mut sideways: f32 = sideways;
        let mut forwards: f32 = forwards;
        let mut sqrtFloat: f32 = math_helper::sqrt_float(sideways * sideways + forwards * forwards);
        if sqrtFloat < 0.01
        {
            return;
        }
        if sqrtFloat < 1.0
        {
            sqrtFloat = 1.0;
        }
        let n: f32 = scale / sqrtFloat;
        sideways *= n;
        forwards *= n;
        let sin: f32 = math_helper::sin(self.m_rotationYaw.get() * 3.1415927 / 180.0);
        let cos: f32 = math_helper::cos(self.m_rotationYaw.get() * 3.1415927 / 180.0);
        self.m_motionX.set(self.m_motionX.get() + (sideways * cos - forwards * sin) as f64);
        self.m_motionZ.set(self.m_motionZ.get() + (forwards * cos + sideways * sin) as f64);
    }

    pub fn dropItem(self: &Rc<Self>, item: i32, amount: i32) -> Rc<Entity>
    {
        self.dropItemWithOffset(item, amount, 0.0)
    }

    pub fn dropItemWithOffset(self: &Rc<Self>, item: i32, amount: i32, offsetY: f32) -> Rc<Entity>
    {
        self.entityDropItem(crate::item::ItemStack::withDamage(item, amount, 0), offsetY)
    }

    pub fn entityDropItem(self: &Rc<Self>, item: Rc<crate::item::ItemStack>, offsetY: f32) -> Rc<Entity>
    {
        let world: Rc<World> = self.world();
        let entityitem: Rc<Entity> =
            crate::entity::entity_item::create(&world, self.posX(), self.posY() + offsetY as f64, self.posZ(), item);
        crate::entity::entity_item::fields(&entityitem).m_delayBeforeCanPickup.set(10);
        world.entityJoinedWorld(&entityitem);
        entityitem
    }
}

impl JavaObject for Entity
{
    fn javaEquals(&self, other: &Self) -> bool { self.equals(other) }

    fn javaHash(&self) -> i32 { self.hashCode() }
}

pub fn setWorld(this: &Rc<Entity>, world: Option<Rc<World>>) { this.m_worldObj.set(world); }
