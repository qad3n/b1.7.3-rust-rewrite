use crate::model::{ModelBase, ModelBaseFields, ModelBiped};
use game::math::math_helper;
use std::f32::consts::PI;

pub struct ModelZombie
{
    pub m_biped: ModelBiped,
}

impl ModelZombie
{
    pub fn new() -> ModelZombie { ModelZombie { m_biped: ModelBiped::new(0.0, 0.0) } }
}

impl ModelBase for ModelZombie
{
    fn base(&self) -> &ModelBaseFields { &self.m_biped.m_base }

    fn render(&self, walk: f32, walkSpeed: f32, bob: f32, yaw: f32, pitch: f32, scale: f32)
    {
        self.m_biped.renderParts(self, walk, walkSpeed, bob, yaw, pitch, scale);
    }

    fn setRotationAngles(&self, walk: f32, walkSpeed: f32, bob: f32, yaw: f32, pitch: f32, _scale: f32)
    {
        let biped: &ModelBiped = &self.m_biped;
        biped.bipedRotationAngles(walk, walkSpeed, bob, yaw, pitch);
        let onGround: f32 = biped.m_base.m_onGround.get();
        let f: f32 = math_helper::sin(onGround * PI);
        let f2: f32 = math_helper::sin((1.0 - (1.0 - onGround) * (1.0 - onGround)) * PI);
        let rightArm = &biped.m_bipedRightArm;
        let leftArm = &biped.m_bipedLeftArm;
        rightArm.m_rotateAngleZ.set(0.0);
        leftArm.m_rotateAngleZ.set(0.0);
        rightArm.m_rotateAngleY.set(-(0.1 - f * 0.6));
        leftArm.m_rotateAngleY.set(0.1 - f * 0.6);
        rightArm.m_rotateAngleX.set(-1.5707964);
        leftArm.m_rotateAngleX.set(-1.5707964);
        rightArm.m_rotateAngleX.set(rightArm.m_rotateAngleX.get() - (f * 1.2 - f2 * 0.4));
        leftArm.m_rotateAngleX.set(leftArm.m_rotateAngleX.get() - (f * 1.2 - f2 * 0.4));
        biped.armBob(bob);
    }

    fn biped(&self) -> Option<&ModelBiped> { Some(&self.m_biped) }
}
