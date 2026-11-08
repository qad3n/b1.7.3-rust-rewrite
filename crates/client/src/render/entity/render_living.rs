use crate::client::Minecraft;
use crate::gui::FontRenderer;
use crate::model::ModelBase;
use crate::render::Tessellator;
use crate::render::entity::{LivingRenderFields, RENDER, Render, RenderClass};
use game::entity::{Entity, entity_living};
use game::math::math_helper;
use jvm::field::Field;
use jvm::throwable::{self, OrNpe, Throwable};
use lwjgl::opengl::{GL11, GL12};
use std::any::Any;
use std::rc::Rc;

const MODEL_SCALE: f32 = 0.0625;
const RENDER_PASSES: i32 = 4;
const LABEL_SCALE: f32 = 0.016666668;
const LABEL_SHADOW_COLOR: i32 = 553648127;

pub fn living(mainModel: Rc<dyn ModelBase>) -> Option<LivingRenderFields>
{
    Some(LivingRenderFields { m_mainModel: mainModel, m_renderPassModel: Field::new(None) })
}

pub fn create(class: &'static RenderClass, model: Rc<dyn ModelBase>, shadowSize: f32, ext: Box<dyn Any>) -> Rc<Render>
{
    let render: Rc<Render> = Render::alloc(class, living(model), ext);
    render.m_shadowSize.set(shadowSize);
    render
}

pub fn new(model: Rc<dyn ModelBase>, shadowSize: f32) -> Rc<Render>
{
    create(&RENDER_LIVING, model, shadowSize, Box::new(()))
}

pub fn setRenderPassModel(this: &Render, model: Rc<dyn ModelBase>) { this.living().m_renderPassModel.set(Some(model)); }

fn renderPassModel(this: &Render) -> Rc<dyn ModelBase> { this.living().m_renderPassModel.get().orNpe() }

fn lerp(previous: f32, current: f32, tickDelta: f32) -> f32 { previous + (current - previous) * tickDelta }

fn renderBody(this: &Rc<Render>, entity: &Rc<Entity>, dx: f64, dy: f64, dz: f64, tickDelta: f32)
{
    let fields: &entity_living::LivingFields = entity_living::fields(entity);
    let mainModel: &Rc<dyn ModelBase> = &this.living().m_mainModel;
    let f: f32 = lerp(fields.m_prevRenderYawOffset.get(), fields.m_renderYawOffset.get(), tickDelta);
    let f1: f32 = lerp(entity.m_prevRotationYaw.get(), entity.m_rotationYaw.get(), tickDelta);
    let f2: f32 = lerp(entity.m_prevRotationPitch.get(), entity.m_rotationPitch.get(), tickDelta);
    this.applyTranslation(entity, dx, dy, dz);
    let f3: f32 = this.getBob(entity, tickDelta);
    this.rotateCorpse(entity, f3, f, tickDelta);
    let f4: f32 = MODEL_SCALE;
    GL11::glEnable(GL12::GL_RESCALE_NORMAL);
    GL11::glScalef(-1.0, -1.0, 1.0);
    this.preRenderCallback(entity, tickDelta);
    GL11::glTranslatef(0.0, -24.0 * f4 - 0.0078125, 0.0);
    let mut f5: f32 = lerp(fields.m_lastWalkAnimationSpeed.get(), fields.m_walkAnimationSpeed.get(), tickDelta);
    let f6: f32 = fields.m_walkAnimationProgress.get() - fields.m_walkAnimationSpeed.get() * (1.0 - tickDelta);
    if f5 > 1.0
    {
        f5 = 1.0;
    }
    let skinUrl: Option<String> = entity.m_skinUrl.get().map(|url| jvm::string::JString::display(Some(&url)));
    let texture: Option<String> = entity.getEntityTexture().map(|path| jvm::string::JString::display(Some(&path)));
    this.loadDownloadableImageTexture(skinUrl.as_deref(), texture.as_deref());
    GL11::glEnable(GL11::GL_ALPHA_TEST);
    mainModel.setLivingAnimations(entity, f6, f5, tickDelta);
    mainModel.render(f6, f5, f3, f1 - f, f2, f4);
    for i in 0..RENDER_PASSES
    {
        if this.shouldRenderPass(entity, i, tickDelta)
        {
            renderPassModel(this).render(f6, f5, f3, f1 - f, f2, f4);
            GL11::glDisable(GL11::GL_BLEND);
            GL11::glEnable(GL11::GL_ALPHA_TEST);
        }
    }
    this.renderEquippedItems(entity, tickDelta);
    let f10: f32 = entity.getEntityBrightness(tickDelta);
    let j: i32 = this.getColorMultiplier(entity, f10, tickDelta);
    let hurt: bool = fields.m_hurtTime.get() > 0 || fields.m_deathTime.get() > 0;
    if (j >> 24 & 0xFF) > 0 || hurt
    {
        GL11::glDisable(GL11::GL_TEXTURE_2D);
        GL11::glDisable(GL11::GL_ALPHA_TEST);
        GL11::glEnable(GL11::GL_BLEND);
        GL11::glBlendFunc(GL11::GL_SRC_ALPHA, GL11::GL_ONE_MINUS_SRC_ALPHA);
        GL11::glDepthFunc(GL11::GL_EQUAL);
        let overlay = |r: f32, g: f32, b: f32, a: f32| {
            GL11::glColor4f(r, g, b, a);
            mainModel.render(f6, f5, f3, f1 - f, f2, f4);
            for k in 0..RENDER_PASSES
            {
                if this.bindDecorationTexture(entity, k, tickDelta)
                {
                    GL11::glColor4f(r, g, b, a);
                    renderPassModel(this).render(f6, f5, f3, f1 - f, f2, f4);
                }
            }
        };
        if hurt
        {
            overlay(f10, 0.0, 0.0, 0.4);
        }
        if (j >> 24 & 0xFF) > 0
        {
            let channel = |shift: i32| -> f32 { (j >> shift & 0xFF) as f32 / 255.0 };
            overlay(channel(16), channel(8), channel(0), channel(24));
        }
        GL11::glDepthFunc(GL11::GL_LEQUAL);
        GL11::glDisable(GL11::GL_BLEND);
        GL11::glEnable(GL11::GL_ALPHA_TEST);
        GL11::glEnable(GL11::GL_TEXTURE_2D);
    }
    GL11::glDisable(GL12::GL_RESCALE_NORMAL);
}

fn doRender(this: &Rc<Render>, entity: &Rc<Entity>, dx: f64, dy: f64, dz: f64, _yaw: f32, tickDelta: f32)
{
    GL11::glPushMatrix();
    GL11::glDisable(GL11::GL_CULL_FACE);
    let fields: &LivingRenderFields = this.living();
    let mainModel: &Rc<dyn ModelBase> = &fields.m_mainModel;
    mainModel.base().m_onGround.set(this.getAttackAnimationProgress(entity, tickDelta));
    if let Some(passModel) = fields.m_renderPassModel.get()
    {
        passModel.base().m_onGround.set(mainModel.base().m_onGround.get());
    }
    mainModel.base().m_isRiding.set(entity.isRiding());
    if let Some(passModel) = fields.m_renderPassModel.get()
    {
        passModel.base().m_isRiding.set(mainModel.base().m_isRiding.get());
    }
    let result: Result<(), Throwable> =
        throwable::catchThrowable(&throwable::EXCEPTION, || renderBody(this, entity, dx, dy, dz, tickDelta));
    if let Err(exception) = result
    {
        exception.printStackTrace();
    }
    GL11::glEnable(GL11::GL_CULL_FACE);
    GL11::glPopMatrix();
    this.passSpecialRender(entity, dx, dy, dz);
}

pub fn applyTranslation(_this: &Rc<Render>, _entity: &Rc<Entity>, dx: f64, dy: f64, dz: f64)
{
    GL11::glTranslatef(dx as f32, dy as f32, dz as f32);
}

pub fn rotateCorpse(this: &Rc<Render>, entity: &Rc<Entity>, _bob: f32, bodyYaw: f32, tickDelta: f32)
{
    GL11::glRotatef(180.0 - bodyYaw, 0.0, 1.0, 0.0);
    let deathTime: i32 = entity_living::fields(entity).m_deathTime.get();
    if deathTime > 0
    {
        let mut f: f32 = (deathTime as f32 + tickDelta - 1.0) / 20.0 * 1.6;
        f = math_helper::sqrt_float(f);
        if f > 1.0
        {
            f = 1.0;
        }
        GL11::glRotatef(f * this.getDeathMaxRotation(entity), 0.0, 0.0, 1.0);
    }
}

pub fn renderLivingLabel(this: &Render, entity: &Rc<Entity>, name: &str, dx: f64, dy: f64, dz: f64, range: i32)
{
    let manager = this.renderManager();
    let f: f32 = entity.getDistanceToEntity(&manager.m_livingPlayer.get().orNpe());
    if f > range as f32
    {
        return;
    }
    let fontrenderer: Rc<FontRenderer> = this.getFontRendererFromRenderManager().orNpe();
    let f1: f32 = 1.6;
    let f2: f32 = LABEL_SCALE * f1;
    GL11::glPushMatrix();
    GL11::glTranslatef(dx as f32 + 0.0, dy as f32 + 2.3, dz as f32);
    GL11::glNormal3f(0.0, 1.0, 0.0);
    GL11::glRotatef(-manager.m_playerViewY.get(), 0.0, 1.0, 0.0);
    GL11::glRotatef(manager.m_playerViewX.get(), 1.0, 0.0, 0.0);
    GL11::glScalef(-f2, -f2, f2);
    GL11::glDisable(GL11::GL_LIGHTING);
    GL11::glDepthMask(false);
    GL11::glDisable(GL11::GL_DEPTH_TEST);
    GL11::glEnable(GL11::GL_BLEND);
    GL11::glBlendFunc(GL11::GL_SRC_ALPHA, GL11::GL_ONE_MINUS_SRC_ALPHA);
    let b0: i32 = if name == "deadmau5" { -10 } else { 0 };
    labelBackground(&fontrenderer, name, b0 as f64);
    fontrenderer.drawString(name, -fontrenderer.getStringWidth(name) / 2, b0, LABEL_SHADOW_COLOR);
    GL11::glEnable(GL11::GL_DEPTH_TEST);
    GL11::glDepthMask(true);
    fontrenderer.drawString(name, -fontrenderer.getStringWidth(name) / 2, b0, -1);
    GL11::glEnable(GL11::GL_LIGHTING);
    GL11::glDisable(GL11::GL_BLEND);
    GL11::glColor4f(1.0, 1.0, 1.0, 1.0);
    GL11::glPopMatrix();
}

pub fn labelBackground(fontrenderer: &FontRenderer, name: &str, top: f64)
{
    let tessellator: Rc<Tessellator> = Tessellator::instance();
    GL11::glDisable(GL11::GL_TEXTURE_2D);
    tessellator.startDrawingQuads();
    let i: i32 = fontrenderer.getStringWidth(name) / 2;
    tessellator.setColorRGBA_F(0.0, 0.0, 0.0, 0.25);
    tessellator.addVertex((-i - 1) as f64, -1.0 + top, 0.0);
    tessellator.addVertex((-i - 1) as f64, 8.0 + top, 0.0);
    tessellator.addVertex((i + 1) as f64, 8.0 + top, 0.0);
    tessellator.addVertex((i + 1) as f64, -1.0 + top, 0.0);
    tessellator.draw();
    GL11::glEnable(GL11::GL_TEXTURE_2D);
}

pub const RENDER_LIVING: RenderClass = RenderClass {
    m_name: "RenderLiving",
    m_parent: Some(&RENDER),
    doRender,
    applyTranslation,
    rotateCorpse,
    getAttackAnimationProgress: |_, entity, tickDelta| entity_living::getSwingProgress(entity, tickDelta),
    getBob: |_, entity, tickDelta| entity.m_ticksExisted.get() as f32 + tickDelta,
    renderEquippedItems: |_, _, _| {},
    bindDecorationTexture: |this, entity, layer, tickDelta| this.shouldRenderPass(entity, layer, tickDelta),
    shouldRenderPass: |_, _, _, _| false,
    getDeathMaxRotation: |_, _| 90.0,
    getColorMultiplier: |_, _, _, _| 0,
    preRenderCallback: |_, _, _| {},
    passSpecialRender: |this, entity, dx, dy, dz| {
        if Minecraft::isDebugInfoEnabled()
        {
            renderLivingLabel(this, entity, &entity.m_entityId.get().to_string(), dx, dy, dz, 64);
        }
    },
    ..RENDER
};
