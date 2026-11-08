use crate::model::{ModelBase, ModelBaseFields, ModelRenderer};
use std::f64::consts::PI;

const TENTACLES: usize = 8;

pub struct ModelSquid
{
    m_base: ModelBaseFields,
    m_squidBody: ModelRenderer,
    m_squidTentacles: Vec<ModelRenderer>,
}

impl ModelSquid
{
    pub fn new() -> ModelSquid
    {
        let n: i32 = -16;
        let body: ModelRenderer = ModelRenderer::new(0, 0).withBox(-6.0, -8.0, -6.0, 12, 16, 12, 0.0);
        body.m_rotationPointY.set(body.m_rotationPointY.get() + (24 + n) as f32);
        let tentacles: Vec<ModelRenderer> = (0..TENTACLES)
            .map(|i| {
                let mut d: f64 = i as f64 * PI * 2.0 / TENTACLES as f64;
                let f: f32 = jvm::math::cos(d) as f32 * 5.0;
                let f2: f32 = jvm::math::sin(d) as f32 * 5.0;
                let tentacle: ModelRenderer = ModelRenderer::new(48, 0).withBox(-1.0, 0.0, -1.0, 2, 18, 2, 0.0);
                tentacle.m_rotationPointX.set(f);
                tentacle.m_rotationPointZ.set(f2);
                tentacle.m_rotationPointY.set((31 + n) as f32);
                d = i as f64 * PI * -2.0 / TENTACLES as f64 + 1.5707963267948966;
                tentacle.m_rotateAngleY.set(d as f32);
                tentacle
            })
            .collect();
        ModelSquid { m_base: ModelBaseFields::default(), m_squidBody: body, m_squidTentacles: tentacles }
    }
}

impl ModelBase for ModelSquid
{
    fn base(&self) -> &ModelBaseFields { &self.m_base }

    fn setRotationAngles(&self, _walk: f32, _walkSpeed: f32, bob: f32, _yaw: f32, _pitch: f32, _scale: f32)
    {
        for tentacle in &self.m_squidTentacles
        {
            tentacle.m_rotateAngleX.set(bob);
        }
    }

    fn render(&self, walk: f32, walkSpeed: f32, bob: f32, yaw: f32, pitch: f32, scale: f32)
    {
        self.setRotationAngles(walk, walkSpeed, bob, yaw, pitch, scale);
        self.m_squidBody.render(scale);
        for tentacle in &self.m_squidTentacles
        {
            tentacle.render(scale);
        }
    }
}
