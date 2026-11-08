use super::{LivingFields, fields};
use crate::block::{Block, StepSound};
use crate::entity::{ENTITY, Entity};
use crate::math::math_helper;
use crate::port::playSoundAtEntity;
use crate::world::World;
use jvm::math;
use jvm::throwable::OrNpe;
use std::rc::Rc;

fn randomPitch(this: &Entity) -> f32 { (this.m_rand.nextFloat() - this.m_rand.nextFloat()) * 0.2 + 1.0 }

pub fn playLivingSound(this: &Rc<Entity>)
{
    if let Some(s) = this.getLivingSound()
    {
        this.world().playSoundAtEntity(this, &s, this.getSoundVolume(), randomPitch(this));
    }
}

pub fn spawnExplosionParticle(this: &Rc<Entity>)
{
    let world: Rc<World> = this.world();
    for _ in 0..20
    {
        let d0: f64 = this.m_rand.nextGaussian() * 0.02;
        let d1: f64 = this.m_rand.nextGaussian() * 0.02;
        let d2: f64 = this.m_rand.nextGaussian() * 0.02;
        let d3: f64 = 10.0;
        let width: f32 = this.m_width.get();
        let x: f64 = this.posX() + (this.m_rand.nextFloat() * width * 2.0) as f64 - width as f64 - d0 * d3;
        let y: f64 = this.posY() + (this.m_rand.nextFloat() * this.m_height.get()) as f64 - d1 * d3;
        let z: f64 = this.posZ() + (this.m_rand.nextFloat() * width * 2.0) as f64 - width as f64 - d2 * d3;
        world.spawnParticle("explode", x, y, z, d0, d1, d2);
    }
}

pub fn heal(this: &Rc<Entity>, amount: i32)
{
    let living: &LivingFields = fields(this);
    if living.m_health.get() <= 0
    {
        return;
    }
    living.m_health.set(living.m_health.get() + amount);
    if living.m_health.get() > 20
    {
        living.m_health.set(20);
    }
    this.m_heartsLife.set(living.m_heartsHalvesLife.get() / 2);
}

pub fn attackEntityFrom(this: &Rc<Entity>, source: Option<&Rc<Entity>>, amount: i32) -> bool
{
    let living: &LivingFields = fields(this);
    let world: Rc<World> = this.world();
    if world.m_multiplayerWorld.get()
    {
        return false;
    }
    living.m_entityAge.set(0);
    if living.m_health.get() <= 0
    {
        return false;
    }
    living.m_walkAnimationSpeed.set(1.5);
    let mut flag: bool = true;
    if this.m_heartsLife.get() as f32 > living.m_heartsHalvesLife.get() as f32 / 2.0
    {
        if amount <= living.m_prevDamageTaken.get()
        {
            return false;
        }
        this.damageEntity(amount - living.m_prevDamageTaken.get());
        living.m_prevDamageTaken.set(amount);
        flag = false;
    }
    else
    {
        living.m_prevDamageTaken.set(amount);
        living.m_prevHealth.set(living.m_health.get());
        this.m_heartsLife.set(living.m_heartsHalvesLife.get());
        this.damageEntity(amount);
        living.m_maxHurtTime.set(10);
        living.m_hurtTime.set(10);
    }
    living.m_attackedAtYaw.set(0.0);
    if flag
    {
        world.doEntityEvent(this, 2);
        this.setBeenAttacked();
        if let Some(source) = source
        {
            let mut d0: f64 = source.posX() - this.posX();
            let mut d1: f64 = source.posZ() - this.posZ();
            while d0 * d0 + d1 * d1 < 1.0E-4
            {
                d0 = (math::random() - math::random()) * 0.01;
                d1 = (math::random() - math::random()) * 0.01;
            }
            living.m_attackedAtYaw.set(
                (math::atan2(d1, d0) * 180.0 / std::f64::consts::PI as f32 as f64) as f32 - this.m_rotationYaw.get(),
            );
            knockBack(this, source, amount, d0, d1);
        }
        else
        {
            living.m_attackedAtYaw.set(((math::random() * 2.0) as i32 * 180) as f32);
        }
    }
    if living.m_health.get() <= 0
    {
        if flag
        {
            // java may pass a null sound name so the port wrapper plays nothing for it
            playSoundAtEntity(&world, this, this.getDeathSound(), this.getSoundVolume(), randomPitch(this));
        }
        this.onDeath(source);
    }
    else if flag
    {
        // java may pass a null sound name so the port wrapper plays nothing for it
        playSoundAtEntity(&world, this, this.getHurtSound(), this.getSoundVolume(), randomPitch(this));
    }
    true
}

pub fn knockBack(this: &Rc<Entity>, _attacker: &Rc<Entity>, _amount: i32, dx: f64, dz: f64)
{
    let f: f32 = math_helper::sqrt_double(dx * dx + dz * dz);
    let f1: f32 = 0.4;
    this.m_motionX.set(this.m_motionX.get() / 2.0);
    this.m_motionY.set(this.m_motionY.get() / 2.0);
    this.m_motionZ.set(this.m_motionZ.get() / 2.0);
    this.m_motionX.set(this.m_motionX.get() - dx / f as f64 * f1 as f64);
    this.m_motionY.set(this.m_motionY.get() + 0.4f32 as f64);
    this.m_motionZ.set(this.m_motionZ.get() - dz / f as f64 * f1 as f64);
    if this.m_motionY.get() > 0.4f32 as f64
    {
        this.m_motionY.set(0.4f32 as f64);
    }
}

pub fn onDeath(this: &Rc<Entity>, killer: Option<&Rc<Entity>>)
{
    let living: &LivingFields = fields(this);
    if living.m_scoreValue.get() >= 0
        && let Some(killer) = killer
    {
        killer.addToPlayerScore(Some(this), living.m_scoreValue.get());
    }
    if let Some(killer) = killer
    {
        killer.onKillEntity(this);
    }
    living.m_unused_flag.set(true);
    if !this.world().m_multiplayerWorld.get()
    {
        this.dropFewItems();
    }
    this.world().doEntityEvent(this, 3);
}

pub fn dropFewItems(this: &Rc<Entity>)
{
    let i: i32 = this.getDropItemId();
    if i > 0
    {
        let j: i32 = this.m_rand.nextIntBounded(3);
        for _ in 0..j
        {
            this.dropItem(i, 1);
        }
    }
}

pub fn fall(this: &Rc<Entity>, distance: f32)
{
    (ENTITY.fall)(this, distance);
    let i: i32 = math::ceil((distance - 3.0) as f64) as i32;
    if i > 0
    {
        this.attackEntityFrom(None, i);
        let world: Rc<World> = this.world();
        let j: i32 = world.getBlockId(
            math_helper::floor_double(this.posX()),
            math_helper::floor_double(this.posY() - 0.2f32 as f64 - this.m_yOffset.get() as f64),
            math_helper::floor_double(this.posZ()),
        );
        if j > 0
        {
            let sound: Rc<StepSound> = Block::byId(j).orNpe().m_stepSound.get();
            world.playSoundAtEntity(this, &sound.getStepping(), sound.getVolume() * 0.5, sound.getPitch() * 0.75);
        }
    }
}

pub fn onEntityDeath(_this: &Rc<Entity>) {}

pub fn handleHealthUpdate(this: &Rc<Entity>, event: i8)
{
    let living: &LivingFields = fields(this);
    let world: Rc<World> = this.world();
    if event == 2
    {
        living.m_walkAnimationSpeed.set(1.5);
        this.m_heartsLife.set(living.m_heartsHalvesLife.get());
        living.m_maxHurtTime.set(10);
        living.m_hurtTime.set(10);
        living.m_attackedAtYaw.set(0.0);
        // java may pass a null sound name so the port wrapper plays nothing for it
        playSoundAtEntity(&world, this, this.getHurtSound(), this.getSoundVolume(), randomPitch(this));
        this.attackEntityFrom(None, 0);
    }
    else if event == 3
    {
        // java may pass a null sound name so the port wrapper plays nothing for it
        playSoundAtEntity(&world, this, this.getDeathSound(), this.getSoundVolume(), randomPitch(this));
        living.m_health.set(0);
        this.onDeath(None);
    }
    else
    {
        (ENTITY.handleHealthUpdate)(this, event);
    }
}
