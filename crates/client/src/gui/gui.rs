use crate::gui::FontRenderer;
use crate::render::Tessellator;
use lwjgl::opengl::GL11;
use std::cell::Cell;
use std::rc::Rc;

mod panel
{
    pub const FACE: i32 = 0xFFC6C6C6u32 as i32;
    pub const LIGHT: i32 = 0xFFFFFFFFu32 as i32;
    pub const SHADOW: i32 = 0xFF555555u32 as i32;
    pub const EDGE: i32 = 0xFF000000u32 as i32;
}

#[derive(Default)]
pub struct Gui
{
    pub m_zLevel: Cell<f32>,
}

fn channels(color: i32) -> (f32, f32, f32, f32)
{
    (
        (color >> 24 & 0xFF) as f32 / 255.0,
        (color >> 16 & 0xFF) as f32 / 255.0,
        (color >> 8 & 0xFF) as f32 / 255.0,
        (color & 0xFF) as f32 / 255.0,
    )
}

impl Gui
{
    pub fn new() -> Self { Self::default() }

    pub fn drawHorizontalLine(&self, mut x1: i32, mut x2: i32, y: i32, color: i32)
    {
        if x2 < x1
        {
            std::mem::swap(&mut x1, &mut x2);
        }

        self.drawRect(x1, y, x2 + 1, y + 1, color);
    }

    pub fn drawVerticalLine(&self, x: i32, mut y1: i32, mut y2: i32, color: i32)
    {
        if y2 < y1
        {
            std::mem::swap(&mut y1, &mut y2);
        }

        self.drawRect(x, y1 + 1, x + 1, y2, color);
    }

    pub fn drawRect(&self, mut x1: i32, mut y1: i32, mut x2: i32, mut y2: i32, color: i32)
    {
        if x1 < x2
        {
            std::mem::swap(&mut x1, &mut x2);
        }

        if y1 < y2
        {
            std::mem::swap(&mut y1, &mut y2);
        }

        let (f, f2, f3, f4): (f32, f32, f32, f32) = channels(color);
        let tessellator: Rc<Tessellator> = Tessellator::instance();

        GL11::glEnable(3042);
        GL11::glDisable(3553);
        GL11::glBlendFunc(770, 771);
        GL11::glColor4f(f2, f3, f4, f);
        tessellator.startDrawingQuads();
        tessellator.addVertex(x1 as f64, y2 as f64, 0.0);
        tessellator.addVertex(x2 as f64, y2 as f64, 0.0);
        tessellator.addVertex(x2 as f64, y1 as f64, 0.0);
        tessellator.addVertex(x1 as f64, y1 as f64, 0.0);
        tessellator.draw();
        GL11::glEnable(3553);
        GL11::glDisable(3042);
    }

    // rewrite: a container background as the gui sheets paint it with rounded corners at any size
    pub fn drawPanel(&self, x: i32, y: i32, width: i32, height: i32)
    {
        let (w, h): (i32, i32) = (width, height);
        let rects: [([i32; 4], i32); 19] = [
            ([2, 2, w - 4, h - 4], panel::FACE),
            ([2, 1, w - 5, 1], panel::LIGHT),
            ([1, 2, w - 4, 1], panel::LIGHT),
            ([1, 3, 2, h - 6], panel::LIGHT),
            ([3, 3, 1, 1], panel::LIGHT),
            ([w - 3, 3, 2, h - 5], panel::SHADOW),
            ([3, h - 3, w - 4, 1], panel::SHADOW),
            ([3, h - 2, w - 5, 1], panel::SHADOW),
            ([w - 4, h - 4, 1, 1], panel::SHADOW),
            ([2, 0, w - 5, 1], panel::EDGE),
            ([0, 2, 1, h - 5], panel::EDGE),
            ([3, h - 1, w - 5, 1], panel::EDGE),
            ([w - 1, 3, 1, h - 5], panel::EDGE),
            ([1, 1, 1, 1], panel::EDGE),
            ([w - 3, 1, 1, 1], panel::EDGE),
            ([w - 2, 2, 1, 1], panel::EDGE),
            ([1, h - 3, 1, 1], panel::EDGE),
            ([2, h - 2, 1, 1], panel::EDGE),
            ([w - 2, h - 2, 1, 1], panel::EDGE),
        ];

        for ([left, top, rectWidth, rectHeight], color) in rects
        {
            self.drawRect(x + left, y + top, x + left + rectWidth, y + top + rectHeight, color);
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn drawGradientRect(&self, x1: i32, y1: i32, x2: i32, y2: i32, color1: i32, color2: i32)
    {
        let (f, f2, f3, f4): (f32, f32, f32, f32) = channels(color1);
        let (f5, f6, f7, f8): (f32, f32, f32, f32) = channels(color2);

        GL11::glDisable(3553);
        GL11::glEnable(3042);
        GL11::glDisable(3008);
        GL11::glBlendFunc(770, 771);
        GL11::glShadeModel(7425);

        let tessellator: Rc<Tessellator> = Tessellator::instance();
        tessellator.startDrawingQuads();
        tessellator.setColorRGBA_F(f2, f3, f4, f);
        tessellator.addVertex(x2 as f64, y1 as f64, 0.0);
        tessellator.addVertex(x1 as f64, y1 as f64, 0.0);
        tessellator.setColorRGBA_F(f6, f7, f8, f5);
        tessellator.addVertex(x1 as f64, y2 as f64, 0.0);
        tessellator.addVertex(x2 as f64, y2 as f64, 0.0);
        tessellator.draw();
        GL11::glShadeModel(7424);
        GL11::glDisable(3042);
        GL11::glEnable(3008);
        GL11::glEnable(3553);
    }

    pub fn drawCenteredString(&self, textRenderer: &FontRenderer, text: &str, centerX: i32, y: i32, color: i32)
    {
        textRenderer.drawStringWithShadow(text, centerX - textRenderer.getStringWidth(text) / 2, y, color);
    }

    pub fn drawString(&self, textRenderer: &FontRenderer, text: &str, x: i32, y: i32, color: i32)
    {
        textRenderer.drawStringWithShadow(text, x, y, color);
    }

    pub fn drawTexturedModalRect(&self, x: i32, y: i32, u: i32, v: i32, width: i32, height: i32)
    {
        let f: f32 = 0.00390625;
        let f2: f32 = 0.00390625;
        let z: f64 = self.m_zLevel.get() as f64;
        let tessellator: Rc<Tessellator> = Tessellator::instance();
        tessellator.startDrawingQuads();
        tessellator.addVertexWithUV(
            x as f64,
            (y + height) as f64,
            z,
            (u as f32 * f) as f64,
            ((v + height) as f32 * f2) as f64,
        );
        tessellator.addVertexWithUV(
            (x + width) as f64,
            (y + height) as f64,
            z,
            ((u + width) as f32 * f) as f64,
            ((v + height) as f32 * f2) as f64,
        );
        tessellator.addVertexWithUV(
            (x + width) as f64,
            y as f64,
            z,
            ((u + width) as f32 * f) as f64,
            (v as f32 * f2) as f64,
        );
        tessellator.addVertexWithUV(x as f64, y as f64, z, (u as f32 * f) as f64, (v as f32 * f2) as f64);
        tessellator.draw();
    }
}
