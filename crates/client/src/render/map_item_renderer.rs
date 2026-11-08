use crate::gui::FontRenderer;
use crate::render::{RenderEngine, Tessellator};
use crate::settings::GameSettings;
use game::block::MapColor;
use game::entity::Entity;
use game::world::MapDataBase;
use jvm::image::BufferedImage;
use jvm::throwable::OrNpe;
use lwjgl::opengl::GL11;
use std::cell::RefCell;
use std::rc::Rc;

pub struct MapItemRenderer
{
    m_colors: RefCell<Vec<i32>>,
    m_texture: i32,
    m_options: Rc<GameSettings>,
    m_textRenderer: Rc<FontRenderer>,
}

impl MapItemRenderer
{
    pub fn new(
        textRenderer: Rc<FontRenderer>,
        options: Rc<GameSettings>,
        textureManager: &RenderEngine,
    ) -> MapItemRenderer
    {
        let texture: i32 = textureManager.allocateAndSetupTexture(Rc::new(BufferedImage::new(128, 128, 2)));
        MapItemRenderer {
            m_colors: RefCell::new(vec![0; 16384]),
            m_texture: texture,
            m_options: options,
            m_textRenderer: textRenderer,
        }
    }

    fn pixelColor(&self, n: i32, by: i8) -> i32
    {
        if by as i32 / 4 == 0
        {
            return (((n + n / 128) & 1) * 8 + 16) << 24;
        }
        let index: i32 = by as i32 / 4;
        let mapColors: Rc<game::block::map_color::MapColors> = MapColor::colors();
        if !(0..mapColors.m_mapColorArray.len() as i32).contains(&index)
        {
            jvm::throwable::throw(&jvm::throwable::ARRAY_INDEX, &index.to_string());
        }
        let n2: i32 = mapColors.m_mapColorArray[index as usize].as_ref().orNpe().m_colorValue;
        let n4: i32 = match by & 3
        {
            2 => 255,
            0 => 180,
            _ => 220,
        };
        let mut n5: i32 = (n2 >> 16 & 0xFF) * n4 / 255;
        let mut n6: i32 = (n2 >> 8 & 0xFF) * n4 / 255;
        let mut n7: i32 = (n2 & 0xFF) * n4 / 255;
        if self.m_options.m_anaglyph.get()
        {
            let n8: i32 = (n5 * 30 + n6 * 59 + n7 * 11) / 100;
            let n9: i32 = (n5 * 30 + n6 * 70) / 100;
            let n10: i32 = (n5 * 30 + n7 * 70) / 100;
            n5 = n8;
            n6 = n9;
            n7 = n10;
        }
        0xFF000000u32 as i32 | n5 << 16 | n6 << 8 | n7
    }

    pub fn render(&self, _player: &Rc<Entity>, textureManager: &RenderEngine, mapData: &MapDataBase)
    {
        {
            let source: std::cell::Ref<Vec<i8>> = mapData.m_map.m_colors.borrow();
            let mut colors: std::cell::RefMut<Vec<i32>> = self.m_colors.borrow_mut();
            for n in 0..16384
            {
                colors[n as usize] = self.pixelColor(n, source[n as usize]);
            }
        }
        textureManager.bind(&self.m_colors.borrow(), 128, 128, self.m_texture);
        let n: i32 = 0;
        let by: i32 = 0;
        let tessellator: Rc<Tessellator> = Tessellator::instance();
        let f: f32 = 0.0;
        GL11::glBindTexture(3553, self.m_texture);
        GL11::glEnable(3042);
        GL11::glDisable(3008);
        tessellator.startDrawingQuads();
        let depth: f64 = -0.01f32 as f64;
        tessellator.addVertexWithUV((n as f32 + f) as f64, ((by + 128) as f32 - f) as f64, depth, 0.0, 1.0);
        tessellator.addVertexWithUV(((n + 128) as f32 - f) as f64, ((by + 128) as f32 - f) as f64, depth, 1.0, 1.0);
        tessellator.addVertexWithUV(((n + 128) as f32 - f) as f64, (by as f32 + f) as f64, depth, 1.0, 0.0);
        tessellator.addVertexWithUV((n as f32 + f) as f64, (by as f32 + f) as f64, depth, 0.0, 0.0);
        tessellator.draw();
        GL11::glEnable(3008);
        GL11::glDisable(3042);
        textureManager.bindTexture(textureManager.getTexture("/misc/mapicons.png"));
        for mapCoord in mapData.m_map.m_decorations.borrow().iter()
        {
            GL11::glPushMatrix();
            GL11::glTranslatef(
                n as f32 + mapCoord.m_x as f32 / 2.0 + 64.0,
                by as f32 + mapCoord.m_z as f32 / 2.0 + 64.0,
                -0.02,
            );
            GL11::glRotatef((mapCoord.m_rotation as i32 * 360) as f32 / 16.0, 0.0, 0.0, 1.0);
            GL11::glScalef(4.0, 4.0, 3.0);
            GL11::glTranslatef(-0.125, 0.125, 0.0);
            let kind: i32 = mapCoord.m_type as i32;
            let f2: f64 = ((kind % 4) as f32 / 4.0) as f64;
            let f3: f64 = ((kind / 4) as f32 / 4.0) as f64;
            let f4: f64 = ((kind % 4 + 1) as f32 / 4.0) as f64;
            let f5: f64 = ((kind / 4 + 1) as f32 / 4.0) as f64;
            tessellator.startDrawingQuads();
            tessellator.addVertexWithUV(-1.0, 1.0, 0.0, f2, f3);
            tessellator.addVertexWithUV(1.0, 1.0, 0.0, f4, f3);
            tessellator.addVertexWithUV(1.0, -1.0, 0.0, f4, f5);
            tessellator.addVertexWithUV(-1.0, -1.0, 0.0, f2, f5);
            tessellator.draw();
            GL11::glPopMatrix();
        }
        GL11::glPushMatrix();
        GL11::glTranslatef(0.0, 0.0, -0.04);
        GL11::glScalef(1.0, 1.0, 1.0);
        self.m_textRenderer.drawString(&mapData.m_id, n, by, -16777216);
        GL11::glPopMatrix();
    }
}
