use crate::entity::{Entity, EntityClass, EntityExt, WatchedValue, entity_living};
use crate::item::Item;
use crate::math::math_helper;
use crate::world::{Chunk, World};
use jvm::string::JString;
use std::cell::Cell;
use std::rc::Rc;

const PI_F: f32 = std::f64::consts::PI as f32;
const SLIME_CHUNK_SEED: i64 = 987234911;

#[derive(Default)]
pub struct SlimeExt
{
    pub m_living: entity_living::LivingFields,
    pub m_stretch: Cell<f32>,
    pub m_lastStretch: Cell<f32>,
    m_slimeJumpDelay: Cell<i32>,
}

impl EntityExt for SlimeExt
{
    fn asAny(&self) -> &dyn std::any::Any { self }

    fn living(&self) -> Option<&entity_living::LivingFields> { Some(&self.m_living) }
}

pub fn fields(this: &Entity) -> &SlimeExt { this.ext::<SlimeExt>() }

pub fn setSlimeSize(this: &Rc<Entity>, size: i32)
{
    this.m_dataWatcher.updateObject(16, WatchedValue::Byte(size as i8));
    this.setSize(0.6 * size as f32, 0.6 * size as f32);
    entity_living::fields(this).m_health.set(size * size);
    this.setPosition(this.posX(), this.posY(), this.posZ());
}

pub fn getSlimeSize(this: &Rc<Entity>) -> i32 { this.m_dataWatcher.getWatchableObjectByte(16) as i32 }

fn onUpdate(this: &Rc<Entity>)
{
    let slime: &SlimeExt = fields(this);
    slime.m_lastStretch.set(slime.m_stretch.get());
    let flag: bool = this.m_onGround.get();
    (entity_living::CLASS.onUpdate)(this);
    if this.m_onGround.get() && !flag
    {
        let world: Rc<World> = this.world();
        let i: i32 = getSlimeSize(this);
        for _ in 0..i * 8
        {
            let f: f32 = this.m_rand.nextFloat() * PI_F * 2.0;
            let f1: f32 = this.m_rand.nextFloat() * 0.5 + 0.5;
            let f2: f32 = math_helper::sin(f) * i as f32 * 0.5 * f1;
            let f3: f32 = math_helper::cos(f) * i as f32 * 0.5 * f1;
            world.spawnParticle(
                "slime",
                this.posX() + f2 as f64,
                this.m_boundingBox.minY(),
                this.posZ() + f3 as f64,
                0.0,
                0.0,
                0.0,
            );
        }
        if i > 2
        {
            let pitch: f32 = ((this.m_rand.nextFloat() - this.m_rand.nextFloat()) * 0.2 + 1.0) / 0.8;
            world.playSoundAtEntity(this, "mob.slime", this.getSoundVolume(), pitch);
        }
        slime.m_stretch.set(-0.5);
    }
    slime.m_stretch.set(slime.m_stretch.get() * 0.6);
}

fn updatePlayerActionState(this: &Rc<Entity>)
{
    entity_living::checkDespawn(this);
    let slime: &SlimeExt = fields(this);
    let living: &entity_living::LivingFields = entity_living::fields(this);
    let world: Rc<World> = this.world();
    let entityplayer: Option<Rc<Entity>> = world.getClosestPlayerToEntity(this, 16.0);
    if let Some(player) = &entityplayer
    {
        entity_living::faceEntity(this, player, 10.0, 20.0);
    }
    let jump: bool = this.m_onGround.get() && {
        let delay: i32 = slime.m_slimeJumpDelay.get();
        slime.m_slimeJumpDelay.set(delay - 1);
        delay <= 0
    };
    if jump
    {
        slime.m_slimeJumpDelay.set(this.m_rand.nextIntBounded(20) + 10);
        if entityplayer.is_some()
        {
            slime.m_slimeJumpDelay.set(slime.m_slimeJumpDelay.get() / 3);
        }
        living.m_isJumping.set(true);
        if getSlimeSize(this) > 1
        {
            let pitch: f32 = ((this.m_rand.nextFloat() - this.m_rand.nextFloat()) * 0.2 + 1.0) * 0.8;
            world.playSoundAtEntity(this, "mob.slime", this.getSoundVolume(), pitch);
        }
        slime.m_stretch.set(1.0);
        living.m_moveStrafing.set(1.0 - this.m_rand.nextFloat() * 2.0);
        living.m_moveForward.set((1 * getSlimeSize(this)) as f32);
    }
    else
    {
        living.m_isJumping.set(false);
        if this.m_onGround.get()
        {
            living.m_moveForward.set(0.0);
            living.m_moveStrafing.set(0.0);
        }
    }
}

fn setEntityDead(this: &Rc<Entity>)
{
    let i: i32 = getSlimeSize(this);
    let world: Rc<World> = this.world();
    if !world.m_multiplayerWorld.get() && i > 1 && entity_living::fields(this).m_health.get() == 0
    {
        for j in 0..4
        {
            let f: f32 = ((j % 2) as f32 - 0.5) * i as f32 / 4.0;
            let f1: f32 = ((j / 2) as f32 - 0.5) * i as f32 / 4.0;
            let entityslime: Rc<Entity> = new(Some(Rc::clone(&world)));
            setSlimeSize(&entityslime, i / 2);
            entityslime.setLocationAndAngles(
                this.posX() + f as f64,
                this.posY() + 0.5,
                this.posZ() + f1 as f64,
                this.m_rand.nextFloat() * 360.0,
                0.0,
            );
            world.entityJoinedWorld(&entityslime);
        }
    }
    (entity_living::CLASS.setEntityDead)(this);
}

fn onCollideWithPlayer(this: &Rc<Entity>, player: &Rc<Entity>)
{
    let i: i32 = getSlimeSize(this);
    if i > 1
        && entity_living::canEntityBeSeen(this, player)
        && (this.getDistanceToEntity(player) as f64) < 0.6 * i as f64
        && player.attackEntityFrom(Some(this), i)
    {
        let pitch: f32 = (this.m_rand.nextFloat() - this.m_rand.nextFloat()) * 0.2 + 1.0;
        this.world().playSoundAtEntity(this, "mob.slimeattack", 1.0, pitch);
    }
}

fn getCanSpawnHere(this: &Rc<Entity>) -> bool
{
    let world: Rc<World> = this.world();
    let chunk: Rc<Chunk> =
        world.getChunkFromBlockCoords(math_helper::floor_double(this.posX()), math_helper::floor_double(this.posZ()));
    (getSlimeSize(this) == 1 || world.m_difficultySetting.get() > 0)
        && this.m_rand.nextIntBounded(10) == 0
        && chunk.getRandomForSlime(SLIME_CHUNK_SEED).nextIntBounded(10) == 0
        && this.posY() < 16.0
}

pub const CLASS: EntityClass = EntityClass {
    m_name: "EntitySlime",
    m_parent: Some(&entity_living::CLASS),
    m_interfaces: &["IMob", "IAnimals"],
    entityInit: |this| {
        (entity_living::CLASS.entityInit)(this);
        this.m_dataWatcher.addObject(16, WatchedValue::Byte(1));
    },
    writeEntityToNBT: |this, nbt| {
        (entity_living::CLASS.writeEntityToNBT)(this, nbt);
        nbt.setInteger("Size", getSlimeSize(this) - 1);
    },
    readEntityFromNBT: |this, nbt| {
        (entity_living::CLASS.readEntityFromNBT)(this, nbt);
        setSlimeSize(this, nbt.getInteger("Size") + 1);
    },
    onUpdate,
    updatePlayerActionState,
    setEntityDead,
    onCollideWithPlayer,
    getHurtSound: |_| Some("mob.slime".to_string()),
    getDeathSound: |_| Some("mob.slime".to_string()),
    getDropItemId: |this| {
        if getSlimeSize(this) == 1
        {
            return Item::slimeBall().m_shiftedIndex;
        }
        0
    },
    getCanSpawnHere,
    getSoundVolume: |_| 0.6,
    ..entity_living::CLASS
};

pub fn new(world: Option<Rc<World>>) -> Rc<Entity>
{
    let entity: Rc<Entity> = Entity::alloc(&CLASS, Box::new(SlimeExt::default()));
    entity_living::init(&entity, world);
    fields(&entity).m_slimeJumpDelay.set(0);
    entity_living::fields(&entity).m_texture.set(Some(JString::from("/mob/slime.png")));
    let i: i32 = 1 << entity.m_rand.nextIntBounded(3);
    entity.m_yOffset.set(0.0);
    fields(&entity).m_slimeJumpDelay.set(entity.m_rand.nextIntBounded(20) + 10);
    setSlimeSize(&entity, i);
    entity
}
