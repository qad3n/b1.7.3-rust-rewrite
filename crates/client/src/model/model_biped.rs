use crate::model::{ModelBase, ModelBaseFields, ModelRenderer};
use game::math::math_helper;
use std::cell::Cell;
use std::f32::consts::PI;

const DEGREES_PER_RADIAN: f32 = 180.0 / PI;
const NO_SWING: f32 = -9990.0;

pub struct ModelBiped
{
    pub m_base: ModelBaseFields,
    pub m_bipedHead: ModelRenderer,
    pub m_bipedHeadwear: ModelRenderer,
    pub m_bipedBody: ModelRenderer,
    pub m_bipedRightArm: ModelRenderer,
    pub m_bipedLeftArm: ModelRenderer,
    pub m_bipedRightLeg: ModelRenderer,
    pub m_bipedLeftLeg: ModelRenderer,
    pub m_bipedEars: ModelRenderer,
    pub m_bipedCloak: ModelRenderer,
    pub m_itemInLeftHand: Cell<bool>,
    pub m_itemInRightHand: Cell<bool>,
    pub m_isSneak: Cell<bool>,
}

impl ModelBiped
{
    pub fn new(reduction: f32, pivotPoint: f32) -> ModelBiped
    {
        ModelBiped {
            m_base: ModelBaseFields::default(),
            m_bipedCloak: ModelRenderer::new(0, 0).withBox(-5.0, 0.0, -1.0, 10, 16, 1, reduction),
            m_bipedEars: ModelRenderer::new(24, 0).withBox(-3.0, -6.0, -1.0, 6, 6, 1, reduction),
            m_bipedHead: ModelRenderer::new(0, 0).withBox(-4.0, -8.0, -4.0, 8, 8, 8, reduction).at(
                0.0,
                0.0 + pivotPoint,
                0.0,
            ),
            m_bipedHeadwear: ModelRenderer::new(32, 0).withBox(-4.0, -8.0, -4.0, 8, 8, 8, reduction + 0.5).at(
                0.0,
                0.0 + pivotPoint,
                0.0,
            ),
            m_bipedBody: ModelRenderer::new(16, 16).withBox(-4.0, 0.0, -2.0, 8, 12, 4, reduction).at(
                0.0,
                0.0 + pivotPoint,
                0.0,
            ),
            m_bipedRightArm: ModelRenderer::new(40, 16).withBox(-3.0, -2.0, -2.0, 4, 12, 4, reduction).at(
                -5.0,
                2.0 + pivotPoint,
                0.0,
            ),
            m_bipedLeftArm: ModelRenderer::mirrored(40, 16).withBox(-1.0, -2.0, -2.0, 4, 12, 4, reduction).at(
                5.0,
                2.0 + pivotPoint,
                0.0,
            ),
            m_bipedRightLeg: ModelRenderer::new(0, 16).withBox(-2.0, 0.0, -2.0, 4, 12, 4, reduction).at(
                -2.0,
                12.0 + pivotPoint,
                0.0,
            ),
            m_bipedLeftLeg: ModelRenderer::mirrored(0, 16).withBox(-2.0, 0.0, -2.0, 4, 12, 4, reduction).at(
                2.0,
                12.0 + pivotPoint,
                0.0,
            ),
            m_itemInLeftHand: Cell::new(false),
            m_itemInRightHand: Cell::new(false),
            m_isSneak: Cell::new(false),
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
        self.m_bipedHead.render(scale);
        self.m_bipedBody.render(scale);
        self.m_bipedRightArm.render(scale);
        self.m_bipedLeftArm.render(scale);
        self.m_bipedRightLeg.render(scale);
        self.m_bipedLeftLeg.render(scale);
        self.m_bipedHeadwear.render(scale);
    }

    pub fn bipedRotationAngles(&self, walk: f32, walkSpeed: f32, bob: f32, yaw: f32, pitch: f32)
    {
        let head: &ModelRenderer = &self.m_bipedHead;
        let body: &ModelRenderer = &self.m_bipedBody;
        let rightArm: &ModelRenderer = &self.m_bipedRightArm;
        let leftArm: &ModelRenderer = &self.m_bipedLeftArm;
        let rightLeg: &ModelRenderer = &self.m_bipedRightLeg;
        let leftLeg: &ModelRenderer = &self.m_bipedLeftLeg;
        head.m_rotateAngleY.set(yaw / DEGREES_PER_RADIAN);
        head.m_rotateAngleX.set(pitch / DEGREES_PER_RADIAN);
        self.m_bipedHeadwear.m_rotateAngleY.set(head.m_rotateAngleY.get());
        self.m_bipedHeadwear.m_rotateAngleX.set(head.m_rotateAngleX.get());
        rightArm.m_rotateAngleX.set(math_helper::cos(walk * 0.6662 + PI) * 2.0 * walkSpeed * 0.5);
        leftArm.m_rotateAngleX.set(math_helper::cos(walk * 0.6662) * 2.0 * walkSpeed * 0.5);
        rightArm.m_rotateAngleZ.set(0.0);
        leftArm.m_rotateAngleZ.set(0.0);
        rightLeg.m_rotateAngleX.set(math_helper::cos(walk * 0.6662) * 1.4 * walkSpeed);
        leftLeg.m_rotateAngleX.set(math_helper::cos(walk * 0.6662 + PI) * 1.4 * walkSpeed);
        rightLeg.m_rotateAngleY.set(0.0);
        leftLeg.m_rotateAngleY.set(0.0);
        if self.m_base.m_isRiding.get()
        {
            rightArm.m_rotateAngleX.set(rightArm.m_rotateAngleX.get() + (-std::f64::consts::PI / 5.0) as f32);
            leftArm.m_rotateAngleX.set(leftArm.m_rotateAngleX.get() + (-std::f64::consts::PI / 5.0) as f32);
            rightLeg.m_rotateAngleX.set((-std::f64::consts::PI * 2.0 / 5.0) as f32);
            leftLeg.m_rotateAngleX.set((-std::f64::consts::PI * 2.0 / 5.0) as f32);
            rightLeg.m_rotateAngleY.set((std::f64::consts::PI / 10.0) as f32);
            leftLeg.m_rotateAngleY.set((-std::f64::consts::PI / 10.0) as f32);
        }
        if self.m_itemInLeftHand.get()
        {
            leftArm.m_rotateAngleX.set(leftArm.m_rotateAngleX.get() * 0.5 - (std::f64::consts::PI / 10.0) as f32);
        }
        if self.m_itemInRightHand.get()
        {
            rightArm.m_rotateAngleX.set(rightArm.m_rotateAngleX.get() * 0.5 - (std::f64::consts::PI / 10.0) as f32);
        }
        rightArm.m_rotateAngleY.set(0.0);
        leftArm.m_rotateAngleY.set(0.0);
        let onGround: f32 = self.m_base.m_onGround.get();
        if onGround > NO_SWING
        {
            let mut f: f32 = onGround;
            body.m_rotateAngleY.set(math_helper::sin(math_helper::sqrt_float(f) * PI * 2.0) * 0.2);
            let bodyYaw: f32 = body.m_rotateAngleY.get();
            rightArm.m_rotationPointZ.set(math_helper::sin(bodyYaw) * 5.0);
            rightArm.m_rotationPointX.set(-math_helper::cos(bodyYaw) * 5.0);
            leftArm.m_rotationPointZ.set(-math_helper::sin(bodyYaw) * 5.0);
            leftArm.m_rotationPointX.set(math_helper::cos(bodyYaw) * 5.0);
            rightArm.m_rotateAngleY.set(rightArm.m_rotateAngleY.get() + bodyYaw);
            leftArm.m_rotateAngleY.set(leftArm.m_rotateAngleY.get() + bodyYaw);
            leftArm.m_rotateAngleX.set(leftArm.m_rotateAngleX.get() + bodyYaw);
            f = 1.0 - onGround;
            f *= f;
            f *= f;
            f = 1.0 - f;
            let f1: f32 = math_helper::sin(f * PI);
            let f2: f32 = math_helper::sin(onGround * PI) * -(head.m_rotateAngleX.get() - 0.7) * 0.75;
            rightArm.m_rotateAngleX.set((rightArm.m_rotateAngleX.get() as f64 - (f1 as f64 * 1.2 + f2 as f64)) as f32);
            rightArm.m_rotateAngleY.set(rightArm.m_rotateAngleY.get() + bodyYaw * 2.0);
            rightArm.m_rotateAngleZ.set(math_helper::sin(onGround * PI) * -0.4);
        }
        if self.m_isSneak.get()
        {
            body.m_rotateAngleX.set(0.5);
            rightLeg.m_rotateAngleX.set(rightLeg.m_rotateAngleX.get() - 0.0);
            leftLeg.m_rotateAngleX.set(leftLeg.m_rotateAngleX.get() - 0.0);
            rightArm.m_rotateAngleX.set(rightArm.m_rotateAngleX.get() + 0.4);
            leftArm.m_rotateAngleX.set(leftArm.m_rotateAngleX.get() + 0.4);
            rightLeg.m_rotationPointZ.set(4.0);
            leftLeg.m_rotationPointZ.set(4.0);
            rightLeg.m_rotationPointY.set(9.0);
            leftLeg.m_rotationPointY.set(9.0);
            head.m_rotationPointY.set(1.0);
        }
        else
        {
            body.m_rotateAngleX.set(0.0);
            rightLeg.m_rotationPointZ.set(0.0);
            leftLeg.m_rotationPointZ.set(0.0);
            rightLeg.m_rotationPointY.set(12.0);
            leftLeg.m_rotationPointY.set(12.0);
            head.m_rotationPointY.set(0.0);
        }
        self.armBob(bob);
    }

    pub fn armBob(&self, bob: f32)
    {
        let rightArm: &ModelRenderer = &self.m_bipedRightArm;
        let leftArm: &ModelRenderer = &self.m_bipedLeftArm;
        rightArm.m_rotateAngleZ.set(rightArm.m_rotateAngleZ.get() + (math_helper::cos(bob * 0.09) * 0.05 + 0.05));
        leftArm.m_rotateAngleZ.set(leftArm.m_rotateAngleZ.get() - (math_helper::cos(bob * 0.09) * 0.05 + 0.05));
        rightArm.m_rotateAngleX.set(rightArm.m_rotateAngleX.get() + math_helper::sin(bob * 0.067) * 0.05);
        leftArm.m_rotateAngleX.set(leftArm.m_rotateAngleX.get() - math_helper::sin(bob * 0.067) * 0.05);
    }

    pub fn renderEars(&self, scale: f32)
    {
        self.m_bipedEars.m_rotateAngleY.set(self.m_bipedHead.m_rotateAngleY.get());
        self.m_bipedEars.m_rotateAngleX.set(self.m_bipedHead.m_rotateAngleX.get());
        self.m_bipedEars.m_rotationPointX.set(0.0);
        self.m_bipedEars.m_rotationPointY.set(0.0);
        self.m_bipedEars.render(scale);
    }

    pub fn renderCloak(&self, scale: f32) { self.m_bipedCloak.render(scale); }
}

impl ModelBase for ModelBiped
{
    fn base(&self) -> &ModelBaseFields { &self.m_base }

    fn render(&self, walk: f32, walkSpeed: f32, bob: f32, yaw: f32, pitch: f32, scale: f32)
    {
        self.renderParts(self, walk, walkSpeed, bob, yaw, pitch, scale);
    }

    fn setRotationAngles(&self, walk: f32, walkSpeed: f32, bob: f32, yaw: f32, pitch: f32, _scale: f32)
    {
        self.bipedRotationAngles(walk, walkSpeed, bob, yaw, pitch);
    }

    fn biped(&self) -> Option<&ModelBiped> { Some(self) }
}
