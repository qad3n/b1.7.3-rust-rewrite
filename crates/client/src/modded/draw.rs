use crate::client::Minecraft;
use crate::gui::{FontRenderer, Gui, GuiButton};
use crate::render::{RenderEngine, RenderItem};
use jvm::throwable::OrNpe;
use lwjgl::opengl::GL11;
use modloader::host::context::stackFrom;
use modloader::mod_protocol::abi::{DrawList, DrawOp};
use std::cell::OnceCell;
use std::rc::Rc;

// a button the guest drew, kept so a click can be matched against it
#[derive(Clone, Copy)]
pub struct ButtonRect
{
    pub m_id: i32,
    pub m_x: i32,
    pub m_y: i32,
    pub m_w: i32,
    pub m_h: i32,
}

impl ButtonRect
{
    pub fn contains(&self, x: i32, y: i32) -> bool
    {
        x >= self.m_x && y >= self.m_y && x < self.m_x + self.m_w && y < self.m_y + self.m_h
    }
}

thread_local! {
    static ITEMS: OnceCell<Rc<RenderItem>> = const { OnceCell::new() };
}

fn itemRenderer() -> Rc<RenderItem> { ITEMS.with(|items| Rc::clone(items.get_or_init(|| Rc::new(RenderItem::new())))) }

fn texturePath(texture: &str) -> String
{
    modloader::entities::modTexturePath(texture).unwrap_or_else(|| texture.to_owned())
}

// paints one guest draw list at an origin, returns the buttons it placed
pub fn draw(mc: &Rc<Minecraft>, list: &DrawList, originX: i32, originY: i32, mouse: [i32; 2]) -> Vec<ButtonRect>
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
                engine.bindTexture(engine.getTexture(&texturePath(texture)));
                gui.drawTexturedModalRect(originX + x, originY + y, *u, *v, *w, *h);
            }
            DrawOp::Image { texture, x, y, w, h } =>
            {
                GL11::glColor4f(1.0, 1.0, 1.0, 1.0);
                GL11::glEnable(GL11::GL_BLEND);
                GL11::glBlendFunc(GL11::GL_SRC_ALPHA, GL11::GL_ONE_MINUS_SRC_ALPHA);
                engine.bindTexture(engine.getTexture(&texturePath(texture)));
                let tessellator: Rc<crate::render::Tessellator> = crate::render::Tessellator::instance();
                let [left, top] = [(originX + x) as f64, (originY + y) as f64];
                let [right, bottom] = [left + *w as f64, top + *h as f64];
                tessellator.startDrawingQuads();
                tessellator.addVertexWithUV(left, bottom, 0.0, 0.0, 1.0);
                tessellator.addVertexWithUV(right, bottom, 0.0, 1.0, 1.0);
                tessellator.addVertexWithUV(right, top, 0.0, 1.0, 0.0);
                tessellator.addVertexWithUV(left, top, 0.0, 0.0, 0.0);
                tessellator.draw();
                GL11::glDisable(GL11::GL_BLEND);
            }
            DrawOp::Region { texture, x, y, w, h, uv, size } =>
            {
                GL11::glColor4f(1.0, 1.0, 1.0, 1.0);
                engine.bindTexture(engine.getTexture(&texturePath(texture)));
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
                    itemRenderer().renderItemIntoGUI(&font, &engine, Some(&stack), originX + x, originY + y);
                    itemRenderer().renderItemOverlayIntoGUI(&font, &engine, Some(&stack), originX + x, originY + y);
                }
            }
            DrawOp::Button { id, x, y, w, h, label, enabled } =>
            {
                let button: Rc<GuiButton> = GuiButton::withSize(*id, originX + x, originY + y, *w, *h, label);
                button.m_enabled.set(*enabled);
                button.drawButton(mc, mouse[0], mouse[1]);
                buttons.push(ButtonRect { m_id: *id, m_x: originX + x, m_y: originY + y, m_w: *w, m_h: *h });
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
