use crate::model::{ModelBase, ModelBaseFields, ModelRenderer};
use game::math::math_helper;
use jvm::random::Random;

const TENTACLES: i32 = 9;
const TENTACLE_SEED: i64 = 1660;

pub struct ModelGhast
{
    m_base: ModelBaseFields,
    m_body: ModelRenderer,
    m_tentacles: Vec<ModelRenderer>,
}

impl ModelGhast
{
    pub fn new() -> ModelGhast
    {
        let n: i32 = -16;
        let body: ModelRenderer = ModelRenderer::new(0, 0).withBox(-8.0, -8.0, -8.0, 16, 16, 16, 0.0);
        body.m_rotationPointY.set(body.m_rotationPointY.get() + (24 + n) as f32);
        let random: Random = Random::withSeed(TENTACLE_SEED);
        let tentacles: Vec<ModelRenderer> = (0..TENTACLES)
            .map(|i| {
                let f: f32 = (((i % 3) as f32 - (i / 3 % 2) as f32 * 0.5 + 0.25) / 2.0 * 2.0 - 1.0) * 5.0;
                let f2: f32 = ((i / 3) as f32 / 2.0 * 2.0 - 1.0) * 5.0;
                let n2: i32 = random.nextIntBounded(7) + 8;
                let tentacle: ModelRenderer = ModelRenderer::new(0, 0).withBox(-1.0, 0.0, -1.0, 2, n2, 2, 0.0);
                tentacle.m_rotationPointX.set(f);
                tentacle.m_rotationPointZ.set(f2);
                tentacle.m_rotationPointY.set((31 + n) as f32);
                tentacle
            })
            .collect();
        ModelGhast { m_base: ModelBaseFields::default(), m_body: body, m_tentacles: tentacles }
    }
}

impl ModelBase for ModelGhast
{
    fn base(&self) -> &ModelBaseFields { &self.m_base }

    fn setRotationAngles(&self, _walk: f32, _walkSpeed: f32, bob: f32, _yaw: f32, _pitch: f32, _scale: f32)
    {
        for (i, tentacle) in self.m_tentacles.iter().enumerate()
        {
            tentacle.m_rotateAngleX.set(0.2 * math_helper::sin(bob * 0.3 + i as f32) + 0.4);
        }
    }

    fn render(&self, walk: f32, walkSpeed: f32, bob: f32, yaw: f32, pitch: f32, scale: f32)
    {
        self.setRotationAngles(walk, walkSpeed, bob, yaw, pitch, scale);
        self.m_body.render(scale);
        for tentacle in &self.m_tentacles
        {
            tentacle.render(scale);
        }
    }
}
