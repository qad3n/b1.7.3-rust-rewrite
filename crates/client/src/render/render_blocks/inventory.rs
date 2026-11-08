use super::RenderBlocks;
use crate::render::Tessellator;
use game::block::Block;
use lwjgl::opengl::GL11;
use std::rc::Rc;

impl RenderBlocks
{
    fn renderInventoryCube(&mut self, block: &Rc<Block>, metadata: Option<i32>, inset: f32)
    {
        let tessellator: Rc<Tessellator> = Tessellator::instance();
        let sprite = |face: i32| match metadata
        {
            Some(metadata) => block.getBlockTextureFromSideAndMetadata(face, metadata),
            None => block.getBlockTextureFromSide(face),
        };
        tessellator.startDrawingQuads();
        tessellator.setNormal(0.0, -1.0, 0.0);
        self.renderBottomFace(block, 0.0, 0.0, 0.0, sprite(0));
        tessellator.draw();
        tessellator.startDrawingQuads();
        tessellator.setNormal(0.0, 1.0, 0.0);
        self.renderTopFace(block, 0.0, 0.0, 0.0, sprite(1));
        tessellator.draw();
        let sides: [(f32, f32, f32, f32, f32, f32, i32); 4] = [
            (0.0, 0.0, -1.0, 0.0, 0.0, inset, 2),
            (0.0, 0.0, 1.0, 0.0, 0.0, -inset, 3),
            (-1.0, 0.0, 0.0, inset, 0.0, 0.0, 4),
            (1.0, 0.0, 0.0, -inset, 0.0, 0.0, 5),
        ];
        for (nx, ny, nz, tx, ty, tz, face) in sides
        {
            tessellator.startDrawingQuads();
            tessellator.setNormal(nx, ny, nz);
            if inset != 0.0
            {
                tessellator.setTranslationF(tx, ty, tz);
            }
            match face
            {
                2 => self.renderEastFace(block, 0.0, 0.0, 0.0, sprite(face)),
                3 => self.renderWestFace(block, 0.0, 0.0, 0.0, sprite(face)),
                4 => self.renderNorthFace(block, 0.0, 0.0, 0.0, sprite(face)),
                _ => self.renderSouthFace(block, 0.0, 0.0, 0.0, sprite(face)),
            }
            if inset != 0.0
            {
                tessellator.setTranslationF(-tx, -ty, -tz);
            }
            tessellator.draw();
        }
    }

    pub fn renderBlockOnInventory(&mut self, block: &Rc<Block>, mut metadata: i32, brightness: f32)
    {
        let tessellator: Rc<Tessellator> = Tessellator::instance();
        if self.m_updateColor
        {
            let n: i32 = block.getRenderColor(metadata);
            let f2: f32 = (n >> 16 & 0xFF) as f32 / 255.0;
            let f: f32 = (n >> 8 & 0xFF) as f32 / 255.0;
            let f3: f32 = (n & 0xFF) as f32 / 255.0;
            GL11::glColor4f(f2 * brightness, f * brightness, f3 * brightness, 1.0);
        }
        let n: i32 = block.getRenderType();
        match n
        {
            0 | 16 =>
            {
                if n == 16
                {
                    metadata = 1;
                }
                block.setBlockBoundsForItemRender();
                GL11::glTranslatef(-0.5, -0.5, -0.5);
                self.renderInventoryCube(block, Some(metadata), 0.0);
                GL11::glTranslatef(0.5, 0.5, 0.5);
            }
            1 =>
            {
                tessellator.startDrawingQuads();
                tessellator.setNormal(0.0, -1.0, 0.0);
                self.renderCrossedSquares(block, metadata, -0.5, -0.5, -0.5);
                tessellator.draw();
            }
            13 =>
            {
                block.setBlockBoundsForItemRender();
                GL11::glTranslatef(-0.5, -0.5, -0.5);
                self.renderInventoryCube(block, None, 0.0625);
                GL11::glTranslatef(0.5, 0.5, 0.5);
            }
            6 =>
            {
                tessellator.startDrawingQuads();
                tessellator.setNormal(0.0, -1.0, 0.0);
                self.tesselatePlant(block, metadata, -0.5, -0.5, -0.5);
                tessellator.draw();
            }
            2 =>
            {
                tessellator.startDrawingQuads();
                tessellator.setNormal(0.0, -1.0, 0.0);
                self.renderTorchAtAngle(block, -0.5, -0.5, -0.5, 0.0, 0.0);
                tessellator.draw();
            }
            10 =>
            {
                for i in 0..2
                {
                    if i == 0
                    {
                        block.setBlockBounds(0.0, 0.0, 0.0, 1.0, 1.0, 0.5);
                    }
                    if i == 1
                    {
                        block.setBlockBounds(0.0, 0.0, 0.5, 1.0, 0.5, 1.0);
                    }
                    GL11::glTranslatef(-0.5, -0.5, -0.5);
                    self.renderInventoryCube(block, None, 0.0);
                    GL11::glTranslatef(0.5, 0.5, 0.5);
                }
            }
            11 =>
            {
                for i in 0..4
                {
                    let mut f: f32 = 0.125;
                    if i == 0
                    {
                        block.setBlockBounds(0.5 - f, 0.0, 0.0, 0.5 + f, 1.0, f * 2.0);
                    }
                    if i == 1
                    {
                        block.setBlockBounds(0.5 - f, 0.0, 1.0 - f * 2.0, 0.5 + f, 1.0, 1.0);
                    }
                    f = 0.0625;
                    if i == 2
                    {
                        block.setBlockBounds(0.5 - f, 1.0 - f * 3.0, -f * 2.0, 0.5 + f, 1.0 - f, 1.0 + f * 2.0);
                    }
                    if i == 3
                    {
                        block.setBlockBounds(0.5 - f, 0.5 - f * 3.0, -f * 2.0, 0.5 + f, 0.5 - f, 1.0 + f * 2.0);
                    }
                    GL11::glTranslatef(-0.5, -0.5, -0.5);
                    self.renderInventoryCube(block, None, 0.0);
                    GL11::glTranslatef(0.5, 0.5, 0.5);
                }
                block.setBlockBounds(0.0, 0.0, 0.0, 1.0, 1.0, 1.0);
            }
            _ =>
            {}
        }
    }
}
