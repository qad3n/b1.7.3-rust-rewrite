use crate::model::{ModelBase, ModelBaseFields, ModelRenderer};
use game::math::math_helper;
use std::f32::consts::PI;

const DEGREES_PER_RADIAN: f32 = 57.295776;

pub struct ModelChicken
{
    m_base: ModelBaseFields,
    pub m_head: ModelRenderer,
    pub m_body: ModelRenderer,
    pub m_rightLeg: ModelRenderer,
    pub m_leftLeg: ModelRenderer,
    pub m_rightWing: ModelRenderer,
    pub m_leftWing: ModelRenderer,
    pub m_bill: ModelRenderer,
    pub m_chin: ModelRenderer,
}

impl ModelChicken
{
    pub fn new() -> ModelChicken
    {
        let n: f32 = 16.0;
        ModelChicken {
            m_base: ModelBaseFields::default(),
            m_head: ModelRenderer::new(0, 0).withBox(-2.0, -6.0, -2.0, 4, 6, 3, 0.0).at(0.0, -1.0 + n, -4.0),
            m_bill: ModelRenderer::new(14, 0).withBox(-2.0, -4.0, -4.0, 4, 2, 2, 0.0).at(0.0, -1.0 + n, -4.0),
            m_chin: ModelRenderer::new(14, 4).withBox(-1.0, -2.0, -3.0, 2, 2, 2, 0.0).at(0.0, -1.0 + n, -4.0),
            m_body: ModelRenderer::new(0, 9).withBox(-3.0, -4.0, -3.0, 6, 8, 6, 0.0).at(0.0, n, 0.0),
            m_rightLeg: ModelRenderer::new(26, 0).withBox(-1.0, 0.0, -3.0, 3, 5, 3, 0.0).at(-2.0, 3.0 + n, 1.0),
            m_leftLeg: ModelRenderer::new(26, 0).withBox(-1.0, 0.0, -3.0, 3, 5, 3, 0.0).at(1.0, 3.0 + n, 1.0),
            m_rightWing: ModelRenderer::new(24, 13).withBox(0.0, 0.0, -3.0, 1, 4, 6, 0.0).at(-4.0, -3.0 + n, 0.0),
            m_leftWing: ModelRenderer::new(24, 13).withBox(-1.0, 0.0, -3.0, 1, 4, 6, 0.0).at(4.0, -3.0 + n, 0.0),
        }
    }
}

impl ModelBase for ModelChicken
{
    fn base(&self) -> &ModelBaseFields { &self.m_base }

    fn render(&self, walk: f32, walkSpeed: f32, bob: f32, yaw: f32, pitch: f32, scale: f32)
    {
        self.setRotationAngles(walk, walkSpeed, bob, yaw, pitch, scale);
        for part in [
            &self.m_head,
            &self.m_bill,
            &self.m_chin,
            &self.m_body,
            &self.m_rightLeg,
            &self.m_leftLeg,
            &self.m_rightWing,
            &self.m_leftWing,
        ]
        {
            part.render(scale);
        }
    }

    fn setRotationAngles(&self, walk: f32, walkSpeed: f32, bob: f32, yaw: f32, pitch: f32, _scale: f32)
    {
        self.m_head.m_rotateAngleX.set(-(pitch / DEGREES_PER_RADIAN));
        self.m_head.m_rotateAngleY.set(yaw / DEGREES_PER_RADIAN);
        for part in [&self.m_bill, &self.m_chin]
        {
            part.m_rotateAngleX.set(self.m_head.m_rotateAngleX.get());
            part.m_rotateAngleY.set(self.m_head.m_rotateAngleY.get());
        }
        self.m_body.m_rotateAngleX.set(1.5707964);
        self.m_rightLeg.m_rotateAngleX.set(math_helper::cos(walk * 0.6662) * 1.4 * walkSpeed);
        self.m_leftLeg.m_rotateAngleX.set(math_helper::cos(walk * 0.6662 + PI) * 1.4 * walkSpeed);
        self.m_rightWing.m_rotateAngleZ.set(bob);
        self.m_leftWing.m_rotateAngleZ.set(-bob);
    }
}
