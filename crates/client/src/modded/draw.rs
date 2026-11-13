use crate::client::Minecraft;
use crate::gui::{FontRenderer, Gui, GuiButton};
use crate::render::{RenderEngine, RenderHelper, RenderItem};
use game::item::ItemStack;
use jvm::throwable::OrNpe;
use lwjgl::opengl::{GL11, GL12};
use modloader::host::context::stackFrom;
use modloader::mod_protocol::abi::{DrawList, DrawOp};
use std::cell::OnceCell;
use std::rc::Rc;

// a button the guest drew, kept so a click can be matched against it
#[derive(Clone, Copy)]
pub(crate) struct ButtonRect
{
    pub m_id: i32,
    pub m_x: i32,
    pub m_y: i32,
    pub m_w: i32,
    pub m_h: i32,
}

impl ButtonRect
{
    pub(crate) fn contains(&self, x: i32, y: i32) -> bool
    {
        x >= self.m_x && y >= self.m_y && x < self.m_x + self.m_w && y < self.m_y + self.m_h
    }
}

// the light rotation GuiContainer sets before its slots
const ITEM_LIGHT_TILT: f32 = 120.0;
const OPAQUE_WHITE: u32 = 0xFFFFFFFF;

thread_local!
{
    static ITEMS: OnceCell<Rc<RenderItem>> = const { OnceCell::new() };
}

fn itemRenderer() -> Rc<RenderItem> { ITEMS.with(|items| Rc::clone(items.get_or_init(|| Rc::new(RenderItem::new())))) }

// a whole texture stretched over the rectangle, blended by its alpha times the tint's
fn drawImage(engine: &Rc<RenderEngine>, texture: &str, rect: [i32; 4], color: u32)
{
    let [alpha, red, green, blue] = color.to_be_bytes().map(|channel| f32::from(channel) / 255.0);
    GL11::glColor4f(red, green, blue, alpha);
    GL11::glEnable(GL11::GL_BLEND);
    GL11::glBlendFunc(GL11::GL_SRC_ALPHA, GL11::GL_ONE_MINUS_SRC_ALPHA);
    super::textures::bind(engine, texture);

    let tessellator: Rc<crate::render::Tessellator> = crate::render::Tessellator::instance();
    let [left, top] = [rect[0] as f64, rect[1] as f64];
    let [right, bottom] = [left + rect[2] as f64, top + rect[3] as f64];

    tessellator.startDrawingQuads();
    tessellator.addVertexWithUV(left, bottom, 0.0, 0.0, 1.0);
    tessellator.addVertexWithUV(right, bottom, 0.0, 1.0, 1.0);
    tessellator.addVertexWithUV(right, top, 0.0, 1.0, 0.0);
    tessellator.addVertexWithUV(left, top, 0.0, 0.0, 0.0);
    tessellator.draw();
    GL11::glDisable(GL11::GL_BLEND);
    GL11::glColor4f(1.0, 1.0, 1.0, 1.0);
}

// lit and depth tested as GuiContainer slots, then back to the flat state later ops expect
pub(crate) fn drawItem(font: &Rc<FontRenderer>, engine: &Rc<RenderEngine>, stack: &Rc<ItemStack>, x: i32, y: i32)
{
    let depth: bool = GL11::glIsEnabled(GL11::GL_DEPTH_TEST);

    GL11::glPushMatrix();
    GL11::glRotatef(ITEM_LIGHT_TILT, 1.0, 0.0, 0.0);
    RenderHelper::enableStandardItemLighting();
    GL11::glPopMatrix();
    GL11::glEnable(GL12::GL_RESCALE_NORMAL);
    GL11::glEnable(GL11::GL_DEPTH_TEST);
    itemRenderer().renderItemIntoGUI(font, engine, Some(stack), x, y);
    itemRenderer().renderItemOverlayIntoGUI(font, engine, Some(stack), x, y);
    GL11::glDisable(GL12::GL_RESCALE_NORMAL);
    RenderHelper::disableStandardItemLighting();
    GL11::glDisable(GL11::GL_LIGHTING);

    if !depth
    {
        GL11::glDisable(GL11::GL_DEPTH_TEST);
    }
}

// paints one guest draw list at an origin, returns the buttons it placed
pub(crate) fn draw(mc: &Rc<Minecraft>, list: &DrawList, originX: i32, originY: i32, mouse: [i32; 2])
-> Vec<ButtonRect>
{
    let gui: Gui = Gui::new();
    let font: Rc<FontRenderer> = mc.m_fontRenderer.get().orNpe();
    let engine: Rc<RenderEngine> = mc.m_renderEngine.get().orNpe();
    let mut buttons: Vec<ButtonRect> = Vec::new();

    for op in &list.ops
    {
        match op
        {
            DrawOp::Rect { x, y, w, h, color } =>
            {
                gui.drawRect(originX + x, originY + y, originX + x + w, originY + y + h, *color as i32)
            }

            DrawOp::Gradient { x, y, w, h, top, bottom } => gui.drawGradientRect(
                originX + x,
                originY + y,
                originX + x + w,
                originY + y + h,
                *top as i32,
                *bottom as i32,
            ),

            DrawOp::Text { x, y, text, color, shadow, centered } =>
            {
                let left: i32 = if *centered
                {
                    originX + x - font.getStringWidth(text) / 2
                }
                else
                {
                    originX + x
                };

                if *shadow
                {
                    font.drawStringWithShadow(text, left, originY + y, *color as i32);
                }
                else
                {
                    font.drawString(text, left, originY + y, *color as i32);
                }
            }

            DrawOp::Texture { texture, x, y, u, v, w, h } =>
            {
                GL11::glColor4f(1.0, 1.0, 1.0, 1.0);
                super::textures::bind(&engine, texture);
                gui.drawTexturedModalRect(originX + x, originY + y, *u, *v, *w, *h);
            }

            DrawOp::Image { texture, x, y, w, h } =>
            {
                drawImage(&engine, texture, [originX + x, originY + y, *w, *h], OPAQUE_WHITE)
            }

            DrawOp::TintedImage { texture, x, y, w, h, color } =>
            {
                drawImage(&engine, texture, [originX + x, originY + y, *w, *h], *color)
            }

            DrawOp::Region { texture, x, y, w, h, uv, size } =>
            {
                GL11::glColor4f(1.0, 1.0, 1.0, 1.0);
                super::textures::bind(&engine, texture);

                let tessellator: Rc<crate::render::Tessellator> = crate::render::Tessellator::instance();
                let [left, top] = [(originX + x) as f64, (originY + y) as f64];
                let [right, bottom] = [left + *w as f64, top + *h as f64];
                let [u, v, uw, vh] = uv.map(f64::from);
                let [width, height] = size.map(f64::from);
                let [u0, v0, u1, v1] = [u / width, v / height, (u + uw) / width, (v + vh) / height];

                tessellator.startDrawingQuads();
                tessellator.addVertexWithUV(left, bottom, 0.0, u0, v1);
                tessellator.addVertexWithUV(right, bottom, 0.0, u1, v1);
                tessellator.addVertexWithUV(right, top, 0.0, u1, v0);
                tessellator.addVertexWithUV(left, top, 0.0, u0, v0);
                tessellator.draw();
            }

            DrawOp::Item { x, y, stack } =>
            {
                if let Some(stack) = stackFrom(stack)
                {
                    drawItem(&font, &engine, &stack, originX + x, originY + y);
                }
            }

            DrawOp::Button { id, x, y, w, h, label, enabled } =>
            {
                let button: Rc<GuiButton> = GuiButton::withSize(*id, originX + x, originY + y, *w, *h, label);
                button.m_enabled.set(*enabled);
                button.drawButton(mc, mouse[0], mouse[1]);

                // vanilla GuiButton.mousePressed ignores a disabled button
                if *enabled
                {
                    buttons.push(ButtonRect { m_id: *id, m_x: originX + x, m_y: originY + y, m_w: *w, m_h: *h });
                }
            }

            DrawOp::Particle { name, pos, motion } =>
            {
                if let Some(world) = mc.m_theWorld.get()
                {
                    world.spawnParticle(name, pos[0], pos[1], pos[2], motion[0], motion[1], motion[2]);
                }
            }
        }
    }

    buttons
}
