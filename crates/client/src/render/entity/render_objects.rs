use crate::model::{ModelBase, ModelBoat, ModelMinecart};
use crate::render::entity::{RENDER, Render, RenderClass};
use crate::render::{RenderBlocks, RenderManager, Tessellator};
use game::block::Block;
use game::entity::{
    Entity, entity_arrow, entity_boat, entity_falling_sand, entity_fish, entity_living, entity_minecart,
    entity_painting, entity_tnt_primed,
};
use game::item::Item;
use game::math::{Vec3D, math_helper};
use jvm::random::Random;
use jvm::throwable::OrNpe;
use lwjgl::opengl::{GL11, GL12};
use std::cell::RefCell;
use std::f32::consts::PI;
use std::rc::Rc;

const MODEL_SCALE: f32 = 0.0625;
const PAINTING_SEED: i64 = 187;
const TEXTURE_SIZE: f32 = 256.0;

fn lerp(previous: f32, current: f32, tickDelta: f32) -> f32 { previous + (current - previous) * tickDelta }

fn plain(class: &'static RenderClass, shadowSize: f32, ext: Box<dyn std::any::Any>) -> Rc<Render>
{
    let render: Rc<Render> = Render::alloc(class, None, ext);
    render.m_shadowSize.set(shadowSize);
    render
}

pub fn newEntity() -> Rc<Render> { plain(&RENDER_ENTITY, 0.0, Box::new(())) }

pub const RENDER_ENTITY: RenderClass = RenderClass {
    m_name: "RenderEntity",
    m_parent: Some(&RENDER),
    doRender: |_, entity, dx, dy, dz, _, _| {
        GL11::glPushMatrix();
        Render::renderOffsetAABB(
            &entity.m_boundingBox,
            dx - entity.m_lastTickPosX.get(),
            dy - entity.m_lastTickPosY.get(),
            dz - entity.m_lastTickPosZ.get(),
        );
        GL11::glPopMatrix();
    },
    ..RENDER
};

pub fn newArrow() -> Rc<Render> { plain(&RENDER_ARROW, 0.0, Box::new(())) }

fn renderArrow(this: &Rc<Render>, entity: &Rc<Entity>, d: f64, d2: f64, d3: f64, _yaw: f32, f2: f32)
{
    if entity.m_prevRotationYaw.get() == 0.0 && entity.m_prevRotationPitch.get() == 0.0
    {
        return;
    }
    this.loadTexture("/item/arrows.png");
    GL11::glPushMatrix();
    GL11::glTranslatef(d as f32, d2 as f32, d3 as f32);
    GL11::glRotatef(lerp(entity.m_prevRotationYaw.get(), entity.m_rotationYaw.get(), f2) - 90.0, 0.0, 1.0, 0.0);
    GL11::glRotatef(lerp(entity.m_prevRotationPitch.get(), entity.m_rotationPitch.get(), f2), 0.0, 0.0, 1.0);
    let tessellator: Rc<Tessellator> = Tessellator::instance();
    let n: i32 = 0;
    let f3: f64 = 0.0;
    let f4: f64 = 0.5;
    let f5: f64 = ((0 + n * 10) as f32 / 32.0) as f64;
    let f6: f64 = ((5 + n * 10) as f32 / 32.0) as f64;
    let f7: f64 = 0.0;
    let f8: f64 = 0.15625;
    let f9: f64 = ((5 + n * 10) as f32 / 32.0) as f64;
    let f10: f64 = ((10 + n * 10) as f32 / 32.0) as f64;
    let f11: f32 = 0.05625;
    GL11::glEnable(GL12::GL_RESCALE_NORMAL);
    let f12: f32 = entity_arrow::fields(entity).m_arrowShake.get() as f32 - f2;
    if f12 > 0.0
    {
        let f13: f32 = -math_helper::sin(f12 * 3.0) * f12;
        GL11::glRotatef(f13, 0.0, 0.0, 1.0);
    }
    GL11::glRotatef(45.0, 1.0, 0.0, 0.0);
    GL11::glScalef(f11, f11, f11);
    GL11::glTranslatef(-4.0, 0.0, 0.0);
    GL11::glNormal3f(f11, 0.0, 0.0);
    tessellator.startDrawingQuads();
    tessellator.addVertexWithUV(-7.0, -2.0, -2.0, f7, f9);
    tessellator.addVertexWithUV(-7.0, -2.0, 2.0, f8, f9);
    tessellator.addVertexWithUV(-7.0, 2.0, 2.0, f8, f10);
    tessellator.addVertexWithUV(-7.0, 2.0, -2.0, f7, f10);
    tessellator.draw();
    GL11::glNormal3f(-f11, 0.0, 0.0);
    tessellator.startDrawingQuads();
    tessellator.addVertexWithUV(-7.0, 2.0, -2.0, f7, f9);
    tessellator.addVertexWithUV(-7.0, 2.0, 2.0, f8, f9);
    tessellator.addVertexWithUV(-7.0, -2.0, 2.0, f8, f10);
    tessellator.addVertexWithUV(-7.0, -2.0, -2.0, f7, f10);
    tessellator.draw();
    for _ in 0..4
    {
        GL11::glRotatef(90.0, 1.0, 0.0, 0.0);
        GL11::glNormal3f(0.0, 0.0, f11);
        tessellator.startDrawingQuads();
        tessellator.addVertexWithUV(-8.0, -2.0, 0.0, f3, f5);
        tessellator.addVertexWithUV(8.0, -2.0, 0.0, f4, f5);
        tessellator.addVertexWithUV(8.0, 2.0, 0.0, f4, f6);
        tessellator.addVertexWithUV(-8.0, 2.0, 0.0, f3, f6);
        tessellator.draw();
    }
    GL11::glDisable(GL12::GL_RESCALE_NORMAL);
    GL11::glPopMatrix();
}

pub const RENDER_ARROW: RenderClass =
    RenderClass { m_name: "RenderArrow", m_parent: Some(&RENDER), doRender: renderArrow, ..RENDER };

struct ModelRenderExt
{
    m_model: Rc<dyn ModelBase>,
}

fn model(this: &Render) -> &Rc<dyn ModelBase> { &this.ext::<ModelRenderExt>().m_model }

fn rockAfterHit(timeSinceHit: i32, currentDamage: i32, rockDirection: i32, tickDelta: f32)
{
    let f3: f32 = timeSinceHit as f32 - tickDelta;
    let mut f4: f32 = currentDamage as f32 - tickDelta;
    if f4 < 0.0
    {
        f4 = 0.0;
    }
    if f3 > 0.0
    {
        GL11::glRotatef(math_helper::sin(f3) * f3 * f4 / 10.0 * rockDirection as f32, 1.0, 0.0, 0.0);
    }
}

pub fn newBoat() -> Rc<Render>
{
    plain(&RENDER_BOAT, 0.5, Box::new(ModelRenderExt { m_model: Rc::new(ModelBoat::new()) }))
}

pub const RENDER_BOAT: RenderClass = RenderClass {
    m_name: "RenderBoat",
    m_parent: Some(&RENDER),
    doRender: |this, entity, d, d2, d3, f, f2| {
        GL11::glPushMatrix();
        GL11::glTranslatef(d as f32, d2 as f32, d3 as f32);
        GL11::glRotatef(180.0 - f, 0.0, 1.0, 0.0);
        let boat: &entity_boat::BoatFields = entity_boat::fields(entity);
        rockAfterHit(boat.m_boatTimeSinceHit.get(), boat.m_boatCurrentDamage.get(), boat.m_boatRockDirection.get(), f2);
        this.loadTexture("/terrain.png");
        let f5: f32 = 0.75;
        GL11::glScalef(f5, f5, f5);
        GL11::glScalef(1.0 / f5, 1.0 / f5, 1.0 / f5);
        this.loadTexture("/item/boat.png");
        GL11::glScalef(-1.0, -1.0, 1.0);
        model(this).render(0.0, 0.0, -0.1, 0.0, 0.0, MODEL_SCALE);
        GL11::glPopMatrix();
    },
    ..RENDER
};

pub fn newMinecart() -> Rc<Render>
{
    plain(&RENDER_MINECART, 0.5, Box::new(ModelRenderExt { m_model: Rc::new(ModelMinecart::new()) }))
}

fn renderMinecart(this: &Rc<Render>, entity: &Rc<Entity>, mut d: f64, mut d2: f64, mut d3: f64, mut f: f32, f2: f32)
{
    GL11::glPushMatrix();
    let tick: f64 = f2 as f64;
    let d4: f64 = entity.m_lastTickPosX.get() + (entity.posX() - entity.m_lastTickPosX.get()) * tick;
    let d5: f64 = entity.m_lastTickPosY.get() + (entity.posY() - entity.m_lastTickPosY.get()) * tick;
    let d6: f64 = entity.m_lastTickPosZ.get() + (entity.posZ() - entity.m_lastTickPosZ.get()) * tick;
    let d7: f64 = 0.3f32 as f64;
    let vec3D: Option<Rc<Vec3D>> = entity_minecart::snapPositionToRail(entity, d4, d5, d6);
    let mut f3: f32 = lerp(entity.m_prevRotationPitch.get(), entity.m_rotationPitch.get(), f2);
    if let Some(vec3D) = vec3D
    {
        let vec3D2: Rc<Vec3D> =
            entity_minecart::snapPositionToRailWithOffset(entity, d4, d5, d6, d7).unwrap_or_else(|| Rc::clone(&vec3D));
        let vec3D3: Rc<Vec3D> =
            entity_minecart::snapPositionToRailWithOffset(entity, d4, d5, d6, -d7).unwrap_or_else(|| Rc::clone(&vec3D));
        d += vec3D.x() - d4;
        d2 += (vec3D2.y() + vec3D3.y()) / 2.0 - d5;
        d3 += vec3D.z() - d6;
        let mut vec3D4: Rc<Vec3D> = vec3D3.addVector(-vec3D2.x(), -vec3D2.y(), -vec3D2.z());
        if vec3D4.lengthVector() != 0.0
        {
            vec3D4 = vec3D4.normalize();
            f = (jvm::math::atan2(vec3D4.z(), vec3D4.x()) * 180.0 / std::f64::consts::PI) as f32;
            f3 = (jvm::math::atan(vec3D4.y()) * 73.0) as f32;
        }
    }
    GL11::glTranslatef(d as f32, d2 as f32, d3 as f32);
    GL11::glRotatef(180.0 - f, 0.0, 1.0, 0.0);
    GL11::glRotatef(-f3, 0.0, 0.0, 1.0);
    let cart: &entity_minecart::MinecartFields = entity_minecart::fields(entity);
    rockAfterHit(
        cart.m_minecartTimeSinceHit.get(),
        cart.m_minecartCurrentDamage.get(),
        cart.m_minecartRockDirection.get(),
        f2,
    );
    let minecartType: i32 = cart.m_minecartType.get();
    if minecartType != 0
    {
        this.loadTexture("/terrain.png");
        let f6: f32 = 0.75;
        GL11::glScalef(f6, f6, f6);
        GL11::glTranslatef(0.0, 0.3125, 0.0);
        GL11::glRotatef(90.0, 0.0, 1.0, 0.0);
        let cargo: Option<Rc<Block>> = match minecartType
        {
            1 => Some(Block::chest()),
            2 => Some(Block::stoneOvenIdle()),
            _ => None,
        };
        if let Some(cargo) = cargo
        {
            RenderBlocks::empty().renderBlockOnInventory(&cargo, 0, entity.getEntityBrightness(f2));
        }
        GL11::glRotatef(-90.0, 0.0, 1.0, 0.0);
        GL11::glTranslatef(0.0, -0.3125, 0.0);
        GL11::glScalef(1.0 / f6, 1.0 / f6, 1.0 / f6);
    }
    this.loadTexture("/item/cart.png");
    GL11::glScalef(-1.0, -1.0, 1.0);
    model(this).render(0.0, 0.0, -0.1, 0.0, 0.0, MODEL_SCALE);
    GL11::glPopMatrix();
}

pub const RENDER_MINECART: RenderClass =
    RenderClass { m_name: "RenderMinecart", m_parent: Some(&RENDER), doRender: renderMinecart, ..RENDER };

struct BlockRenderExt
{
    m_blockRenderer: RefCell<RenderBlocks>,
}

fn blockRenderer(this: &Render) -> std::cell::RefMut<'_, RenderBlocks>
{
    this.ext::<BlockRenderExt>().m_blockRenderer.borrow_mut()
}

fn blockExt() -> Box<BlockRenderExt>
{
    Box::new(BlockRenderExt { m_blockRenderer: RefCell::new(RenderBlocks::empty()) })
}

pub fn newTNTPrimed() -> Rc<Render> { plain(&RENDER_TNT_PRIMED, 0.5, blockExt()) }

pub const RENDER_TNT_PRIMED: RenderClass = RenderClass {
    m_name: "RenderTNTPrimed",
    m_parent: Some(&RENDER),
    doRender: |this, entity, d, d2, d3, _, f2| {
        GL11::glPushMatrix();
        GL11::glTranslatef(d as f32, d2 as f32, d3 as f32);
        let fuse: i32 = entity_tnt_primed::fields(entity).m_fuse.get();
        if fuse as f32 - f2 + 1.0 < 10.0
        {
            let mut f3: f32 = (1.0 - (fuse as f32 - f2 + 1.0) / 10.0).clamp(0.0, 1.0);
            f3 *= f3;
            f3 *= f3;
            let f4: f32 = 1.0 + f3 * 0.3;
            GL11::glScalef(f4, f4, f4);
        }
        let f3: f32 = (1.0 - (fuse as f32 - f2 + 1.0) / 100.0) * 0.8;
        this.loadTexture("/terrain.png");
        blockRenderer(this).renderBlockOnInventory(&Block::tnt(), 0, entity.getEntityBrightness(f2));
        if fuse / 5 % 2 == 0
        {
            GL11::glDisable(GL11::GL_TEXTURE_2D);
            GL11::glDisable(GL11::GL_LIGHTING);
            GL11::glEnable(GL11::GL_BLEND);
            GL11::glBlendFunc(GL11::GL_SRC_ALPHA, GL11::GL_DST_ALPHA);
            GL11::glColor4f(1.0, 1.0, 1.0, f3);
            blockRenderer(this).renderBlockOnInventory(&Block::tnt(), 0, 1.0);
            GL11::glColor4f(1.0, 1.0, 1.0, 1.0);
            GL11::glDisable(GL11::GL_BLEND);
            GL11::glEnable(GL11::GL_LIGHTING);
            GL11::glEnable(GL11::GL_TEXTURE_2D);
        }
        GL11::glPopMatrix();
    },
    ..RENDER
};

pub fn newFallingSand() -> Rc<Render> { plain(&RENDER_FALLING_SAND, 0.5, blockExt()) }

pub const RENDER_FALLING_SAND: RenderClass = RenderClass {
    m_name: "RenderFallingSand",
    m_parent: Some(&RENDER),
    doRender: |this, entity, d, d2, d3, _, _| {
        GL11::glPushMatrix();
        GL11::glTranslatef(d as f32, d2 as f32, d3 as f32);
        this.loadTexture("/terrain.png");
        let block: Option<Rc<Block>> = Block::byId(entity_falling_sand::fields(entity).m_blockID.get());
        let world = entity_falling_sand::getWorld(entity).orNpe();
        GL11::glDisable(GL11::GL_LIGHTING);
        blockRenderer(this).renderBlockFallingSand(
            &block.orNpe(),
            &world,
            math_helper::floor_double(entity.posX()),
            math_helper::floor_double(entity.posY()),
            math_helper::floor_double(entity.posZ()),
        );
        GL11::glEnable(GL11::GL_LIGHTING);
        GL11::glPopMatrix();
    },
    ..RENDER
};

fn renderBillboard(manager: &RenderManager, u0: f32, u1: f32, v0: f32, v1: f32, xOffset: f32, yOffset: f32)
{
    let tessellator: Rc<Tessellator> = Tessellator::instance();
    let f5: f32 = 1.0;
    GL11::glRotatef(180.0 - manager.m_playerViewY.get(), 0.0, 1.0, 0.0);
    GL11::glRotatef(-manager.m_playerViewX.get(), 1.0, 0.0, 0.0);
    tessellator.startDrawingQuads();
    tessellator.setNormal(0.0, 1.0, 0.0);
    tessellator.addVertexWithUV((0.0 - xOffset) as f64, (0.0 - yOffset) as f64, 0.0, u0 as f64, v1 as f64);
    tessellator.addVertexWithUV((f5 - xOffset) as f64, (0.0 - yOffset) as f64, 0.0, u1 as f64, v1 as f64);
    tessellator.addVertexWithUV((f5 - xOffset) as f64, (1.0 - yOffset) as f64, 0.0, u1 as f64, v0 as f64);
    tessellator.addVertexWithUV((0.0 - xOffset) as f64, (1.0 - yOffset) as f64, 0.0, u0 as f64, v0 as f64);
    tessellator.draw();
}

fn renderItemIcon(this: &Render, d: f64, d2: f64, d3: f64, scale: f32, icon: i32)
{
    GL11::glPushMatrix();
    GL11::glTranslatef(d as f32, d2 as f32, d3 as f32);
    GL11::glEnable(GL12::GL_RESCALE_NORMAL);
    GL11::glScalef(scale, scale, scale);
    this.loadTexture("/gui/items.png");
    let u0: f32 = (icon % 16 * 16 + 0) as f32 / TEXTURE_SIZE;
    let u1: f32 = (icon % 16 * 16 + 16) as f32 / TEXTURE_SIZE;
    let v0: f32 = (icon / 16 * 16 + 0) as f32 / TEXTURE_SIZE;
    let v1: f32 = (icon / 16 * 16 + 16) as f32 / TEXTURE_SIZE;
    renderBillboard(&this.renderManager(), u0, u1, v0, v1, 0.5, 0.25);
    GL11::glDisable(GL12::GL_RESCALE_NORMAL);
    GL11::glPopMatrix();
}

pub fn newFireball() -> Rc<Render> { plain(&RENDER_FIREBALL, 0.0, Box::new(())) }

pub const RENDER_FIREBALL: RenderClass = RenderClass {
    m_name: "RenderFireball",
    m_parent: Some(&RENDER),
    doRender: |this, _, d, d2, d3, _, _| {
        let f3: f32 = 2.0;
        renderItemIcon(this, d, d2, d3, f3 / 1.0, Item::snowball().getIconFromDamage(0));
    },
    ..RENDER
};

struct SnowballRenderExt
{
    m_itemIconIndex: i32,
}

pub fn newSnowball(sprite: i32) -> Rc<Render>
{
    plain(&RENDER_SNOWBALL, 0.0, Box::new(SnowballRenderExt { m_itemIconIndex: sprite }))
}

pub const RENDER_SNOWBALL: RenderClass = RenderClass {
    m_name: "RenderSnowball",
    m_parent: Some(&RENDER),
    doRender: |this, _, dx, dy, dz, _, _| {
        renderItemIcon(this, dx, dy, dz, 0.5, this.ext::<SnowballRenderExt>().m_itemIconIndex);
    },
    ..RENDER
};

pub fn newFish() -> Rc<Render> { plain(&RENDER_FISH, 0.0, Box::new(())) }

fn renderFish(this: &Rc<Render>, entity: &Rc<Entity>, d: f64, d2: f64, d3: f64, _f: f32, f2: f32)
{
    let manager: Rc<RenderManager> = this.renderManager();
    GL11::glPushMatrix();
    GL11::glTranslatef(d as f32, d2 as f32, d3 as f32);
    GL11::glEnable(GL12::GL_RESCALE_NORMAL);
    GL11::glScalef(0.5, 0.5, 0.5);
    let n: i32 = 1;
    let n2: i32 = 2;
    this.loadTexture("/particles.png");
    let f3: f32 = (n * 8 + 0) as f32 / 128.0;
    let f4: f32 = (n * 8 + 8) as f32 / 128.0;
    let f5: f32 = (n2 * 8 + 0) as f32 / 128.0;
    let f6: f32 = (n2 * 8 + 8) as f32 / 128.0;
    renderBillboard(&manager, f3, f4, f5, f6, 0.5, 0.5);
    GL11::glDisable(GL12::GL_RESCALE_NORMAL);
    GL11::glPopMatrix();
    let Some(angler) = entity_fish::fields(entity).m_throwable.m_thrower.get()
    else
    {
        return;
    };
    let tick: f64 = f2 as f64;
    let mut f10: f32 = lerp(angler.m_prevRotationYaw.get(), angler.m_rotationYaw.get(), f2) * PI / 180.0;
    let mut d4: f64 = math_helper::sin(f10) as f64;
    let mut d5: f64 = math_helper::cos(f10) as f64;
    let f11: f32 = entity_living::getSwingProgress(&angler, f2);
    let f12: f32 = math_helper::sin(math_helper::sqrt_float(f11) * PI);
    let vec3D: Rc<Vec3D> = Vec3D::createVector(-0.5, 0.03, 0.8);
    vec3D.rotateAroundX(-lerp(angler.m_prevRotationPitch.get(), angler.m_rotationPitch.get(), f2) * PI / 180.0);
    vec3D.rotateAroundY(-lerp(angler.m_prevRotationYaw.get(), angler.m_rotationYaw.get(), f2) * PI / 180.0);
    vec3D.rotateAroundY(f12 * 0.5);
    vec3D.rotateAroundX(-f12 * 0.7);
    let anglerX: f64 = angler.m_prevPosX.get() + (angler.posX() - angler.m_prevPosX.get()) * tick;
    let anglerY: f64 = angler.m_prevPosY.get() + (angler.posY() - angler.m_prevPosY.get()) * tick;
    let anglerZ: f64 = angler.m_prevPosZ.get() + (angler.posZ() - angler.m_prevPosZ.get()) * tick;
    let mut d6: f64 = anglerX + vec3D.x();
    let mut d7: f64 = anglerY + vec3D.y();
    let mut d8: f64 = anglerZ + vec3D.z();
    if manager.m_options.get().orNpe().m_thirdPersonView.get()
    {
        let living: &entity_living::LivingFields = entity_living::fields(&angler);
        f10 = lerp(living.m_prevRenderYawOffset.get(), living.m_renderYawOffset.get(), f2) * PI / 180.0;
        d4 = math_helper::sin(f10) as f64;
        d5 = math_helper::cos(f10) as f64;
        d6 = anglerX - d5 * 0.35 - d4 * 0.85;
        d7 = anglerY - 0.45;
        d8 = anglerZ - d4 * 0.35 + d5 * 0.85;
    }
    let d9: f64 = entity.m_prevPosX.get() + (entity.posX() - entity.m_prevPosX.get()) * tick;
    let d10: f64 = entity.m_prevPosY.get() + (entity.posY() - entity.m_prevPosY.get()) * tick + 0.25;
    let d11: f64 = entity.m_prevPosZ.get() + (entity.posZ() - entity.m_prevPosZ.get()) * tick;
    let d12: f64 = (d6 - d9) as f32 as f64;
    let d13: f64 = (d7 - d10) as f32 as f64;
    let d14: f64 = (d8 - d11) as f32 as f64;
    GL11::glDisable(GL11::GL_TEXTURE_2D);
    GL11::glDisable(GL11::GL_LIGHTING);
    let tessellator: Rc<Tessellator> = Tessellator::instance();
    tessellator.startDrawing(GL11::GL_LINE_STRIP);
    tessellator.setColorOpaque_I(0);
    let n3: i32 = 16;
    for i in 0..=n3
    {
        let f13: f32 = i as f32 / n3 as f32;
        tessellator.addVertex(
            d + d12 * f13 as f64,
            d2 + d13 * (f13 * f13 + f13) as f64 * 0.5 + 0.25,
            d3 + d14 * f13 as f64,
        );
    }
    tessellator.draw();
    GL11::glEnable(GL11::GL_LIGHTING);
    GL11::glEnable(GL11::GL_TEXTURE_2D);
}

pub const RENDER_FISH: RenderClass =
    RenderClass { m_name: "RenderFish", m_parent: Some(&RENDER), doRender: renderFish, ..RENDER };

pub fn newLightningBolt() -> Rc<Render> { plain(&RENDER_LIGHTNING_BOLT, 0.0, Box::new(())) }

fn renderLightningBolt(_this: &Rc<Render>, entity: &Rc<Entity>, d: f64, d2: f64, d3: f64, _f: f32, _f2: f32)
{
    let tessellator: Rc<Tessellator> = Tessellator::instance();
    GL11::glDisable(GL11::GL_TEXTURE_2D);
    GL11::glDisable(GL11::GL_LIGHTING);
    GL11::glEnable(GL11::GL_BLEND);
    GL11::glBlendFunc(GL11::GL_SRC_ALPHA, GL11::GL_ONE);
    let seed: i64 = game::entity::entity_lightning_bolt::fields(entity).m_seed.get();
    let mut dArray: [f64; 8] = [0.0; 8];
    let mut dArray2: [f64; 8] = [0.0; 8];
    let mut d4: f64 = 0.0;
    let mut d5: f64 = 0.0;
    let random: Random = Random::withSeed(seed);
    for i in (0..8).rev()
    {
        dArray[i] = d4;
        dArray2[i] = d5;
        d4 += (random.nextIntBounded(11) - 5) as f64;
        d5 += (random.nextIntBounded(11) - 5) as f64;
    }
    for i in 0..4
    {
        let random2: Random = Random::withSeed(seed);
        for j in 0..3
        {
            let n: i32 = if j > 0 { 7 - j } else { 7 };
            let n2: i32 = if j > 0 { n - 2 } else { 0 };
            let mut d6: f64 = dArray[n as usize] - d4;
            let mut d7: f64 = dArray2[n as usize] - d5;
            for k in (n2..=n).rev()
            {
                let d8: f64 = d6;
                let d9: f64 = d7;
                if j == 0
                {
                    d6 += (random2.nextIntBounded(11) - 5) as f64;
                    d7 += (random2.nextIntBounded(11) - 5) as f64;
                }
                else
                {
                    d6 += (random2.nextIntBounded(31) - 15) as f64;
                    d7 += (random2.nextIntBounded(31) - 15) as f64;
                }
                tessellator.startDrawing(GL11::GL_TRIANGLE_STRIP);
                let f3: f32 = 0.5;
                tessellator.setColorRGBA_F(0.9 * f3, 0.9 * f3, 1.0 * f3, 0.3);
                let mut d10: f64 = 0.1 + i as f64 * 0.2;
                if j == 0
                {
                    d10 *= k as f64 * 0.1 + 1.0;
                }
                let mut d11: f64 = 0.1 + i as f64 * 0.2;
                if j == 0
                {
                    d11 *= (k - 1) as f64 * 0.1 + 1.0;
                }
                for i2 in 0..5
                {
                    let mut d12: f64 = d + 0.5 - d10;
                    let mut d13: f64 = d3 + 0.5 - d10;
                    if i2 == 1 || i2 == 2
                    {
                        d12 += d10 * 2.0;
                    }
                    if i2 == 2 || i2 == 3
                    {
                        d13 += d10 * 2.0;
                    }
                    let mut d14: f64 = d + 0.5 - d11;
                    let mut d15: f64 = d3 + 0.5 - d11;
                    if i2 == 1 || i2 == 2
                    {
                        d14 += d11 * 2.0;
                    }
                    if i2 == 2 || i2 == 3
                    {
                        d15 += d11 * 2.0;
                    }
                    tessellator.addVertex(d14 + d6, d2 + (k * 16) as f64, d15 + d7);
                    tessellator.addVertex(d12 + d8, d2 + ((k + 1) * 16) as f64, d13 + d9);
                }
                tessellator.draw();
            }
        }
    }
    GL11::glDisable(GL11::GL_BLEND);
    GL11::glEnable(GL11::GL_LIGHTING);
    GL11::glEnable(GL11::GL_TEXTURE_2D);
}

pub const RENDER_LIGHTNING_BOLT: RenderClass =
    RenderClass { m_name: "RenderLightningBolt", m_parent: Some(&RENDER), doRender: renderLightningBolt, ..RENDER };

struct PaintingRenderExt
{
    m_rand: Random,
}

pub fn newPainting() -> Rc<Render>
{
    plain(&RENDER_PAINTING, 0.0, Box::new(PaintingRenderExt { m_rand: Random::new() }))
}

fn applyBrightness(manager: &RenderManager, painting: &Rc<Entity>, u: f32, v: f32)
{
    let mut n: i32 = math_helper::floor_double(painting.posX());
    let n2: i32 = math_helper::floor_double(painting.posY() + (v / 16.0) as f64);
    let mut n3: i32 = math_helper::floor_double(painting.posZ());
    match entity_painting::fields(painting).m_direction.get()
    {
        0 => n = math_helper::floor_double(painting.posX() + (u / 16.0) as f64),
        1 => n3 = math_helper::floor_double(painting.posZ() - (u / 16.0) as f64),
        2 => n = math_helper::floor_double(painting.posX() - (u / 16.0) as f64),
        3 => n3 = math_helper::floor_double(painting.posZ() + (u / 16.0) as f64),
        _ =>
        {}
    }
    let f: f32 = manager.m_worldObj.get().orNpe().getLightBrightness(n, n2, n3);
    GL11::glColor3f(f, f, f);
}

fn renderPaintingTiles(manager: &RenderManager, painting: &Rc<Entity>, width: i32, height: i32, u: i32, v: i32)
{
    let f: f32 = -width as f32 / 2.0;
    let f2: f32 = -height as f32 / 2.0;
    let f3: f64 = -0.5;
    let f4: f64 = 0.5;
    let tessellator: Rc<Tessellator> = Tessellator::instance();
    for i in 0..width / 16
    {
        for j in 0..height / 16
        {
            let f5: f32 = f + ((i + 1) * 16) as f32;
            let f6: f32 = f + (i * 16) as f32;
            let f7: f32 = f2 + ((j + 1) * 16) as f32;
            let f8: f32 = f2 + (j * 16) as f32;
            applyBrightness(manager, painting, (f5 + f6) / 2.0, (f7 + f8) / 2.0);
            let f9: f64 = ((u + width - i * 16) as f32 / TEXTURE_SIZE) as f64;
            let f10: f64 = ((u + width - (i + 1) * 16) as f32 / TEXTURE_SIZE) as f64;
            let f11: f64 = ((v + height - j * 16) as f32 / TEXTURE_SIZE) as f64;
            let f12: f64 = ((v + height - (j + 1) * 16) as f32 / TEXTURE_SIZE) as f64;
            let (f13, f14, f15, f16): (f64, f64, f64, f64) = (0.75, 0.8125, 0.0, 0.0625);
            let (f17, f18, f19, f20): (f64, f64, f64, f64) = (0.75, 0.8125, 0.001953125, 0.001953125);
            let (f21, f22, f23, f24): (f64, f64, f64, f64) = (0.7519531f32 as f64, 0.7519531f32 as f64, 0.0, 0.0625);
            let (x5, x6, y7, y8): (f64, f64, f64, f64) = (f5 as f64, f6 as f64, f7 as f64, f8 as f64);
            tessellator.startDrawingQuads();
            tessellator.setNormal(0.0, 0.0, -1.0);
            tessellator.addVertexWithUV(x5, y8, f3, f10, f11);
            tessellator.addVertexWithUV(x6, y8, f3, f9, f11);
            tessellator.addVertexWithUV(x6, y7, f3, f9, f12);
            tessellator.addVertexWithUV(x5, y7, f3, f10, f12);
            tessellator.setNormal(0.0, 0.0, 1.0);
            tessellator.addVertexWithUV(x5, y7, f4, f13, f15);
            tessellator.addVertexWithUV(x6, y7, f4, f14, f15);
            tessellator.addVertexWithUV(x6, y8, f4, f14, f16);
            tessellator.addVertexWithUV(x5, y8, f4, f13, f16);
            tessellator.setNormal(0.0, -1.0, 0.0);
            tessellator.addVertexWithUV(x5, y7, f3, f17, f19);
            tessellator.addVertexWithUV(x6, y7, f3, f18, f19);
            tessellator.addVertexWithUV(x6, y7, f4, f18, f20);
            tessellator.addVertexWithUV(x5, y7, f4, f17, f20);
            tessellator.setNormal(0.0, 1.0, 0.0);
            tessellator.addVertexWithUV(x5, y8, f4, f17, f19);
            tessellator.addVertexWithUV(x6, y8, f4, f18, f19);
            tessellator.addVertexWithUV(x6, y8, f3, f18, f20);
            tessellator.addVertexWithUV(x5, y8, f3, f17, f20);
            tessellator.setNormal(-1.0, 0.0, 0.0);
            tessellator.addVertexWithUV(x5, y7, f4, f22, f23);
            tessellator.addVertexWithUV(x5, y8, f4, f22, f24);
            tessellator.addVertexWithUV(x5, y8, f3, f21, f24);
            tessellator.addVertexWithUV(x5, y7, f3, f21, f23);
            tessellator.setNormal(1.0, 0.0, 0.0);
            tessellator.addVertexWithUV(x6, y7, f3, f22, f23);
            tessellator.addVertexWithUV(x6, y8, f3, f22, f24);
            tessellator.addVertexWithUV(x6, y8, f4, f21, f24);
            tessellator.addVertexWithUV(x6, y7, f4, f21, f23);
            tessellator.draw();
        }
    }
}

pub const RENDER_PAINTING: RenderClass = RenderClass {
    m_name: "RenderPainting",
    m_parent: Some(&RENDER),
    doRender: |this, entity, d, d2, d3, f, _| {
        this.ext::<PaintingRenderExt>().m_rand.setSeed(PAINTING_SEED);
        GL11::glPushMatrix();
        GL11::glTranslatef(d as f32, d2 as f32, d3 as f32);
        GL11::glRotatef(f, 0.0, 1.0, 0.0);
        GL11::glEnable(GL12::GL_RESCALE_NORMAL);
        this.loadTexture("/art/kz.png");
        let art: &entity_painting::EnumArt = entity_painting::fields(entity).m_art.get().orNpe();
        let f3: f32 = 0.0625;
        GL11::glScalef(f3, f3, f3);
        renderPaintingTiles(&this.renderManager(), entity, art.m_sizeX, art.m_sizeY, art.m_offsetX, art.m_offsetY);
        GL11::glDisable(GL12::GL_RESCALE_NORMAL);
        GL11::glPopMatrix();
    },
    ..RENDER
};
