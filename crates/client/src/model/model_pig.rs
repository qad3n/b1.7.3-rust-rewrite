use crate::model::ModelQuadruped;

pub struct ModelPig;

impl ModelPig
{
    pub fn new(reduction: f32) -> ModelQuadruped { ModelQuadruped::new(6, reduction) }
}
