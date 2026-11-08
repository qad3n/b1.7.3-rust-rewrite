use crate::gui::FontRenderer;
use crate::model::SignModel;
use crate::render::tile::TileEntitySpecialRenderer;
use game::block::Block;
use game::tileentity::{TileEntity, tile_entity_sign};
use jvm::string::JString;
use jvm::throwable::OrNpe;
use lwjgl::opengl::GL11;
use std::rc::Rc;

const SIGN_SCALE: f32 = 0.6666667;
const TEXT_SCALE: f32 = 0.016666668;
const LINE_SPACING: i32 = 10;

pub struct TileEntitySignRenderer
{
    m_signModel: SignModel,
}

impl TileEntitySignRenderer
{
    pub fn new() -> TileEntitySignRenderer { TileEntitySignRenderer { m_signModel: SignModel::new() } }
}

impl TileEntitySpecialRenderer for TileEntitySignRenderer
{
    fn renderTileEntityAt(&self, tileEntity: &Rc<TileEntity>, d: f64, d2: f64, d3: f64, _f: f32)
    {
        let block: Option<Rc<Block>> = tileEntity.getBlockType();
        GL11::glPushMatrix();
        let f3: f32 = SIGN_SCALE;
        GL11::glTranslatef(d as f32 + 0.5, d2 as f32 + 0.75 * f3, d3 as f32 + 0.5);
        if block.is_some_and(|block| Rc::ptr_eq(&block, &Block::signPost()))
        {
            let f4: f32 = (tileEntity.getBlockMetadata() * 360) as f32 / 16.0;
            GL11::glRotatef(-f4, 0.0, 1.0, 0.0);
            self.m_signModel.m_signStick.m_showModel.set(true);
        }
        else
        {
            let f2: f32 = match tileEntity.getBlockMetadata()
            {
                2 => 180.0,
                4 => 90.0,
                5 => -90.0,
                _ => 0.0,
            };
            GL11::glRotatef(-f2, 0.0, 1.0, 0.0);
            GL11::glTranslatef(0.0, -0.3125, -0.4375);
            self.m_signModel.m_signStick.m_showModel.set(false);
        }
        self.bindTextureByName("/item/sign.png");
        GL11::glPushMatrix();
        GL11::glScalef(f3, -f3, -f3);
        self.m_signModel.render();
        GL11::glPopMatrix();
        let fontRenderer: Rc<FontRenderer> = self.getFontRenderer().orNpe();
        let f2: f32 = TEXT_SCALE * f3;
        GL11::glTranslatef(0.0, 0.5 * f3, 0.07 * f3);
        GL11::glScalef(f2, -f2, f2);
        GL11::glNormal3f(0.0, 0.0, -f2);
        GL11::glDepthMask(false);
        let n: i32 = 0;
        let sign: &tile_entity_sign::SignExt = tile_entity_sign::fields(tileEntity);
        let lines: [Option<JString>; 4] = sign.m_signText.get();
        let count: i32 = lines.len() as i32;
        for (i, line) in lines.iter().enumerate()
        {
            let i: i32 = i as i32;
            let mut string: String = JString::display(line.as_ref());
            if i == sign.m_lineBeingEdited.get()
            {
                string = format!("> {string} <");
            }
            fontRenderer.drawString(
                &string,
                -fontRenderer.getStringWidth(&string) / 2,
                i * LINE_SPACING - count * 5,
                n,
            );
        }
        GL11::glDepthMask(true);
        GL11::glColor4f(1.0, 1.0, 1.0, 1.0);
        GL11::glPopMatrix();
    }
}
