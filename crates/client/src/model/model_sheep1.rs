use crate::model::{ModelQuadruped, ModelRenderer};

pub struct ModelSheep1;

impl ModelSheep1
{
    pub fn new() -> ModelQuadruped
    {
        let mut model: ModelQuadruped = ModelQuadruped::new(12, 0.0);
        model.m_head = ModelRenderer::new(0, 0).withBox(-3.0, -4.0, -4.0, 6, 6, 6, 0.6).at(0.0, 6.0, -8.0);
        model.m_body = ModelRenderer::new(28, 8).withBox(-4.0, -10.0, -7.0, 8, 16, 6, 1.75).at(0.0, 5.0, 2.0);
        let f: f32 = 0.5;
        let leg = |x: f32, z: f32| -> ModelRenderer {
            ModelRenderer::new(0, 16).withBox(-2.0, 0.0, -2.0, 4, 6, 4, f).at(x, 12.0, z)
        };
        model.m_leg1 = leg(-3.0, 7.0);
        model.m_leg2 = leg(3.0, 7.0);
        model.m_leg3 = leg(-3.0, -5.0);
        model.m_leg4 = leg(3.0, -5.0);
        model
    }
}
