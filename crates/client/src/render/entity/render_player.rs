use crate::client::Minecraft;
use crate::gui::FontRenderer;
use crate::model::{ModelBase, ModelBiped};
use crate::render::entity::render_biped::{modelBipedMain, moveToRightHand, rendersAsBlock, transformHeldItem};
use crate::render::entity::render_living::{
    self, RENDER_LIVING, labelBackground, renderLivingLabel, setRenderPassModel,
};
use crate::render::entity::{Render, RenderClass};
use game::entity::{Entity, entity_living, entity_player};
use game::item::{Item, ItemStack, item_armor};
use game::math::math_helper;
use jvm::string::JString;
use jvm::throwable::OrNpe;
use lwjgl::opengl::GL11;
use std::f32::consts::PI;
use std::rc::Rc;

const ARMOR_FILENAME_PREFIX: [&str; 5] = ["cloth", "chain", "iron", "diamond", "gold"];
const MODEL_SCALE: f32 = 0.0625;
const LABEL_SCALE: f32 = 0.016666668;
const LABEL_SHADOW_COLOR: i32 = 553648127;
const BLOCK_ITEM_LIMIT: i32 = 256;
const HELMET_SLOT: i32 = 3;

struct PlayerRenderExt
{
    m_modelArmorChestplate: Rc<ModelBiped>,
    m_modelArmor: Rc<ModelBiped>,
}

fn ext(this: &Render) -> &PlayerRenderExt { this.ext::<PlayerRenderExt>() }

pub fn new() -> Rc<Render>
{
    let playerExt: PlayerRenderExt = PlayerRenderExt {
        m_modelArmorChestplate: Rc::new(ModelBiped::new(1.0, 0.0)),
        m_modelArmor: Rc::new(ModelBiped::new(0.5, 0.0)),
    };
    render_living::create(&RENDER_PLAYER, Rc::new(ModelBiped::new(0.0, 0.0)), 0.5, Box::new(playerExt))
}

fn username(entity: &Entity) -> String
{
    JString::display(Some(&entity_player::fields(entity).m_username.get().orNpe()))
}

fn shouldRenderPass(this: &Rc<Render>, entity: &Rc<Entity>, layer: i32, _tickDelta: f32) -> bool
{
    let Some(itemstack) = entity_player::inventory(entity).armorItemInSlot(3 - layer)
    else
    {
        return false;
    };
    let Some(item) = itemstack.getItemNullable().filter(|item| item.instanceOf("ItemArmor"))
    else
    {
        return false;
    };
    let renderIndex: i32 = item_armor::fields(&item).m_renderIndex;
    this.loadTexture(&format!(
        "/armor/{}_{}.png",
        ARMOR_FILENAME_PREFIX[renderIndex as usize],
        if layer == 2 { 2 } else { 1 }
    ));
    let modelbiped: &Rc<ModelBiped> = if layer == 2
    {
        &ext(this).m_modelArmor
    }
    else
    {
        &ext(this).m_modelArmorChestplate
    };
    modelbiped.m_bipedHead.m_showModel.set(layer == 0);
    modelbiped.m_bipedHeadwear.m_showModel.set(layer == 0);
    modelbiped.m_bipedBody.m_showModel.set(layer == 1 || layer == 2);
    modelbiped.m_bipedRightArm.m_showModel.set(layer == 1);
    modelbiped.m_bipedLeftArm.m_showModel.set(layer == 1);
    modelbiped.m_bipedRightLeg.m_showModel.set(layer == 2 || layer == 3);
    modelbiped.m_bipedLeftLeg.m_showModel.set(layer == 2 || layer == 3);
    setRenderPassModel(this, Rc::clone(modelbiped) as Rc<dyn ModelBase>);
    true
}

fn setPose(this: &Render, holding: bool, sneaking: bool)
{
    for model in [&*ext(this).m_modelArmorChestplate, &*ext(this).m_modelArmor, modelBipedMain(this)]
    {
        model.m_itemInRightHand.set(holding);
        model.m_isSneak.set(sneaking);
    }
}

fn doRender(this: &Rc<Render>, entity: &Rc<Entity>, dx: f64, dy: f64, dz: f64, yaw: f32, tickDelta: f32)
{
    let itemstack: Option<Rc<ItemStack>> = entity_player::inventory(entity).getCurrentItem();
    setPose(this, itemstack.is_some(), entity.isSneaking());
    let mut d0: f64 = dy - entity.m_yOffset.get() as f64;
    if entity.isSneaking() && !entity.instanceOf("EntityPlayerSP")
    {
        d0 -= 0.125;
    }
    (RENDER_LIVING.doRender)(this, entity, dx, d0, dz, yaw, tickDelta);
    setPose(this, false, false);
}

fn passSpecialRender(this: &Rc<Render>, entity: &Rc<Entity>, dx: f64, dy: f64, dz: f64)
{
    let manager = this.renderManager();
    let camera: Option<Rc<Entity>> = manager.m_livingPlayer.get();
    if !Minecraft::isGuiEnabled() || camera.as_ref().is_some_and(|camera| Rc::ptr_eq(camera, entity))
    {
        return;
    }
    let f: f32 = 1.6;
    let f1: f32 = LABEL_SCALE * f;
    let f2: f32 = entity.getDistanceToEntity(&camera.orNpe());
    let f3: f32 = if entity.isSneaking() { 32.0 } else { 64.0 };
    if !(f2 < f3)
    {
        return;
    }
    let s: String = username(entity);
    if !entity.isSneaking()
    {
        let labelY: f64 = if entity.isPlayerSleeping()
        {
            dy - 1.5
        }
        else
        {
            dy
        };
        renderLivingLabel(this, entity, &s, dx, labelY, dz, 64);
        return;
    }
    let fontrenderer: Rc<FontRenderer> = this.getFontRendererFromRenderManager().orNpe();
    GL11::glPushMatrix();
    GL11::glTranslatef(dx as f32 + 0.0, dy as f32 + 2.3, dz as f32);
    GL11::glNormal3f(0.0, 1.0, 0.0);
    GL11::glRotatef(-manager.m_playerViewY.get(), 0.0, 1.0, 0.0);
    GL11::glRotatef(manager.m_playerViewX.get(), 1.0, 0.0, 0.0);
    GL11::glScalef(-f1, -f1, f1);
    GL11::glDisable(GL11::GL_LIGHTING);
    GL11::glTranslatef(0.0, 0.25 / f1, 0.0);
    GL11::glDepthMask(false);
    GL11::glEnable(GL11::GL_BLEND);
    GL11::glBlendFunc(GL11::GL_SRC_ALPHA, GL11::GL_ONE_MINUS_SRC_ALPHA);
    labelBackground(&fontrenderer, &s, 0.0);
    GL11::glDepthMask(true);
    fontrenderer.drawString(&s, -fontrenderer.getStringWidth(&s) / 2, 0, LABEL_SHADOW_COLOR);
    GL11::glEnable(GL11::GL_LIGHTING);
    GL11::glDisable(GL11::GL_BLEND);
    GL11::glColor4f(1.0, 1.0, 1.0, 1.0);
    GL11::glPopMatrix();
}

fn lerp(previous: f32, current: f32, tickDelta: f32) -> f32 { previous + (current - previous) * tickDelta }

fn lerpDouble(previous: f64, current: f64, tickDelta: f32) -> f64 { previous + (current - previous) * tickDelta as f64 }

fn renderEars(this: &Render, entity: &Rc<Entity>, tickDelta: f32)
{
    let living: &entity_living::LivingFields = entity_living::fields(entity);
    for i in 0..2
    {
        let f1: f32 = lerp(entity.m_prevRotationYaw.get(), entity.m_rotationYaw.get(), tickDelta)
            - lerp(living.m_prevRenderYawOffset.get(), living.m_renderYawOffset.get(), tickDelta);
        let f2: f32 = lerp(entity.m_prevRotationPitch.get(), entity.m_rotationPitch.get(), tickDelta);
        GL11::glPushMatrix();
        GL11::glRotatef(f1, 0.0, 1.0, 0.0);
        GL11::glRotatef(f2, 1.0, 0.0, 0.0);
        GL11::glTranslatef(0.375 * (i * 2 - 1) as f32, 0.0, 0.0);
        GL11::glTranslatef(0.0, -0.375, 0.0);
        GL11::glRotatef(-f2, 1.0, 0.0, 0.0);
        GL11::glRotatef(-f1, 0.0, 1.0, 0.0);
        let f3: f32 = 1.3333334;
        GL11::glScalef(f3, f3, f3);
        modelBipedMain(this).renderEars(MODEL_SCALE);
        GL11::glPopMatrix();
    }
}

fn renderCloak(this: &Render, entity: &Rc<Entity>, tickDelta: f32)
{
    let player: &entity_player::PlayerFields = entity_player::fields(entity);
    let living: &entity_living::LivingFields = entity_living::fields(entity);
    GL11::glPushMatrix();
    GL11::glTranslatef(0.0, 0.0, 0.125);
    let d3: f64 = lerpDouble(player.m_lastCapeX.get(), player.m_capeX.get(), tickDelta)
        - lerpDouble(entity.m_prevPosX.get(), entity.posX(), tickDelta);
    let d4: f64 = lerpDouble(player.m_lastCapeY.get(), player.m_capeY.get(), tickDelta)
        - lerpDouble(entity.m_prevPosY.get(), entity.posY(), tickDelta);
    let d0: f64 = lerpDouble(player.m_lastCapeZ.get(), player.m_capeZ.get(), tickDelta)
        - lerpDouble(entity.m_prevPosZ.get(), entity.posZ(), tickDelta);
    let f4: f32 = lerp(living.m_prevRenderYawOffset.get(), living.m_renderYawOffset.get(), tickDelta);
    let d1: f64 = math_helper::sin(f4 * PI / 180.0) as f64;
    let d2: f64 = -math_helper::cos(f4 * PI / 180.0) as f64;
    let mut f5: f32 = (d4 as f32 * 10.0).clamp(-6.0, 32.0);
    let mut f6: f32 = (d3 * d1 + d0 * d2) as f32 * 100.0;
    let f7: f32 = (d3 * d2 - d0 * d1) as f32 * 100.0;
    if f6 < 0.0
    {
        f6 = 0.0;
    }
    let f8: f32 = lerp(player.m_lastBob.get(), player.m_bob.get(), tickDelta);
    f5 += math_helper::sin(
        lerp(entity.m_prevDistanceWalkedModified.get(), entity.m_distanceWalkedModified.get(), tickDelta) * 6.0,
    ) * 32.0
        * f8;
    if entity.isSneaking()
    {
        f5 += 25.0;
    }
    GL11::glRotatef(6.0 + f6 / 2.0 + f5, 1.0, 0.0, 0.0);
    GL11::glRotatef(f7 / 2.0, 0.0, 0.0, 1.0);
    GL11::glRotatef(-f7 / 2.0, 0.0, 1.0, 0.0);
    GL11::glRotatef(180.0, 0.0, 1.0, 0.0);
    modelBipedMain(this).renderCloak(MODEL_SCALE);
    GL11::glPopMatrix();
}

fn renderEquippedItems(this: &Rc<Render>, entity: &Rc<Entity>, tickDelta: f32)
{
    if let Some(itemstack) = entity_player::inventory(entity).armorItemInSlot(HELMET_SLOT)
        && itemstack.getItem().m_shiftedIndex < BLOCK_ITEM_LIMIT
    {
        GL11::glPushMatrix();
        modelBipedMain(this).m_bipedHead.postRender(MODEL_SCALE);
        if rendersAsBlock(itemstack.m_itemID.get())
        {
            let f: f32 = 0.625;
            GL11::glTranslatef(0.0, -0.25, 0.0);
            GL11::glRotatef(180.0, 0.0, 1.0, 0.0);
            GL11::glScalef(f, -f, f);
        }
        this.renderManager().m_itemRenderer.get().orNpe().renderItem(entity, &itemstack);
        GL11::glPopMatrix();
    }
    let skinUrl: Option<String> = entity.m_skinUrl.get().map(|url| JString::display(Some(&url)));
    if username(entity) == "deadmau5" && this.loadDownloadableImageTexture(skinUrl.as_deref(), None)
    {
        renderEars(this, entity, tickDelta);
    }
    let cloakUrl: Option<String> =
        entity_player::fields(entity).m_playerCloakUrl.get().map(|url| JString::display(Some(&url)));
    if this.loadDownloadableImageTexture(cloakUrl.as_deref(), None)
    {
        renderCloak(this, entity, tickDelta);
    }
    if let Some(mut itemstack1) = entity_player::inventory(entity).getCurrentItem()
    {
        GL11::glPushMatrix();
        moveToRightHand(modelBipedMain(this));
        if entity_player::fields(entity).m_fishEntity.with(Option::is_some)
        {
            itemstack1 = ItemStack::withItem(&Item::stick());
        }
        transformHeldItem(&itemstack1, true);
        this.renderManager().m_itemRenderer.get().orNpe().renderItem(entity, &itemstack1);
        GL11::glPopMatrix();
    }
}

pub fn drawFirstPersonHand(this: &Render)
{
    let model: &ModelBiped = modelBipedMain(this);
    model.m_base.m_onGround.set(0.0);
    model.setRotationAngles(0.0, 0.0, 0.0, 0.0, 0.0, MODEL_SCALE);
    model.m_bipedRightArm.render(MODEL_SCALE);
}

fn applyTranslation(this: &Rc<Render>, entity: &Rc<Entity>, dx: f64, dy: f64, dz: f64)
{
    if entity.isEntityAlive() && entity.isPlayerSleeping()
    {
        let player: &entity_player::PlayerFields = entity_player::fields(entity);
        (RENDER_LIVING.applyTranslation)(
            this,
            entity,
            dx + player.m_sleepingCameraOffsetX.get() as f64,
            dy + player.m_sleepingCameraOffsetY.get() as f64,
            dz + player.m_sleepingCameraOffsetZ.get() as f64,
        );
    }
    else
    {
        (RENDER_LIVING.applyTranslation)(this, entity, dx, dy, dz);
    }
}

fn rotateCorpse(this: &Rc<Render>, entity: &Rc<Entity>, bob: f32, bodyYaw: f32, tickDelta: f32)
{
    if entity.isEntityAlive() && entity.isPlayerSleeping()
    {
        GL11::glRotatef(entity_player::getBedOrientationInDegrees(entity), 0.0, 1.0, 0.0);
        GL11::glRotatef(this.getDeathMaxRotation(entity), 0.0, 0.0, 1.0);
        GL11::glRotatef(270.0, 0.0, 1.0, 0.0);
    }
    else
    {
        (RENDER_LIVING.rotateCorpse)(this, entity, bob, bodyYaw, tickDelta);
    }
}

pub const RENDER_PLAYER: RenderClass = RenderClass {
    m_name: "RenderPlayer",
    m_parent: Some(&RENDER_LIVING),
    shouldRenderPass,
    doRender,
    passSpecialRender,
    renderEquippedItems,
    preRenderCallback: |_, _, _| {
        let f: f32 = 0.9375;
        GL11::glScalef(f, f, f);
    },
    applyTranslation,
    rotateCorpse,
    ..RENDER_LIVING
};
