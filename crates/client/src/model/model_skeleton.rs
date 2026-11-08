use crate::model::{ModelRenderer, ModelZombie};

pub struct ModelSkeleton;

impl ModelSkeleton
{
    pub fn new() -> ModelZombie
    {
        let f: f32 = 0.0;
        let mut zombie: ModelZombie = ModelZombie::new();
        let biped = &mut zombie.m_biped;
        biped.m_bipedRightArm = ModelRenderer::new(40, 16).withBox(-1.0, -2.0, -1.0, 2, 12, 2, f).at(-5.0, 2.0, 0.0);
        biped.m_bipedLeftArm = ModelRenderer::mirrored(40, 16).withBox(-1.0, -2.0, -1.0, 2, 12, 2, f).at(5.0, 2.0, 0.0);
        biped.m_bipedRightLeg = ModelRenderer::new(0, 16).withBox(-1.0, 0.0, -1.0, 2, 12, 2, f).at(-2.0, 12.0, 0.0);
        biped.m_bipedLeftLeg = ModelRenderer::mirrored(0, 16).withBox(-1.0, 0.0, -1.0, 2, 12, 2, f).at(2.0, 12.0, 0.0);
        zombie
    }
}
