use crate::block::Material;
use crate::entity::entity_arrow::{applyHeading, launchFrom, rotateTowardsMotion};
use crate::entity::entity_boat::LerpFields;
use crate::entity::entity_snowball::{self, ThrowableFields, tickInGround, traceHit};
use crate::entity::{ENTITY, Entity, EntityClass, EntityExt, entity_item, entity_player};
use crate::item::{Item, ItemStack};
use crate::math::AxisAlignedBB;
use crate::math::math_helper::{floor_double, sqrt_double};
use crate::stats::StatList;
use crate::world::World;
use jvm::field::Field;
use jvm::throwable::OrNpe;
use std::cell::Cell;
use std::rc::Rc;

pub struct FishFields
{
    pub m_throwable: ThrowableFields,
    m_ticksCatchable: Cell<i32>,
    pub m_bobber: Field<Option<Rc<Entity>>>,
    m_lerp: LerpFields,
}

impl EntityExt for FishFields
{
    fn asAny(&self) -> &dyn std::any::Any { self }
}

pub fn fields(this: &Entity) -> &FishFields { this.ext::<FishFields>() }

fn angler(this: &Entity) -> Rc<Entity> { fields(this).m_throwable.m_thrower.get().orNpe() }

fn alloc(world: Option<Rc<World>>) -> Rc<Entity>
{
    let entity: Rc<Entity> = Entity::alloc(
        &CLASS,
        Box::new(FishFields {
            m_throwable: ThrowableFields::new(),
            m_ticksCatchable: Cell::new(0),
            m_bobber: Field::new(None),
            m_lerp: LerpFields::new(),
        }),
    );
    Entity::init(&entity, world);
    fields(&entity).m_throwable.initTile();
    entity
}

pub fn new(world: Option<Rc<World>>) -> Rc<Entity>
{
    let entity: Rc<Entity> = alloc(world);
    entity.setSize(0.25, 0.25);
    entity.m_ignoreFrustumCheck.set(true);
    entity
}

pub fn create(world: &Rc<World>, x: f64, y: f64, z: f64) -> Rc<Entity>
{
    let entity: Rc<Entity> = new(Some(Rc::clone(world)));
    entity.setPosition(x, y, z);
    entity.m_ignoreFrustumCheck.set(true);
    entity
}

pub fn createFromAngler(world: &Rc<World>, thrower: &Rc<Entity>) -> Rc<Entity>
{
    let entity: Rc<Entity> = alloc(Some(Rc::clone(world)));
    entity.m_ignoreFrustumCheck.set(true);
    fields(&entity).m_throwable.m_thrower.set(Some(Rc::clone(thrower)));
    entity_player::fields(thrower).m_fishEntity.set(Some(Rc::clone(&entity)));
    entity.setSize(0.25, 0.25);
    launchFrom(&entity, thrower, thrower.posY() + 1.62 - thrower.m_yOffset.get() as f64, 0.4);
    thrown(&entity, entity.m_motionX.get(), entity.m_motionY.get(), entity.m_motionZ.get(), 1.5, 1.0);
    entity
}

pub fn thrown(this: &Rc<Entity>, x: f64, y: f64, z: f64, speed: f32, spread: f32)
{
    applyHeading(this, x, y, z, speed, spread);
    fields(this).m_throwable.resetTicksInGround();
}

fn followAngler(this: &Rc<Entity>, fish: &FishFields) -> bool
{
    let angler: Rc<Entity> = angler(this);
    let itemStack: Option<Rc<ItemStack>> = entity_player::getCurrentEquippedItem(&angler);
    if angler.m_isDead.get()
        || !angler.isEntityAlive()
        || itemStack.is_none_or(|stack| stack.m_itemID.get() != Item::fishingRod().m_shiftedIndex)
        || this.getDistanceSqToEntity(&angler) > 1024.0
    {
        this.setEntityDead();
        entity_player::fields(&angler).m_fishEntity.set(None);
        return true;
    }
    let Some(bobber) = fish.m_bobber.get()
    else
    {
        return false;
    };
    if bobber.m_isDead.get()
    {
        fish.m_bobber.set(None);
        return false;
    }
    this.m_posX.set(bobber.posX());
    this.m_posY.set(bobber.m_boundingBox.minY() + bobber.m_height.get() as f64 * 0.8);
    this.m_posZ.set(bobber.posZ());
    true
}

fn spawnSplash(this: &Rc<Entity>, world: &World, kind: &str, f4: f32, motionY: impl Fn() -> f64)
{
    let width: f32 = this.m_width.get();
    let mut n4: i32 = 0;
    while (n4 as f32) < 1.0 + width * 20.0
    {
        let f5: f32 = (this.m_rand.nextFloat() * 2.0 - 1.0) * width;
        let f3: f32 = (this.m_rand.nextFloat() * 2.0 - 1.0) * width;
        world.spawnParticle(
            kind,
            this.posX() + f5 as f64,
            (f4 + 1.0) as f64,
            this.posZ() + f3 as f64,
            this.m_motionX.get(),
            motionY(),
            this.m_motionZ.get(),
        );
        n4 += 1;
    }
}

fn onUpdate(this: &Rc<Entity>)
{
    (ENTITY.onUpdate)(this);
    let fish: &FishFields = fields(this);
    if fish.m_lerp.steps() > 0
    {
        fish.m_lerp.step(this);
        return;
    }
    let world: Rc<World> = this.world();
    if !world.m_multiplayerWorld.get() && followAngler(this, fish)
    {
        return;
    }
    if tickInGround(this, &fish.m_throwable)
    {
        return;
    }

    if let Some(hit) = traceHit(this, &fish.m_throwable, 5, true)
    {
        match &hit.m_entityHit
        {
            Some(entityHit) =>
            {
                if entityHit.attackEntityFrom(fish.m_throwable.m_thrower.get().as_ref(), 0)
                {
                    fish.m_bobber.set(Some(Rc::clone(entityHit)));
                }
            }
            None => fish.m_throwable.m_inGround.set(true),
        }
    }
    if fish.m_throwable.m_inGround.get()
    {
        return;
    }

    this.moveEntity(this.m_motionX.get(), this.m_motionY.get(), this.m_motionZ.get());
    rotateTowardsMotion(this);
    let mut f2: f32 = 0.92;
    if this.m_onGround.get() || this.m_isCollidedHorizontally.get()
    {
        f2 = 0.5;
    }
    let n3: i32 = 5;
    let mut d6: f64 = 0.0;
    for n in 0..n3
    {
        let bounds: &AxisAlignedBB = &this.m_boundingBox;
        let d7: f64 = bounds.minY() + (bounds.maxY() - bounds.minY()) * (n + 0) as f64 / n3 as f64 - 0.125 + 0.125;
        let d8: f64 = bounds.minY() + (bounds.maxY() - bounds.minY()) * (n + 1) as f64 / n3 as f64 - 0.125 + 0.125;
        let axisAlignedBB: Rc<AxisAlignedBB> =
            AxisAlignedBB::getBoundingBoxFromPool(bounds.minX(), d7, bounds.minZ(), bounds.maxX(), d8, bounds.maxZ());
        if world.isAABBInMaterial(&axisAlignedBB, Material::water())
        {
            d6 += 1.0 / n3 as f64;
        }
    }

    if d6 > 0.0
    {
        if fish.m_ticksCatchable.get() > 0
        {
            fish.m_ticksCatchable.set(fish.m_ticksCatchable.get() - 1);
        }
        else
        {
            let mut n: i32 = 500;
            if world.canBlockBeRainedOn(
                floor_double(this.posX()),
                floor_double(this.posY()) + 1,
                floor_double(this.posZ()),
            )
            {
                n = 300;
            }
            if this.m_rand.nextIntBounded(n) == 0
            {
                fish.m_ticksCatchable.set(this.m_rand.nextIntBounded(30) + 10);
                this.m_motionY.set(this.m_motionY.get() - 0.2f32 as f64);
                world.playSoundAtEntity(
                    this,
                    "random.splash",
                    0.25,
                    1.0 + (this.m_rand.nextFloat() - this.m_rand.nextFloat()) * 0.4,
                );
                let f4: f32 = floor_double(this.m_boundingBox.minY()) as f32;
                spawnSplash(this, &world, "bubble", f4, || {
                    this.m_motionY.get() - (this.m_rand.nextFloat() * 0.2) as f64
                });
                spawnSplash(this, &world, "splash", f4, || this.m_motionY.get());
            }
        }
    }

    if fish.m_ticksCatchable.get() > 0
    {
        this.m_motionY.set(
            this.m_motionY.get()
                - (this.m_rand.nextFloat() * this.m_rand.nextFloat() * this.m_rand.nextFloat()) as f64 * 0.2,
        );
    }
    let d9: f64 = d6 * 2.0 - 1.0;
    this.m_motionY.set(this.m_motionY.get() + 0.04f32 as f64 * d9);
    if d6 > 0.0
    {
        f2 = (f2 as f64 * 0.9) as f32;
        this.m_motionY.set(this.m_motionY.get() * 0.8);
    }
    this.m_motionX.set(this.m_motionX.get() * f2 as f64);
    this.m_motionY.set(this.m_motionY.get() * f2 as f64);
    this.m_motionZ.set(this.m_motionZ.get() * f2 as f64);
    this.setPosition(this.posX(), this.posY(), this.posZ());
}

fn pullTowardAngler(this: &Entity, angler: &Entity) -> (f64, f64, f64)
{
    let d: f64 = angler.posX() - this.posX();
    let d2: f64 = angler.posY() - this.posY();
    let d3: f64 = angler.posZ() - this.posZ();
    let d4: f64 = sqrt_double(d * d + d2 * d2 + d3 * d3) as f64;
    let d5: f64 = 0.1;
    (d * d5, d2 * d5 + sqrt_double(d4) as f64 * 0.08, d3 * d5)
}

pub fn catchFish(this: &Rc<Entity>) -> i32
{
    let fish: &FishFields = fields(this);
    let angler: Rc<Entity> = angler(this);
    let mut n: i32 = 0;
    if let Some(bobber) = fish.m_bobber.get()
    {
        let (x, y, z): (f64, f64, f64) = pullTowardAngler(this, &angler);
        bobber.m_motionX.set(bobber.m_motionX.get() + x);
        bobber.m_motionY.set(bobber.m_motionY.get() + y);
        bobber.m_motionZ.set(bobber.m_motionZ.get() + z);
        n = 3;
    }
    else if fish.m_ticksCatchable.get() > 0
    {
        let world: Rc<World> = this.world();
        let entityItem: Rc<Entity> =
            entity_item::create(&world, this.posX(), this.posY(), this.posZ(), ItemStack::withItem(&Item::fishRaw()));
        let (x, y, z): (f64, f64, f64) = pullTowardAngler(this, &angler);
        entityItem.m_motionX.set(x);
        entityItem.m_motionY.set(y);
        entityItem.m_motionZ.set(z);
        world.entityJoinedWorld(&entityItem);
        angler.addStat(StatList::fishCaughtStat().as_ref(), 1);
        n = 1;
    }
    if fish.m_throwable.m_inGround.get()
    {
        n = 2;
    }
    this.setEntityDead();
    entity_player::fields(&angler).m_fishEntity.set(None);
    n
}

pub const CLASS: EntityClass = EntityClass {
    m_name: "EntityFish",
    m_parent: Some(&ENTITY),
    m_interfaces: &[],
    entityInit: |_| {},
    isInRangeToRenderDist: entity_snowball::isInRangeToRenderDist,
    setPositionAndRotation2: |this, x, y, z, yaw, pitch, steps| {
        fields(this).m_lerp.setTarget(this, x, y, z, yaw, pitch, steps)
    },
    setVelocity: |this, x, y, z| fields(this).m_lerp.setVelocity(this, x, y, z),
    onUpdate,
    writeEntityToNBT: entity_snowball::writeEntityToNBT,
    readEntityFromNBT: entity_snowball::readEntityFromNBT,
    getShadowSize: |_| 0.0,
    ..ENTITY
};
