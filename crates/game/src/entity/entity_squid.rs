use crate::block::Material;
use crate::entity::{Entity, EntityClass, EntityExt, entity_creature, entity_living, entity_water_mob};
use crate::item::{Item, ItemStack};
use crate::math::math_helper;
use crate::world::World;
use jvm::string::JString;
use std::cell::Cell;
use std::rc::Rc;

const PI_F: f32 = std::f64::consts::PI as f32;

#[derive(Default)]
pub struct SquidExt
{
    pub m_living: entity_living::LivingFields,
    pub m_creature: entity_creature::CreatureFields,
    pub m_bodyPitch: Cell<f32>,
    pub m_lastBodyPitch: Cell<f32>,
    pub m_bodyRoll: Cell<f32>,
    pub m_lastBodyRoll: Cell<f32>,
    pub m_tentacleMovement: Cell<f32>,
    pub m_lastTentacleMovement: Cell<f32>,
    pub m_tentacleRotation: Cell<f32>,
    pub m_lastTentacleRotation: Cell<f32>,
    m_randomMotionSpeed: Cell<f32>,
    m_tentacleSpeed: Cell<f32>,
    m_rotationSpeed: Cell<f32>,
    m_randomMotionVecX: Cell<f32>,
    m_randomMotionVecY: Cell<f32>,
    m_randomMotionVecZ: Cell<f32>,
}

impl EntityExt for SquidExt
{
    fn asAny(&self) -> &dyn std::any::Any { self }

    fn living(&self) -> Option<&entity_living::LivingFields> { Some(&self.m_living) }

    fn creature(&self) -> Option<&entity_creature::CreatureFields> { Some(&self.m_creature) }
}

pub fn fields(this: &Entity) -> &SquidExt { this.ext::<SquidExt>() }

fn randomTentacleSpeed(this: &Entity) -> f32 { 1.0 / (this.m_rand.nextFloat() + 1.0) * 0.2 }

fn onLivingUpdate(this: &Rc<Entity>)
{
    (entity_water_mob::CLASS.onLivingUpdate)(this);
    let squid: &SquidExt = fields(this);
    let living: &entity_living::LivingFields = entity_living::fields(this);
    squid.m_lastBodyPitch.set(squid.m_bodyPitch.get());
    squid.m_lastBodyRoll.set(squid.m_bodyRoll.get());
    squid.m_lastTentacleMovement.set(squid.m_tentacleMovement.get());
    squid.m_lastTentacleRotation.set(squid.m_tentacleRotation.get());
    squid.m_tentacleMovement.set(squid.m_tentacleMovement.get() + squid.m_tentacleSpeed.get());
    if squid.m_tentacleMovement.get() > PI_F * 2.0
    {
        squid.m_tentacleMovement.set(squid.m_tentacleMovement.get() - PI_F * 2.0);
        if this.m_rand.nextIntBounded(10) == 0
        {
            squid.m_tentacleSpeed.set(randomTentacleSpeed(this));
        }
    }
    if this.isInWater()
    {
        if squid.m_tentacleMovement.get() < PI_F
        {
            let f: f32 = squid.m_tentacleMovement.get() / PI_F;
            squid.m_tentacleRotation.set(math_helper::sin(f * f * PI_F) * PI_F * 0.25);
            if f as f64 > 0.75
            {
                squid.m_randomMotionSpeed.set(1.0);
                squid.m_rotationSpeed.set(1.0);
            }
            else
            {
                squid.m_rotationSpeed.set(squid.m_rotationSpeed.get() * 0.8);
            }
        }
        else
        {
            squid.m_tentacleRotation.set(0.0);
            squid.m_randomMotionSpeed.set(squid.m_randomMotionSpeed.get() * 0.9);
            squid.m_rotationSpeed.set(squid.m_rotationSpeed.get() * 0.99);
        }
        if !living.m_isMultiplayerEntity.get()
        {
            this.m_motionX.set((squid.m_randomMotionVecX.get() * squid.m_randomMotionSpeed.get()) as f64);
            this.m_motionY.set((squid.m_randomMotionVecY.get() * squid.m_randomMotionSpeed.get()) as f64);
            this.m_motionZ.set((squid.m_randomMotionVecZ.get() * squid.m_randomMotionSpeed.get()) as f64);
        }
        let f: f32 = math_helper::sqrt_double(
            this.m_motionX.get() * this.m_motionX.get() + this.m_motionZ.get() * this.m_motionZ.get(),
        );
        let yaw: f32 = -(jvm::math::atan2(this.m_motionX.get(), this.m_motionZ.get()) as f32) * 180.0 / PI_F;
        living.m_renderYawOffset.set(living.m_renderYawOffset.get() + (yaw - living.m_renderYawOffset.get()) * 0.1);
        this.m_rotationYaw.set(living.m_renderYawOffset.get());
        squid.m_bodyRoll.set(squid.m_bodyRoll.get() + PI_F * squid.m_rotationSpeed.get() * 1.5);
        let pitch: f32 = -(jvm::math::atan2(f as f64, this.m_motionY.get()) as f32) * 180.0 / PI_F;
        squid.m_bodyPitch.set(squid.m_bodyPitch.get() + (pitch - squid.m_bodyPitch.get()) * 0.1);
    }
    else
    {
        squid.m_tentacleRotation.set(math_helper::abs(math_helper::sin(squid.m_tentacleMovement.get())) * PI_F * 0.25);
        if !living.m_isMultiplayerEntity.get()
        {
            this.m_motionX.set(0.0);
            this.m_motionY.set(this.m_motionY.get() - 0.08);
            this.m_motionY.set(this.m_motionY.get() * 0.98f32 as f64);
            this.m_motionZ.set(0.0);
        }
        squid
            .m_bodyPitch
            .set((squid.m_bodyPitch.get() as f64 + (-90.0 - squid.m_bodyPitch.get()) as f64 * 0.02) as f32);
    }
}

fn updatePlayerActionState(this: &Rc<Entity>)
{
    let squid: &SquidExt = fields(this);
    if this.m_rand.nextIntBounded(50) == 0
        || !this.m_inWater.get()
        || squid.m_randomMotionVecX.get() == 0.0
            && squid.m_randomMotionVecY.get() == 0.0
            && squid.m_randomMotionVecZ.get() == 0.0
    {
        let f: f32 = this.m_rand.nextFloat() * PI_F * 2.0;
        squid.m_randomMotionVecX.set(math_helper::cos(f) * 0.2);
        squid.m_randomMotionVecY.set(-0.1 + this.m_rand.nextFloat() * 0.2);
        squid.m_randomMotionVecZ.set(math_helper::sin(f) * 0.2);
    }
    entity_living::checkDespawn(this);
}

pub const CLASS: EntityClass = EntityClass {
    m_name: "EntitySquid",
    m_parent: Some(&entity_water_mob::CLASS),
    writeEntityToNBT: |this, nbt| (entity_water_mob::CLASS.writeEntityToNBT)(this, nbt),
    readEntityFromNBT: |this, nbt| (entity_water_mob::CLASS.readEntityFromNBT)(this, nbt),
    getLivingSound: |_| None,
    getHurtSound: |_| None,
    getDeathSound: |_| None,
    getSoundVolume: |_| 0.4,
    getDropItemId: |_| 0,
    dropFewItems: |this| {
        let n: i32 = this.m_rand.nextIntBounded(3) + 1;
        for _ in 0..n
        {
            this.entityDropItem(ItemStack::withItemDamage(&Item::dyePowder(), 1, 0), 0.0);
        }
    },
    interact: |_, _| false,
    isInWater: |this| {
        let bounds: Rc<crate::math::AxisAlignedBB> = this.m_boundingBox.expand(0.0, -0.6f32 as f64, 0.0);
        this.world().handleMaterialAcceleration(&bounds, Material::water(), this)
    },
    onLivingUpdate,
    moveEntityWithHeading: |this, _, _| {
        this.moveEntity(this.m_motionX.get(), this.m_motionY.get(), this.m_motionZ.get())
    },
    updatePlayerActionState,
    ..entity_water_mob::CLASS
};

pub fn new(world: Option<Rc<World>>) -> Rc<Entity>
{
    let entity: Rc<Entity> = Entity::alloc(&CLASS, Box::new(SquidExt::default()));
    entity_water_mob::init(&entity, world);
    entity_living::fields(&entity).m_texture.set(Some(JString::from("/mob/squid.png")));
    entity.setSize(0.95, 0.95);
    fields(&entity).m_tentacleSpeed.set(randomTentacleSpeed(&entity));
    entity
}
