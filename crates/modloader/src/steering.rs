use game::entity::{Entity, entity_living};
use mod_protocol::spec::SteeringSpec;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

mod physics
{
    // EntityLiving.moveEntityWithHeading ground acceleration numerator and air drag
    pub const GROUND_ACCELERATION: f32 = 0.16277136;
    pub const GRAVITY: f64 = 0.08;
    pub const AIR_DRAG: f64 = 0.98;
    // EntityMountable: a mount turns its speed into air control at six tenths
    pub const AIR_CONTROL: f32 = 0.6;
    // backing up is a quarter as fast as going forward
    pub const REVERSE: f32 = 0.25;
    pub const WALK_ANIMATION: f32 = 4.0;
    pub const WALK_BLEND: f32 = 0.4;
}

#[derive(Default, Clone, Copy)]
struct Flight
{
    m_jumpsLeft: i32,
    m_wasJumping: bool,
    m_maxJumps: Option<i32>,
    m_speed: Option<f32>,
}

thread_local! {
    static FLIGHTS: RefCell<HashMap<i32, Flight>> = RefCell::new(HashMap::new());
}

pub fn airJumps(entity: &Entity) -> i32
{
    FLIGHTS.with(|flights| flights.borrow().get(&entity.m_entityId.get()).map_or(0, |flight| flight.m_jumpsLeft))
}

pub fn speed(entity: &Entity) -> f32
{
    FLIGHTS
        .with(|flights| flights.borrow().get(&entity.m_entityId.get()).and_then(|flight| flight.m_speed).unwrap_or(0.0))
}

pub fn setSpeed(entity: &Entity, speed: f32)
{
    FLIGHTS.with(|flights| flights.borrow_mut().entry(entity.m_entityId.get()).or_default().m_speed = Some(speed));
}

pub fn setAirJumps(entity: &Entity, jumps: i32)
{
    FLIGHTS.with(|flights| {
        let mut flights = flights.borrow_mut();
        let flight: &mut Flight = flights.entry(entity.m_entityId.get()).or_default();
        flight.m_maxJumps = Some(jumps);
        flight.m_jumpsLeft = flight.m_jumpsLeft.min(jumps);
    });
}

fn riderOf(mount: &Rc<Entity>) -> Option<Rc<Entity>>
{
    mount.m_riddenByEntity.get().filter(|rider| rider.instanceOf("EntityPlayer"))
}

// EntityMountable.moveEntityWithHeading for a player ridden mount, false leaves the move to the template
pub fn travel(mount: &Rc<Entity>, steering: &SteeringSpec) -> bool
{
    let Some(rider) = riderOf(mount)
    else
    {
        return false;
    };
    if mount.isInWater() || mount.handleLavaMovement()
    {
        return false;
    }
    let input: &entity_living::LivingFields = entity_living::fields(&rider);
    let onGround: bool = mount.m_onGround.get();
    if steering.sneak_dismount && onGround && rider.isSneaking()
    {
        rider.mountEntity(None);
        return false;
    }
    mount.m_rotationYaw.set(rider.m_rotationYaw.get());
    mount.m_prevRotationYaw.set(rider.m_rotationYaw.get());
    mount.m_rotationPitch.set(rider.m_rotationPitch.get());
    mount.m_stepHeight.set(steering.step_height);
    let strafe: f32 = input.m_moveStrafing.get();
    let forward: f32 = if input.m_moveForward.get() <= 0.0
    {
        input.m_moveForward.get() * physics::REVERSE
    }
    else
    {
        input.m_moveForward.get()
    };
    let id: i32 = mount.m_entityId.get();
    let jumping: bool = input.m_isJumping.get();
    let mut flight: Flight = FLIGHTS.with(|flights| flights.borrow().get(&id).copied()).unwrap_or_default();
    if onGround
    {
        flight.m_jumpsLeft = flight.m_maxJumps.unwrap_or(steering.air_jumps);
    }
    let airReady: bool = flight.m_jumpsLeft > 0 && (!steering.air_jump_falling || mount.m_motionY.get() < 0.0);
    if jumping && !flight.m_wasJumping && (onGround || airReady)
    {
        let strength: f32 = if onGround || steering.air_jump == 0.0
        {
            steering.jump
        }
        else
        {
            steering.air_jump
        };
        if !onGround
        {
            flight.m_jumpsLeft -= 1;
        }
        mount.m_motionY.set(strength as f64);
    }
    flight.m_wasJumping = jumping;
    FLIGHTS.with(|flights| flights.borrow_mut().insert(id, flight));
    if mount.m_motionY.get() < 0.0 && !rider.isSneaking()
    {
        mount.m_motionY.set(mount.m_motionY.get() * steering.glide as f64);
    }
    let world = mount.world();
    let slipperiness: f32 = entity_living::groundSlipperiness(mount, &world);
    let speed: f32 = flight.m_speed.unwrap_or(steering.speed);
    let factor: f32 = if onGround
    {
        speed * physics::GROUND_ACCELERATION / (slipperiness * slipperiness * slipperiness)
    }
    else
    {
        speed * physics::AIR_CONTROL
    };
    mount.moveFlying(strafe, forward, factor);
    let slipperiness: f32 = entity_living::groundSlipperiness(mount, &world);
    mount.moveEntity(mount.m_motionX.get(), mount.m_motionY.get(), mount.m_motionZ.get());
    mount.m_motionY.set((mount.m_motionY.get() - physics::GRAVITY) * physics::AIR_DRAG);
    mount.m_motionX.set(mount.m_motionX.get() * slipperiness as f64);
    mount.m_motionZ.set(mount.m_motionZ.get() * slipperiness as f64);
    let living: &entity_living::LivingFields = entity_living::fields(mount);
    living.m_lastWalkAnimationSpeed.set(living.m_walkAnimationSpeed.get());
    let dx: f64 = mount.posX() - mount.m_prevPosX.get();
    let dz: f64 = mount.posZ() - mount.m_prevPosZ.get();
    let speed: f32 = (((dx * dx + dz * dz).sqrt() as f32) * physics::WALK_ANIMATION).min(1.0);
    living
        .m_walkAnimationSpeed
        .set(living.m_walkAnimationSpeed.get() + (speed - living.m_walkAnimationSpeed.get()) * physics::WALK_BLEND);
    living.m_walkAnimationProgress.set(living.m_walkAnimationProgress.get() + living.m_walkAnimationSpeed.get());
    true
}
