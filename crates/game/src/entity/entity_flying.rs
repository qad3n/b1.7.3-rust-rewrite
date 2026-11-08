use crate::entity::{Entity, EntityClass, entity_living};
use crate::math::math_helper;
use crate::world::World;
use std::rc::Rc;

pub fn init(this: &Rc<Entity>, world: Option<Rc<World>>) { entity_living::init(this, world); }

fn moveEntityWithHeading(this: &Rc<Entity>, sideways: f32, forwards: f32)
{
    let living: &entity_living::LivingFields = entity_living::fields(this);
    let world: Rc<World> = this.world();
    if this.isInWater()
    {
        this.moveFlying(sideways, forwards, 0.02);
        this.moveEntity(this.m_motionX.get(), this.m_motionY.get(), this.m_motionZ.get());
        this.m_motionX.set(this.m_motionX.get() * 0.8f32 as f64);
        this.m_motionY.set(this.m_motionY.get() * 0.8f32 as f64);
        this.m_motionZ.set(this.m_motionZ.get() * 0.8f32 as f64);
    }
    else if this.handleLavaMovement()
    {
        this.moveFlying(sideways, forwards, 0.02);
        this.moveEntity(this.m_motionX.get(), this.m_motionY.get(), this.m_motionZ.get());
        this.m_motionX.set(this.m_motionX.get() * 0.5);
        this.m_motionY.set(this.m_motionY.get() * 0.5);
        this.m_motionZ.set(this.m_motionZ.get() * 0.5);
    }
    else
    {
        let mut f: f32 = entity_living::update::groundSlipperiness(this, &world);
        let f2: f32 = 0.16277136 / (f * f * f);
        this.moveFlying(
            sideways,
            forwards,
            if this.m_onGround.get()
            {
                0.1 * f2
            }
            else
            {
                0.02
            },
        );
        f = entity_living::update::groundSlipperiness(this, &world);
        this.moveEntity(this.m_motionX.get(), this.m_motionY.get(), this.m_motionZ.get());
        this.m_motionX.set(this.m_motionX.get() * f as f64);
        this.m_motionY.set(this.m_motionY.get() * f as f64);
        this.m_motionZ.set(this.m_motionZ.get() * f as f64);
    }
    living.m_lastWalkAnimationSpeed.set(living.m_walkAnimationSpeed.get());
    let d: f64 = this.posX() - this.m_prevPosX.get();
    let d2: f64 = this.posZ() - this.m_prevPosZ.get();
    let mut f: f32 = math_helper::sqrt_double(d * d + d2 * d2) * 4.0;
    if f > 1.0
    {
        f = 1.0;
    }
    living.m_walkAnimationSpeed.set(living.m_walkAnimationSpeed.get() + (f - living.m_walkAnimationSpeed.get()) * 0.4);
    living.m_walkAnimationProgress.set(living.m_walkAnimationProgress.get() + living.m_walkAnimationSpeed.get());
}

pub const CLASS: EntityClass = EntityClass {
    m_name: "EntityFlying",
    m_parent: Some(&entity_living::CLASS),
    fall: |_, _| {},
    moveEntityWithHeading,
    isOnLadder: |_| false,
    ..entity_living::CLASS
};
