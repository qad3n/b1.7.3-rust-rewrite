use crate::model::{ModelBase, ModelBaseFields, ModelRenderer};

struct SlimeFace
{
    m_slimeRightEye: ModelRenderer,
    m_slimeLeftEye: ModelRenderer,
    m_slimeMouth: ModelRenderer,
}

pub struct ModelSlime
{
    m_base: ModelBaseFields,
    m_slimeBodies: ModelRenderer,
    m_face: Option<SlimeFace>,
}

impl ModelSlime
{
    pub fn new(v: i32) -> ModelSlime
    {
        let mut slimeBodies: ModelRenderer = ModelRenderer::new(0, v).withBox(-4.0, 16.0, -4.0, 8, 8, 8, 0.0);
        let mut face: Option<SlimeFace> = None;
        if v > 0
        {
            slimeBodies = ModelRenderer::new(0, v).withBox(-3.0, 17.0, -3.0, 6, 6, 6, 0.0);
            face = Some(SlimeFace {
                m_slimeRightEye: ModelRenderer::new(32, 0).withBox(-3.25, 18.0, -3.5, 2, 2, 2, 0.0),
                m_slimeLeftEye: ModelRenderer::new(32, 4).withBox(1.25, 18.0, -3.5, 2, 2, 2, 0.0),
                m_slimeMouth: ModelRenderer::new(32, 8).withBox(0.0, 21.0, -3.5, 1, 1, 1, 0.0),
            });
        }
        ModelSlime { m_base: ModelBaseFields::default(), m_slimeBodies: slimeBodies, m_face: face }
    }
}

impl ModelBase for ModelSlime
{
    fn base(&self) -> &ModelBaseFields { &self.m_base }

    fn render(&self, walk: f32, walkSpeed: f32, bob: f32, yaw: f32, pitch: f32, scale: f32)
    {
        self.setRotationAngles(walk, walkSpeed, bob, yaw, pitch, scale);
        self.m_slimeBodies.render(scale);
        if let Some(face) = &self.m_face
        {
            face.m_slimeRightEye.render(scale);
            face.m_slimeLeftEye.render(scale);
            face.m_slimeMouth.render(scale);
        }
    }
}
