use crate::model::{ModelBase, ModelBaseFields, ModelRenderer};
use game::entity::{Entity, entity_wolf};
use game::math::math_helper;
use lwjgl::opengl::GL11;
use std::f32::consts::PI;
use std::rc::Rc;

const DEGREES_PER_RADIAN: f32 = 180.0 / PI;

pub struct ModelWolf
{
    m_base: ModelBaseFields,
    pub m_wolfHeadMain: ModelRenderer,
    pub m_wolfBody: ModelRenderer,
    pub m_wolfLeg1: ModelRenderer,
    pub m_wolfLeg2: ModelRenderer,
    pub m_wolfLeg3: ModelRenderer,
    pub m_wolfLeg4: ModelRenderer,
    m_wolfRightEar: ModelRenderer,
    m_wolfLeftEar: ModelRenderer,
    m_wolfSnout: ModelRenderer,
    m_wolfTail: ModelRenderer,
    m_wolfMane: ModelRenderer,
}

impl ModelWolf
{
    pub fn new() -> ModelWolf
    {
        let f: f32 = 0.0;
        let f1: f32 = 13.5;
        let leg = |x: f32, z: f32| -> ModelRenderer {
            ModelRenderer::new(0, 18).withBox(-1.0, 0.0, -1.0, 2, 8, 2, f).at(x, 16.0, z)
        };
        ModelWolf {
            m_base: ModelBaseFields::default(),
            m_wolfHeadMain: ModelRenderer::new(0, 0).withBox(-3.0, -3.0, -2.0, 6, 6, 4, f).at(-1.0, f1, -7.0),
            m_wolfBody: ModelRenderer::new(18, 14).withBox(-4.0, -2.0, -3.0, 6, 9, 6, f).at(0.0, 14.0, 2.0),
            m_wolfMane: ModelRenderer::new(21, 0).withBox(-4.0, -3.0, -3.0, 8, 6, 7, f).at(-1.0, 14.0, 2.0),
            m_wolfLeg1: leg(-2.5, 7.0),
            m_wolfLeg2: leg(0.5, 7.0),
            m_wolfLeg3: leg(-2.5, -4.0),
            m_wolfLeg4: leg(0.5, -4.0),
            m_wolfTail: ModelRenderer::new(9, 18).withBox(-1.0, 0.0, -1.0, 2, 8, 2, f).at(-1.0, 12.0, 8.0),
            m_wolfRightEar: ModelRenderer::new(16, 14).withBox(-3.0, -5.0, 0.0, 2, 2, 1, f).at(-1.0, f1, -7.0),
            m_wolfLeftEar: ModelRenderer::new(16, 14).withBox(1.0, -5.0, 0.0, 2, 2, 1, f).at(-1.0, f1, -7.0),
            m_wolfSnout: ModelRenderer::new(0, 10).withBox(-2.0, 0.0, -5.0, 3, 3, 4, f).at(-0.5, f1, -7.0),
        }
    }
}

impl ModelBase for ModelWolf
{
    fn base(&self) -> &ModelBaseFields { &self.m_base }

    fn render(&self, walk: f32, walkSpeed: f32, bob: f32, yaw: f32, pitch: f32, scale: f32)
    {
        self.setRotationAngles(walk, walkSpeed, bob, yaw, pitch, scale);
        self.m_wolfHeadMain.renderWithRotation(scale);
        for leg in [&self.m_wolfBody, &self.m_wolfLeg1, &self.m_wolfLeg2, &self.m_wolfLeg3, &self.m_wolfLeg4]
        {
            leg.render(scale);
        }
        for part in [&self.m_wolfRightEar, &self.m_wolfLeftEar, &self.m_wolfSnout, &self.m_wolfTail]
        {
            part.renderWithRotation(scale);
        }
        self.m_wolfMane.render(scale);
    }

    fn setLivingAnimations(&self, mob: &Rc<Entity>, walk: f32, walkSpeed: f32, tickDelta: f32)
    {
        if entity_wolf::isWolfAngry(mob)
        {
            self.m_wolfTail.m_rotateAngleY.set(0.0);
        }
        else
        {
            self.m_wolfTail.m_rotateAngleY.set(math_helper::cos(walk * 0.6662) * 1.4 * walkSpeed);
        }
        if entity_wolf::isWolfSitting(mob)
        {
            self.m_wolfMane.setRotationPoint(-1.0, 16.0, -3.0);
            self.m_wolfMane.m_rotateAngleX.set((std::f64::consts::PI * 2.0 / 5.0) as f32);
            self.m_wolfMane.m_rotateAngleY.set(0.0);
            self.m_wolfBody.setRotationPoint(0.0, 18.0, 0.0);
            self.m_wolfBody.m_rotateAngleX.set((std::f64::consts::PI / 4.0) as f32);
            self.m_wolfTail.setRotationPoint(-1.0, 21.0, 6.0);
            self.m_wolfLeg1.setRotationPoint(-2.5, 22.0, 2.0);
            self.m_wolfLeg1.m_rotateAngleX.set((std::f64::consts::PI * 3.0 / 2.0) as f32);
            self.m_wolfLeg2.setRotationPoint(0.5, 22.0, 2.0);
            self.m_wolfLeg2.m_rotateAngleX.set((std::f64::consts::PI * 3.0 / 2.0) as f32);
            self.m_wolfLeg3.m_rotateAngleX.set(5.811947);
            self.m_wolfLeg3.setRotationPoint(-2.49, 17.0, -4.0);
            self.m_wolfLeg4.m_rotateAngleX.set(5.811947);
            self.m_wolfLeg4.setRotationPoint(0.51, 17.0, -4.0);
        }
        else
        {
            self.m_wolfBody.setRotationPoint(0.0, 14.0, 2.0);
            self.m_wolfBody.m_rotateAngleX.set((std::f64::consts::PI / 2.0) as f32);
            self.m_wolfMane.setRotationPoint(-1.0, 14.0, -3.0);
            self.m_wolfMane.m_rotateAngleX.set(self.m_wolfBody.m_rotateAngleX.get());
            self.m_wolfTail.setRotationPoint(-1.0, 12.0, 8.0);
            self.m_wolfLeg1.setRotationPoint(-2.5, 16.0, 7.0);
            self.m_wolfLeg2.setRotationPoint(0.5, 16.0, 7.0);
            self.m_wolfLeg3.setRotationPoint(-2.5, 16.0, -4.0);
            self.m_wolfLeg4.setRotationPoint(0.5, 16.0, -4.0);
            self.m_wolfLeg1.m_rotateAngleX.set(math_helper::cos(walk * 0.6662) * 1.4 * walkSpeed);
            self.m_wolfLeg2.m_rotateAngleX.set(math_helper::cos(walk * 0.6662 + PI) * 1.4 * walkSpeed);
            self.m_wolfLeg3.m_rotateAngleX.set(math_helper::cos(walk * 0.6662 + PI) * 1.4 * walkSpeed);
            self.m_wolfLeg4.m_rotateAngleX.set(math_helper::cos(walk * 0.6662) * 1.4 * walkSpeed);
        }
        let f: f32 = entity_wolf::getInterestedAngle(mob, tickDelta) + entity_wolf::getShakeAngle(mob, tickDelta, 0.0);
        for part in [&self.m_wolfHeadMain, &self.m_wolfRightEar, &self.m_wolfLeftEar, &self.m_wolfSnout]
        {
            part.m_rotateAngleZ.set(f);
        }
        self.m_wolfMane.m_rotateAngleZ.set(entity_wolf::getShakeAngle(mob, tickDelta, -0.08));
        self.m_wolfBody.m_rotateAngleZ.set(entity_wolf::getShakeAngle(mob, tickDelta, -0.16));
        self.m_wolfTail.m_rotateAngleZ.set(entity_wolf::getShakeAngle(mob, tickDelta, -0.2));
        if entity_wolf::getWolfShaking(mob)
        {
            let f1: f32 = mob.getEntityBrightness(tickDelta) * entity_wolf::getShadingWhileShaking(mob, tickDelta);
            GL11::glColor3f(f1, f1, f1);
        }
    }

    fn setRotationAngles(&self, _walk: f32, _walkSpeed: f32, bob: f32, yaw: f32, pitch: f32, _scale: f32)
    {
        self.m_wolfHeadMain.m_rotateAngleX.set(pitch / DEGREES_PER_RADIAN);
        self.m_wolfHeadMain.m_rotateAngleY.set(yaw / DEGREES_PER_RADIAN);
        for part in [&self.m_wolfRightEar, &self.m_wolfLeftEar, &self.m_wolfSnout]
        {
            part.m_rotateAngleY.set(self.m_wolfHeadMain.m_rotateAngleY.get());
            part.m_rotateAngleX.set(self.m_wolfHeadMain.m_rotateAngleX.get());
        }
        self.m_wolfTail.m_rotateAngleX.set(bob);
    }
}
