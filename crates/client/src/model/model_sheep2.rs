use crate::model::{ModelQuadruped, ModelRenderer};

pub struct ModelSheep2;

impl ModelSheep2
{
    pub fn new() -> ModelQuadruped
    {
        let mut model: ModelQuadruped = ModelQuadruped::new(12, 0.0);
        model.m_head = ModelRenderer::new(0, 0).withBox(-3.0, -4.0, -6.0, 6, 6, 8, 0.0).at(0.0, 6.0, -8.0);
        model.m_body = ModelRenderer::new(28, 8).withBox(-4.0, -10.0, -7.0, 8, 16, 6, 0.0).at(0.0, 5.0, 2.0);
        model
    }
}
