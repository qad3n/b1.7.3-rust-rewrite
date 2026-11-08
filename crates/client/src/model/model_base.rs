use crate::model::ModelBiped;
use game::entity::Entity;
use std::cell::Cell;
use std::rc::Rc;

#[derive(Default)]
pub struct ModelBaseFields
{
    pub m_onGround: Cell<f32>,
    pub m_isRiding: Cell<bool>,
}

pub trait ModelBase
{
    fn base(&self) -> &ModelBaseFields;

    fn render(&self, _walk: f32, _walkSpeed: f32, _bob: f32, _yaw: f32, _pitch: f32, _scale: f32) {}

    fn setRotationAngles(&self, _walk: f32, _walkSpeed: f32, _bob: f32, _yaw: f32, _pitch: f32, _scale: f32) {}

    fn setLivingAnimations(&self, _mob: &Rc<Entity>, _walk: f32, _walkSpeed: f32, _tickDelta: f32) {}

    fn biped(&self) -> Option<&ModelBiped> { None }
}
