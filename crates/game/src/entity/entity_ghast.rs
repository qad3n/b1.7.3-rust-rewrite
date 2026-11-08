use crate::entity::{Entity, EntityClass, EntityExt, WatchedValue, entity_fireball, entity_flying, entity_living};
use crate::item::Item;
use crate::math::{AxisAlignedBB, Vec3D, math_helper};
use crate::world::World;
use jvm::field::Field;
use jvm::string::JString;
use std::cell::Cell;
use std::rc::Rc;

const PI_F: f32 = std::f64::consts::PI as f32;

#[derive(Default)]
pub struct GhastExt
{
    pub m_living: entity_living::LivingFields,
    pub m_courseChangeCooldown: Cell<i32>,
    pub m_waypointX: Cell<f64>,
    pub m_waypointY: Cell<f64>,
    pub m_waypointZ: Cell<f64>,
    m_targetedEntity: Field<Option<Rc<Entity>>>,
    m_aggroCooldown: Cell<i32>,
    pub m_prevAttackCounter: Cell<i32>,
    pub m_attackCounter: Cell<i32>,
}

impl EntityExt for GhastExt
{
    fn asAny(&self) -> &dyn std::any::Any { self }

    fn living(&self) -> Option<&entity_living::LivingFields> { Some(&self.m_living) }
}

pub fn fields(this: &Entity) -> &GhastExt { this.ext::<GhastExt>() }

fn isCourseTraversable(this: &Rc<Entity>, _targetX: f64, _targetY: f64, _targetZ: f64, distance: f64) -> bool
{
    let ghast: &GhastExt = fields(this);
    let d0: f64 = (ghast.m_waypointX.get() - this.posX()) / distance;
    let d1: f64 = (ghast.m_waypointY.get() - this.posY()) / distance;
    let d2: f64 = (ghast.m_waypointZ.get() - this.posZ()) / distance;
    let axisalignedbb: Rc<AxisAlignedBB> = this.m_boundingBox.copy();
    let world: Rc<World> = this.world();
    let mut i: i32 = 1;
    while (i as f64) < distance
    {
        axisalignedbb.offset(d0, d1, d2);
        if world.getCollidingBoundingBoxes(this, &axisalignedbb).size() > 0
        {
            return false;
        }
        i += 1;
    }
    true
}

fn faceYaw(this: &Entity, x: f64, z: f64)
{
    let yaw: f32 = -(jvm::math::atan2(x, z) as f32) * 180.0 / PI_F;
    this.m_rotationYaw.set(yaw);
    entity_living::fields(this).m_renderYawOffset.set(yaw);
}

fn randomWaypoint(this: &Entity, position: f64) -> f64
{
    position + ((this.m_rand.nextFloat() * 2.0 - 1.0) * 16.0) as f64
}

fn updatePlayerActionState(this: &Rc<Entity>)
{
    let ghast: &GhastExt = fields(this);
    let world: Rc<World> = this.world();
    if !world.m_multiplayerWorld.get() && world.m_difficultySetting.get() == 0
    {
        this.setEntityDead();
    }
    entity_living::checkDespawn(this);
    ghast.m_prevAttackCounter.set(ghast.m_attackCounter.get());
    let d0: f64 = ghast.m_waypointX.get() - this.posX();
    let d1: f64 = ghast.m_waypointY.get() - this.posY();
    let d2: f64 = ghast.m_waypointZ.get() - this.posZ();
    let d3: f64 = math_helper::sqrt_double(d0 * d0 + d1 * d1 + d2 * d2) as f64;
    if d3 < 1.0 || d3 > 60.0
    {
        ghast.m_waypointX.set(randomWaypoint(this, this.posX()));
        ghast.m_waypointY.set(randomWaypoint(this, this.posY()));
        ghast.m_waypointZ.set(randomWaypoint(this, this.posZ()));
    }
    let cooldown: i32 = ghast.m_courseChangeCooldown.get();
    ghast.m_courseChangeCooldown.set(cooldown - 1);
    if cooldown <= 0
    {
        ghast.m_courseChangeCooldown.set(ghast.m_courseChangeCooldown.get() + this.m_rand.nextIntBounded(5) + 2);
        if isCourseTraversable(this, ghast.m_waypointX.get(), ghast.m_waypointY.get(), ghast.m_waypointZ.get(), d3)
        {
            this.m_motionX.set(this.m_motionX.get() + d0 / d3 * 0.1);
            this.m_motionY.set(this.m_motionY.get() + d1 / d3 * 0.1);
            this.m_motionZ.set(this.m_motionZ.get() + d2 / d3 * 0.1);
        }
        else
        {
            ghast.m_waypointX.set(this.posX());
            ghast.m_waypointY.set(this.posY());
            ghast.m_waypointZ.set(this.posZ());
        }
    }
    if ghast.m_targetedEntity.with(|target| target.as_ref().is_some_and(|target| target.m_isDead.get()))
    {
        ghast.m_targetedEntity.set(None);
    }
    let retarget: bool = ghast.m_targetedEntity.with(Option::is_none) || {
        let aggro: i32 = ghast.m_aggroCooldown.get();
        ghast.m_aggroCooldown.set(aggro - 1);
        aggro <= 0
    };
    if retarget
    {
        ghast.m_targetedEntity.set(world.getClosestPlayerToEntity(this, 100.0));
        if ghast.m_targetedEntity.with(Option::is_some)
        {
            ghast.m_aggroCooldown.set(20);
        }
    }
    let d4: f64 = 64.0;
    match ghast.m_targetedEntity.get().filter(|target| target.getDistanceSqToEntity(this) < d4 * d4)
    {
        Some(target) =>
        {
            let d5: f64 = target.posX() - this.posX();
            let d6: f64 = target.m_boundingBox.minY() + (target.m_height.get() / 2.0) as f64
                - (this.posY() + (this.m_height.get() / 2.0) as f64);
            let d7: f64 = target.posZ() - this.posZ();
            faceYaw(this, d5, d7);
            if entity_living::canEntityBeSeen(this, &target)
            {
                if ghast.m_attackCounter.get() == 10
                {
                    let pitch: f32 = (this.m_rand.nextFloat() - this.m_rand.nextFloat()) * 0.2 + 1.0;
                    world.playSoundAtEntity(this, "mob.ghast.charge", this.getSoundVolume(), pitch);
                }
                ghast.m_attackCounter.set(ghast.m_attackCounter.get() + 1);
                if ghast.m_attackCounter.get() == 20
                {
                    let pitch: f32 = (this.m_rand.nextFloat() - this.m_rand.nextFloat()) * 0.2 + 1.0;
                    world.playSoundAtEntity(this, "mob.ghast.fireball", this.getSoundVolume(), pitch);
                    let entityfireball: Rc<Entity> = entity_fireball::createFromShooter(&world, this, d5, d6, d7);
                    let d8: f64 = 4.0;
                    let vec3d: Rc<Vec3D> = entity_living::getLook(this, 1.0);
                    entityfireball.m_posX.set(this.posX() + vec3d.m_xCoord.get() * d8);
                    entityfireball.m_posY.set(this.posY() + (this.m_height.get() / 2.0) as f64 + 0.5);
                    entityfireball.m_posZ.set(this.posZ() + vec3d.m_zCoord.get() * d8);
                    world.entityJoinedWorld(&entityfireball);
                    ghast.m_attackCounter.set(-40);
                }
            }
            else if ghast.m_attackCounter.get() > 0
            {
                ghast.m_attackCounter.set(ghast.m_attackCounter.get() - 1);
            }
        }
        None =>
        {
            faceYaw(this, this.m_motionX.get(), this.m_motionZ.get());
            if ghast.m_attackCounter.get() > 0
            {
                ghast.m_attackCounter.set(ghast.m_attackCounter.get() - 1);
            }
        }
    }
    if !world.m_multiplayerWorld.get()
    {
        let b1: i8 = this.m_dataWatcher.getWatchableObjectByte(16);
        let b0: i8 = if ghast.m_attackCounter.get() > 10
        {
            1
        }
        else
        {
            0
        };
        if b1 != b0
        {
            this.m_dataWatcher.updateObject(16, WatchedValue::Byte(b0));
        }
    }
}

pub const CLASS: EntityClass = EntityClass {
    m_name: "EntityGhast",
    m_parent: Some(&entity_flying::CLASS),
    m_interfaces: &["IMob", "IAnimals"],
    entityInit: |this| {
        (entity_flying::CLASS.entityInit)(this);
        this.m_dataWatcher.addObject(16, WatchedValue::Byte(0));
    },
    onUpdate: |this| {
        (entity_flying::CLASS.onUpdate)(this);
        let b0: i8 = this.m_dataWatcher.getWatchableObjectByte(16);
        let texture: &str = if b0 == 1
        {
            "/mob/ghast_fire.png"
        }
        else
        {
            "/mob/ghast.png"
        };
        entity_living::fields(this).m_texture.set(Some(JString::from(texture)));
    },
    updatePlayerActionState,
    getLivingSound: |_| Some("mob.ghast.moan".to_string()),
    getHurtSound: |_| Some("mob.ghast.scream".to_string()),
    getDeathSound: |_| Some("mob.ghast.death".to_string()),
    getDropItemId: |_| Item::gunpowder().m_shiftedIndex,
    getSoundVolume: |_| 10.0,
    getCanSpawnHere: |this| {
        this.m_rand.nextIntBounded(20) == 0
            && (entity_flying::CLASS.getCanSpawnHere)(this)
            && this.world().m_difficultySetting.get() > 0
    },
    getMaxSpawnedInChunk: |_| 1,
    ..entity_flying::CLASS
};

pub fn new(world: Option<Rc<World>>) -> Rc<Entity>
{
    let entity: Rc<Entity> = Entity::alloc(&CLASS, Box::new(GhastExt::default()));
    entity_flying::init(&entity, world);
    entity_living::fields(&entity).m_texture.set(Some(JString::from("/mob/ghast.png")));
    entity.setSize(4.0, 4.0);
    entity.m_isImmuneToFire.set(true);
    entity
}
