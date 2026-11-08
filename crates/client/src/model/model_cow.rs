use crate::model::{ModelBase, ModelBaseFields, ModelQuadruped, ModelRenderer};

pub struct ModelCow
{
    m_quadruped: ModelQuadruped,
    m_udders: ModelRenderer,
    m_horn1: ModelRenderer,
    m_horn2: ModelRenderer,
}

impl ModelCow
{
    pub fn new() -> ModelCow
    {
        let mut quadruped: ModelQuadruped = ModelQuadruped::new(12, 0.0);
        quadruped.m_head = ModelRenderer::new(0, 0).withBox(-4.0, -4.0, -6.0, 8, 8, 6, 0.0).at(0.0, 4.0, -8.0);
        let horn1: ModelRenderer = ModelRenderer::new(22, 0).withBox(-4.0, -5.0, -4.0, 1, 3, 1, 0.0).at(0.0, 3.0, -7.0);
        let horn2: ModelRenderer = ModelRenderer::new(22, 0).withBox(3.0, -5.0, -4.0, 1, 3, 1, 0.0).at(0.0, 3.0, -7.0);
        let udders: ModelRenderer = ModelRenderer::new(52, 0).withBox(-2.0, -3.0, 0.0, 4, 6, 2, 0.0).at(0.0, 14.0, 6.0);
        udders.m_rotateAngleX.set(1.5707964);
        quadruped.m_body = ModelRenderer::new(18, 4).withBox(-6.0, -10.0, -7.0, 12, 18, 10, 0.0).at(0.0, 5.0, 2.0);
        let shift = |part: &ModelRenderer, dx: f32, dz: f32| {
            part.m_rotationPointX.set(part.m_rotationPointX.get() + dx);
            part.m_rotationPointZ.set(part.m_rotationPointZ.get() + dz);
        };
        shift(&quadruped.m_leg1, -1.0, 0.0);
        shift(&quadruped.m_leg2, 1.0, 0.0);
        shift(&quadruped.m_leg3, -1.0, -1.0);
        shift(&quadruped.m_leg4, 1.0, -1.0);
        ModelCow { m_quadruped: quadruped, m_udders: udders, m_horn1: horn1, m_horn2: horn2 }
    }
}

impl ModelBase for ModelCow
{
    fn base(&self) -> &ModelBaseFields { &self.m_quadruped.m_base }

    fn render(&self, walk: f32, walkSpeed: f32, bob: f32, yaw: f32, pitch: f32, scale: f32)
    {
        self.m_quadruped.renderParts(self, walk, walkSpeed, bob, yaw, pitch, scale);
        self.m_horn1.render(scale);
        self.m_horn2.render(scale);
        self.m_udders.render(scale);
    }

    fn setRotationAngles(&self, walk: f32, walkSpeed: f32, _bob: f32, yaw: f32, pitch: f32, _scale: f32)
    {
        self.m_quadruped.quadrupedRotationAngles(walk, walkSpeed, yaw, pitch);
        let head = &self.m_quadruped.m_head;
        for horn in [&self.m_horn1, &self.m_horn2]
        {
            horn.m_rotateAngleY.set(head.m_rotateAngleY.get());
            horn.m_rotateAngleX.set(head.m_rotateAngleX.get());
        }
    }
}
