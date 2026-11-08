use crate::model::{ModelBase, ModelBaseFields, ModelRenderer};
use game::math::math_helper;
use std::f32::consts::PI;

const DEGREES_PER_RADIAN: f32 = 57.295776;

pub struct ModelQuadruped
{
    pub m_base: ModelBaseFields,
    pub m_head: ModelRenderer,
    pub m_body: ModelRenderer,
    pub m_leg1: ModelRenderer,
    pub m_leg2: ModelRenderer,
    pub m_leg3: ModelRenderer,
    pub m_leg4: ModelRenderer,
}

fn leg(x: f32, z: f32, pivotPoint: i32, reduction: f32) -> ModelRenderer
{
    ModelRenderer::new(0, 16).withBox(-2.0, 0.0, -2.0, 4, pivotPoint, 4, reduction).at(x, (24 - pivotPoint) as f32, z)
}

impl ModelQuadruped
{
    pub fn new(pivotPoint: i32, reduction: f32) -> ModelQuadruped
    {
        ModelQuadruped {
            m_base: ModelBaseFields::default(),
            m_head: ModelRenderer::new(0, 0).withBox(-4.0, -4.0, -8.0, 8, 8, 8, reduction).at(
                0.0,
                (18 - pivotPoint) as f32,
                -6.0,
            ),
            m_body: ModelRenderer::new(28, 8).withBox(-5.0, -10.0, -7.0, 10, 16, 8, reduction).at(
                0.0,
                (17 - pivotPoint) as f32,
                2.0,
            ),
            m_leg1: leg(-3.0, 7.0, pivotPoint, reduction),
            m_leg2: leg(3.0, 7.0, pivotPoint, reduction),
            m_leg3: leg(-3.0, -5.0, pivotPoint, reduction),
            m_leg4: leg(3.0, -5.0, pivotPoint, reduction),
        }
    }

    pub fn renderParts(
        &self,
        model: &dyn ModelBase,
        walk: f32,
        walkSpeed: f32,
        bob: f32,
        yaw: f32,
        pitch: f32,
        scale: f32,
    )
    {
        model.setRotationAngles(walk, walkSpeed, bob, yaw, pitch, scale);
        self.m_head.render(scale);
        self.m_body.render(scale);
        self.m_leg1.render(scale);
        self.m_leg2.render(scale);
        self.m_leg3.render(scale);
        self.m_leg4.render(scale);
    }

    pub fn quadrupedRotationAngles(&self, walk: f32, walkSpeed: f32, yaw: f32, pitch: f32)
    {
        self.m_head.m_rotateAngleX.set(pitch / DEGREES_PER_RADIAN);
        self.m_head.m_rotateAngleY.set(yaw / DEGREES_PER_RADIAN);
        self.m_body.m_rotateAngleX.set(1.5707964);
        self.m_leg1.m_rotateAngleX.set(math_helper::cos(walk * 0.6662) * 1.4 * walkSpeed);
        self.m_leg2.m_rotateAngleX.set(math_helper::cos(walk * 0.6662 + PI) * 1.4 * walkSpeed);
        self.m_leg3.m_rotateAngleX.set(math_helper::cos(walk * 0.6662 + PI) * 1.4 * walkSpeed);
        self.m_leg4.m_rotateAngleX.set(math_helper::cos(walk * 0.6662) * 1.4 * walkSpeed);
    }
}

impl ModelBase for ModelQuadruped
{
    fn base(&self) -> &ModelBaseFields { &self.m_base }

    fn render(&self, walk: f32, walkSpeed: f32, bob: f32, yaw: f32, pitch: f32, scale: f32)
    {
        self.renderParts(self, walk, walkSpeed, bob, yaw, pitch, scale);
    }

    fn setRotationAngles(&self, walk: f32, walkSpeed: f32, _bob: f32, yaw: f32, pitch: f32, _scale: f32)
    {
        self.quadrupedRotationAngles(walk, walkSpeed, yaw, pitch);
    }
}
