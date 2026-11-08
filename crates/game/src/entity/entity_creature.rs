use crate::entity::{Entity, EntityClass, entity_living};
use crate::math::{Vec3D, math_helper};
use crate::pathfinding::PathEntity;
use crate::world::World;
use jvm::field::Field;
use jvm::math;
use std::cell::Cell;
use std::rc::Rc;

#[derive(Default)]
pub struct CreatureFields
{
    pub m_pathToEntity: Field<Option<Rc<PathEntity>>>,
    pub m_playerToAttack: Field<Option<Rc<Entity>>>,
    pub m_hasAttacked: Cell<bool>,
}

pub fn fields(this: &Entity) -> &CreatureFields
{
    match this.m_ext.creature()
    {
        Some(fields) => fields,
        None => jvm::throwable::throw(
            &jvm::throwable::CLASS_CAST,
            &format!("net.minecraft.src.{} cannot be cast to net.minecraft.src.EntityCreature", this.className()),
        ),
    }
}

pub fn init(this: &Rc<Entity>, world: Option<Rc<World>>)
{
    entity_living::init(this, world);
    fields(this).m_hasAttacked.set(false);
}

const PI_F: f32 = std::f64::consts::PI as f32;

fn updatePlayerActionState(this: &Rc<Entity>)
{
    let creature: &CreatureFields = fields(this);
    let living: &entity_living::LivingFields = entity_living::fields(this);
    let world: Rc<World> = this.world();
    creature.m_hasAttacked.set(this.isMovementCeased());
    let f: f32 = 16.0;
    match creature.m_playerToAttack.get()
    {
        None =>
        {
            creature.m_playerToAttack.set(this.findPlayerToAttack());
            if let Some(target) = creature.m_playerToAttack.get()
            {
                creature.m_pathToEntity.set(world.getPathToEntity(this, &target, f));
            }
        }
        Some(target) if !target.isEntityAlive() => creature.m_playerToAttack.set(None),
        Some(target) =>
        {
            let f1: f32 = target.getDistanceToEntity(this);
            if entity_living::canEntityBeSeen(this, &target)
            {
                this.attackEntity(&target, f1);
            }
            else
            {
                this.attackBlockedEntity(&target, f1);
            }
        }
    }
    let hasPath: bool = creature.m_pathToEntity.with(Option::is_some);
    if creature.m_hasAttacked.get()
        || creature.m_playerToAttack.with(Option::is_none)
        || hasPath && this.m_rand.nextIntBounded(20) != 0
    {
        if !creature.m_hasAttacked.get()
            && (creature.m_pathToEntity.with(Option::is_none) && this.m_rand.nextIntBounded(80) == 0
                || this.m_rand.nextIntBounded(80) == 0)
        {
            updateWanderingPath(this);
        }
    }
    else
    {
        let target: Rc<Entity> = creature.m_playerToAttack.get().expect("checked non null above");
        creature.m_pathToEntity.set(world.getPathToEntity(this, &target, f));
    }
    let i: i32 = math_helper::floor_double(this.m_boundingBox.minY() + 0.5);
    let flag: bool = this.isInWater();
    let flag1: bool = this.handleLavaMovement();
    this.m_rotationPitch.set(0.0);
    if creature.m_pathToEntity.with(Option::is_some) && this.m_rand.nextIntBounded(100) != 0
    {
        let mut vec3d: Option<Rc<Vec3D>> = creature.m_pathToEntity.get().map(|path| path.getPosition(this));
        let d0: f64 = (this.m_width.get() * 2.0) as f64;
        while let Some(point) = vec3d.clone()
            && point.squareDistanceToPoint(this.posX(), point.y(), this.posZ()) < d0 * d0
        {
            let path: Rc<PathEntity> = creature.m_pathToEntity.get().expect("path set while walking it");
            path.incrementPathIndex();
            if path.isFinished()
            {
                vec3d = None;
                creature.m_pathToEntity.set(None);
            }
            else
            {
                vec3d = Some(path.getPosition(this));
            }
        }
        living.m_isJumping.set(false);
        if let Some(point) = vec3d
        {
            let d1: f64 = point.x() - this.posX();
            let d2: f64 = point.z() - this.posZ();
            let d3: f64 = point.y() - i as f64;
            let f2: f32 = (math::atan2(d2, d1) * 180.0 / PI_F as f64) as f32 - 90.0;
            let mut f3: f32 = f2 - this.m_rotationYaw.get();
            living.m_moveForward.set(living.m_moveSpeed.get());
            while f3 < -180.0
            {
                f3 += 360.0;
            }
            while f3 >= 180.0
            {
                f3 -= 360.0;
            }
            if f3 > 30.0
            {
                f3 = 30.0;
            }
            if f3 < -30.0
            {
                f3 = -30.0;
            }
            this.m_rotationYaw.set(this.m_rotationYaw.get() + f3);
            if creature.m_hasAttacked.get()
                && let Some(target) = creature.m_playerToAttack.get()
            {
                let d4: f64 = target.posX() - this.posX();
                let d5: f64 = target.posZ() - this.posZ();
                let f4: f32 = this.m_rotationYaw.get();
                this.m_rotationYaw.set((math::atan2(d5, d4) * 180.0 / PI_F as f64) as f32 - 90.0);
                f3 = (f4 - this.m_rotationYaw.get() + 90.0) * PI_F / 180.0;
                living.m_moveStrafing.set(-math_helper::sin(f3) * living.m_moveForward.get() * 1.0);
                living.m_moveForward.set(math_helper::cos(f3) * living.m_moveForward.get() * 1.0);
            }
            if d3 > 0.0
            {
                living.m_isJumping.set(true);
            }
        }
        if let Some(target) = creature.m_playerToAttack.get()
        {
            entity_living::faceEntity(this, &target, 30.0, 30.0);
        }
        if this.m_isCollidedHorizontally.get() && !hasPathTo(this)
        {
            living.m_isJumping.set(true);
        }
        if this.m_rand.nextFloat() < 0.8 && (flag || flag1)
        {
            living.m_isJumping.set(true);
        }
    }
    else
    {
        (entity_living::CLASS.updatePlayerActionState)(this);
        creature.m_pathToEntity.set(None);
    }
}

pub fn updateWanderingPath(this: &Rc<Entity>)
{
    let mut flag: bool = false;
    let mut i: i32 = -1;
    let mut j: i32 = -1;
    let mut k: i32 = -1;
    let mut f: f32 = -99999.0;
    for _ in 0..10
    {
        let i1: i32 = math_helper::floor_double(this.posX() + this.m_rand.nextIntBounded(13) as f64 - 6.0);
        let j1: i32 = math_helper::floor_double(this.posY() + this.m_rand.nextIntBounded(7) as f64 - 3.0);
        let k1: i32 = math_helper::floor_double(this.posZ() + this.m_rand.nextIntBounded(13) as f64 - 6.0);
        let f1: f32 = this.getBlockPathWeight(i1, j1, k1);
        if f1 > f
        {
            f = f1;
            i = i1;
            j = j1;
            k = k1;
            flag = true;
        }
    }
    if flag
    {
        fields(this).m_pathToEntity.set(this.world().getEntityPathToXYZ(this, i, j, k, 10.0));
    }
}

pub fn hasPathTo(this: &Entity) -> bool { fields(this).m_pathToEntity.with(Option::is_some) }

pub fn setPathToEntity(this: &Entity, path: Option<Rc<PathEntity>>) { fields(this).m_pathToEntity.set(path); }

pub fn getTarget(this: &Entity) -> Option<Rc<Entity>> { fields(this).m_playerToAttack.get() }

pub fn setTarget(this: &Entity, target: Option<Rc<Entity>>) { fields(this).m_playerToAttack.set(target); }

pub const CLASS: EntityClass = EntityClass {
    m_name: "EntityCreature",
    m_parent: Some(&entity_living::CLASS),
    isMovementCeased: |_| false,
    updatePlayerActionState,
    attackEntity: |_, _, _| {},
    attackBlockedEntity: |_, _, _| {},
    getBlockPathWeight: |_, _, _, _| 0.0,
    findPlayerToAttack: |_| None,
    getCanSpawnHere: |this| {
        let i: i32 = math_helper::floor_double(this.posX());
        let j: i32 = math_helper::floor_double(this.m_boundingBox.minY());
        let k: i32 = math_helper::floor_double(this.posZ());
        (entity_living::CLASS.getCanSpawnHere)(this) && this.getBlockPathWeight(i, j, k) >= 0.0
    },
    ..entity_living::CLASS
};
