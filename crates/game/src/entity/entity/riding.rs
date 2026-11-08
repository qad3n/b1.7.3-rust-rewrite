use crate::entity::Entity;
use jvm::throwable::OrNpe;
use std::rc::Rc;

pub fn updateRidden(this: &Rc<Entity>)
{
    let riding: Rc<Entity> = this.m_ridingEntity.get().orNpe();
    if riding.m_isDead.get()
    {
        this.m_ridingEntity.set(None);
        return;
    }

    this.m_motionX.set(0.0);
    this.m_motionY.set(0.0);
    this.m_motionZ.set(0.0);
    this.onUpdate();
    let Some(riding) = this.m_ridingEntity.get()
    else
    {
        return;
    };

    riding.updateRiderPosition();
    this.m_entityRiderYawDelta
        .set(this.m_entityRiderYawDelta.get() + (riding.m_rotationYaw.get() - riding.m_prevRotationYaw.get()) as f64);
    this.m_entityRiderPitchDelta.set(
        this.m_entityRiderPitchDelta.get() + (riding.m_rotationPitch.get() - riding.m_prevRotationPitch.get()) as f64,
    );
    while this.m_entityRiderYawDelta.get() >= 180.0
    {
        this.m_entityRiderYawDelta.set(this.m_entityRiderYawDelta.get() - 360.0);
    }
    while this.m_entityRiderYawDelta.get() < -180.0
    {
        this.m_entityRiderYawDelta.set(this.m_entityRiderYawDelta.get() + 360.0);
    }
    while this.m_entityRiderPitchDelta.get() >= 180.0
    {
        this.m_entityRiderPitchDelta.set(this.m_entityRiderPitchDelta.get() - 360.0);
    }
    while this.m_entityRiderPitchDelta.get() < -180.0
    {
        this.m_entityRiderPitchDelta.set(this.m_entityRiderPitchDelta.get() + 360.0);
    }

    let mut d0: f64 = this.m_entityRiderYawDelta.get() * 0.5;
    let mut d1: f64 = this.m_entityRiderPitchDelta.get() * 0.5;
    let f: f32 = 10.0;
    if d0 > f as f64
    {
        d0 = f as f64;
    }
    if d0 < -f as f64
    {
        d0 = -f as f64;
    }
    if d1 > f as f64
    {
        d1 = f as f64;
    }
    if d1 < -f as f64
    {
        d1 = -f as f64;
    }

    this.m_entityRiderYawDelta.set(this.m_entityRiderYawDelta.get() - d0);
    this.m_entityRiderPitchDelta.set(this.m_entityRiderPitchDelta.get() - d1);
    this.m_rotationYaw.set((this.m_rotationYaw.get() as f64 + d0) as f32);
    this.m_rotationPitch.set((this.m_rotationPitch.get() as f64 + d1) as f32);
}

pub fn updateRiderPosition(this: &Rc<Entity>)
{
    let rider: Rc<Entity> = this.m_riddenByEntity.get().orNpe();
    rider.setPosition(this.posX(), this.posY() + this.getMountedYOffset() + rider.getYOffset(), this.posZ());
}

pub fn getMountedYOffset(this: &Rc<Entity>) -> f64 { this.m_height.get() as f64 * 0.75 }

impl Entity {}

pub fn mountEntity(this: &Rc<Entity>, entity: Option<&Rc<Entity>>)
{
    this.m_entityRiderPitchDelta.set(0.0);
    this.m_entityRiderYawDelta.set(0.0);
    let Some(entity) = entity
    else
    {
        if let Some(riding) = this.m_ridingEntity.get()
        {
            this.setLocationAndAngles(
                riding.posX(),
                riding.m_boundingBox.minY() + riding.m_height.get() as f64,
                riding.posZ(),
                this.m_rotationYaw.get(),
                this.m_rotationPitch.get(),
            );
            riding.m_riddenByEntity.set(None);
        }
        this.m_ridingEntity.set(None);
        return;
    };

    if let Some(riding) = this.m_ridingEntity.get()
        && Rc::ptr_eq(&riding, entity)
    {
        riding.m_riddenByEntity.set(None);
        this.m_ridingEntity.set(None);
        this.setLocationAndAngles(
            entity.posX(),
            entity.m_boundingBox.minY() + entity.m_height.get() as f64,
            entity.posZ(),
            this.m_rotationYaw.get(),
            this.m_rotationPitch.get(),
        );
        return;
    }

    if let Some(riding) = this.m_ridingEntity.get()
    {
        riding.m_riddenByEntity.set(None);
    }
    if let Some(rider) = entity.m_riddenByEntity.get()
    {
        rider.m_ridingEntity.set(None);
    }
    this.m_ridingEntity.set(Some(Rc::clone(entity)));
    entity.m_riddenByEntity.set(Some(Rc::clone(this)));
}
