use crate::model::{ModelBase, ModelBaseFields, ModelRenderer};
use game::math::math_helper;
use std::f32::consts::PI;

const DEGREES_PER_RADIAN: f32 = 57.295776;

pub struct ModelCreeper
{
    m_base: ModelBaseFields,
    pub m_head: ModelRenderer,
    pub m_hat: ModelRenderer,
    pub m_body: ModelRenderer,
    pub m_leg1: ModelRenderer,
    pub m_leg2: ModelRenderer,
    pub m_leg3: ModelRenderer,
    pub m_leg4: ModelRenderer,
}

impl ModelCreeper
{
    pub fn new(reduction: f32) -> ModelCreeper
    {
        let n: f32 = 4.0;
        let leg = |x: f32, z: f32| -> ModelRenderer {
            ModelRenderer::new(0, 16).withBox(-2.0, 0.0, -2.0, 4, 6, 4, reduction).at(x, 12.0 + n, z)
        };
        ModelCreeper {
            m_base: ModelBaseFields::default(),
            m_head: ModelRenderer::new(0, 0).withBox(-4.0, -8.0, -4.0, 8, 8, 8, reduction).at(0.0, n, 0.0),
            m_hat: ModelRenderer::new(32, 0).withBox(-4.0, -8.0, -4.0, 8, 8, 8, reduction + 0.5).at(0.0, n, 0.0),
            m_body: ModelRenderer::new(16, 16).withBox(-4.0, 0.0, -2.0, 8, 12, 4, reduction).at(0.0, n, 0.0),
            m_leg1: leg(-2.0, 4.0),
            m_leg2: leg(2.0, 4.0),
            m_leg3: leg(-2.0, -4.0),
            m_leg4: leg(2.0, -4.0),
        }
    }
}

impl ModelBase for ModelCreeper
{
    fn base(&self) -> &ModelBaseFields { &self.m_base }

    fn render(&self, walk: f32, walkSpeed: f32, bob: f32, yaw: f32, pitch: f32, scale: f32)
    {
        self.setRotationAngles(walk, walkSpeed, bob, yaw, pitch, scale);
        for part in [&self.m_head, &self.m_body, &self.m_leg1, &self.m_leg2, &self.m_leg3, &self.m_leg4]
        {
            part.render(scale);
        }
    }

    fn setRotationAngles(&self, walk: f32, walkSpeed: f32, _bob: f32, yaw: f32, pitch: f32, _scale: f32)
    {
        self.m_head.m_rotateAngleY.set(yaw / DEGREES_PER_RADIAN);
        self.m_head.m_rotateAngleX.set(pitch / DEGREES_PER_RADIAN);
        self.m_leg1.m_rotateAngleX.set(math_helper::cos(walk * 0.6662) * 1.4 * walkSpeed);
        self.m_leg2.m_rotateAngleX.set(math_helper::cos(walk * 0.6662 + PI) * 1.4 * walkSpeed);
        self.m_leg3.m_rotateAngleX.set(math_helper::cos(walk * 0.6662 + PI) * 1.4 * walkSpeed);
        self.m_leg4.m_rotateAngleX.set(math_helper::cos(walk * 0.6662) * 1.4 * walkSpeed);
    }
}
