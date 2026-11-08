use crate::model::{ModelBase, ModelBaseFields, ModelRenderer};
use game::math::math_helper;
use std::f32::consts::PI;

const DEGREES_PER_RADIAN: f32 = 57.295776;

pub struct ModelSpider
{
    m_base: ModelBaseFields,
    pub m_spiderHead: ModelRenderer,
    pub m_spiderNeck: ModelRenderer,
    pub m_spiderBody: ModelRenderer,
    pub m_spiderLegs: Vec<ModelRenderer>,
}

impl ModelSpider
{
    pub fn new() -> ModelSpider
    {
        let f: f32 = 0.0;
        let n: f32 = 15.0;
        let legs: Vec<ModelRenderer> = [2.0, 1.0, 0.0, -1.0]
            .into_iter()
            .flat_map(|z: f32| {
                [
                    ModelRenderer::new(18, 0).withBox(-15.0, -1.0, -1.0, 16, 2, 2, f).at(-4.0, n, z),
                    ModelRenderer::new(18, 0).withBox(-1.0, -1.0, -1.0, 16, 2, 2, f).at(4.0, n, z),
                ]
            })
            .collect();
        ModelSpider {
            m_base: ModelBaseFields::default(),
            m_spiderHead: ModelRenderer::new(32, 4).withBox(-4.0, -4.0, -8.0, 8, 8, 8, f).at(0.0, n, -3.0),
            m_spiderNeck: ModelRenderer::new(0, 0).withBox(-3.0, -3.0, -3.0, 6, 6, 6, f).at(0.0, n, 0.0),
            m_spiderBody: ModelRenderer::new(0, 12).withBox(-5.0, -4.0, -6.0, 10, 8, 12, f).at(0.0, n, 9.0),
            m_spiderLegs: legs,
        }
    }
}

impl ModelBase for ModelSpider
{
    fn base(&self) -> &ModelBaseFields { &self.m_base }

    fn render(&self, walk: f32, walkSpeed: f32, bob: f32, yaw: f32, pitch: f32, scale: f32)
    {
        self.setRotationAngles(walk, walkSpeed, bob, yaw, pitch, scale);
        self.m_spiderHead.render(scale);
        self.m_spiderNeck.render(scale);
        self.m_spiderBody.render(scale);
        for leg in &self.m_spiderLegs
        {
            leg.render(scale);
        }
    }

    fn setRotationAngles(&self, walk: f32, walkSpeed: f32, _bob: f32, yaw: f32, pitch: f32, _scale: f32)
    {
        self.m_spiderHead.m_rotateAngleY.set(yaw / DEGREES_PER_RADIAN);
        self.m_spiderHead.m_rotateAngleX.set(pitch / DEGREES_PER_RADIAN);
        let f: f32 = 0.7853982;
        let f2: f32 = -0.0;
        let f3: f32 = 0.3926991;
        let pairs: [(f32, f32, f32, f32); 4] = [
            (-f, f3 * 2.0 + f2, 0.0, 0.0),
            (-f * 0.74, f3 * 1.0 + f2, PI, PI),
            (-f * 0.74, -f3 * 1.0 + f2, 1.5707964, 1.5707964),
            (-f, -f3 * 2.0 + f2, 4.712389, 4.712389),
        ];
        let legs: &[ModelRenderer] = &self.m_spiderLegs;
        for (pair, &(roll, baseYaw, _, _)) in pairs.iter().enumerate()
        {
            legs[pair * 2].m_rotateAngleZ.set(roll);
            legs[pair * 2 + 1].m_rotateAngleZ.set(-roll);
            legs[pair * 2].m_rotateAngleY.set(baseYaw);
            let mirrorYaw: f32 = match pair
            {
                0 => -f3 * 2.0 - f2,
                1 => -f3 * 1.0 - f2,
                2 => f3 * 1.0 - f2,
                _ => f3 * 2.0 - f2,
            };
            legs[pair * 2 + 1].m_rotateAngleY.set(mirrorYaw);
        }
        for (pair, &(_, _, swingPhase, liftPhase)) in pairs.iter().enumerate()
        {
            let swing: f32 = -(math_helper::cos(walk * 0.6662 * 2.0 + swingPhase) * 0.4) * walkSpeed;
            let lift: f32 = (math_helper::sin(walk * 0.6662 + liftPhase) * 0.4).abs() * walkSpeed;
            legs[pair * 2].m_rotateAngleY.set(legs[pair * 2].m_rotateAngleY.get() + swing);
            legs[pair * 2 + 1].m_rotateAngleY.set(legs[pair * 2 + 1].m_rotateAngleY.get() + -swing);
            legs[pair * 2].m_rotateAngleZ.set(legs[pair * 2].m_rotateAngleZ.get() + lift);
            legs[pair * 2 + 1].m_rotateAngleZ.set(legs[pair * 2 + 1].m_rotateAngleZ.get() + -lift);
        }
    }
}
