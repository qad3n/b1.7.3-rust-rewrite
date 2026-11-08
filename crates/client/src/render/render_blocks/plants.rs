use super::{RenderBlocks, spriteOrigin};
use crate::render::{EntityRenderer, Tessellator};
use game::block::Block;
use game::world::IBlockAccess;
use std::rc::Rc;

impl RenderBlocks
{
    pub fn renderBlockReed(&mut self, block: &Rc<Block>, x: i32, y: i32, z: i32) -> bool
    {
        let world: Rc<dyn IBlockAccess> = self.world();
        let tessellator: Rc<Tessellator> = Tessellator::instance();
        let f: f32 = block.getBlockBrightness(&*world, x, y, z);
        let n: i32 = block.colorMultiplier(&*world, x, y, z);
        let mut f2: f32 = (n >> 16 & 0xFF) as f32 / 255.0;
        let mut f3: f32 = (n >> 8 & 0xFF) as f32 / 255.0;
        let mut f4: f32 = (n & 0xFF) as f32 / 255.0;
        if EntityRenderer::anaglyphEnabled()
        {
            let f5: f32 = (f2 * 30.0 + f3 * 59.0 + f4 * 11.0) / 100.0;
            let f6: f32 = (f2 * 30.0 + f3 * 70.0) / 100.0;
            let f7: f32 = (f2 * 30.0 + f4 * 70.0) / 100.0;
            f2 = f5;
            f3 = f6;
            f4 = f7;
        }
        tessellator.setColorOpaque_F(f * f2, f * f3, f * f4);
        let mut d: f64 = x as f64;
        let mut d2: f64 = y as f64;
        let mut d3: f64 = z as f64;
        if Rc::ptr_eq(block, &Block::tallGrass())
        {
            let mut l: i64 = x.wrapping_mul(3129871) as i64 ^ (z as i64).wrapping_mul(116129781) ^ y as i64;
            l = l.wrapping_mul(l).wrapping_mul(42317861).wrapping_add(l.wrapping_mul(11));
            d += (((l >> 16 & 0xF) as f32 / 15.0) as f64 - 0.5) * 0.5;
            d2 += (((l >> 20 & 0xF) as f32 / 15.0) as f64 - 1.0) * 0.2;
            d3 += (((l >> 24 & 0xF) as f32 / 15.0) as f64 - 0.5) * 0.5;
        }
        self.renderCrossedSquares(block, world.getBlockMetadata(x, y, z), d, d2, d3);
        true
    }

    pub fn renderBlockCrops(&mut self, block: &Rc<Block>, x: i32, y: i32, z: i32) -> bool
    {
        let world: Rc<dyn IBlockAccess> = self.world();
        let tessellator: Rc<Tessellator> = Tessellator::instance();
        let f: f32 = block.getBlockBrightness(&*world, x, y, z);
        tessellator.setColorOpaque_F(f, f, f);
        self.tesselatePlant(block, world.getBlockMetadata(x, y, z), x as f64, (y as f32 - 0.0625) as f64, z as f64);
        true
    }

    fn plantSprite(&self, block: &Rc<Block>, metadata: i32) -> (f64, f64, f64, f64)
    {
        let mut n: i32 = block.getBlockTextureFromSideAndMetadata(0, metadata);
        if self.m_overrideBlockTexture >= 0
        {
            n = self.m_overrideBlockTexture;
        }
        let (n2, n3): (i32, i32) = spriteOrigin(n);
        (
            (n2 as f32 / 256.0) as f64,
            ((n2 as f32 + 15.99) / 256.0) as f64,
            (n3 as f32 / 256.0) as f64,
            ((n3 as f32 + 15.99) / 256.0) as f64,
        )
    }

    pub fn renderCrossedSquares(&mut self, block: &Rc<Block>, metadata: i32, x: f64, y: f64, z: f64)
    {
        let tessellator: Rc<Tessellator> = Tessellator::instance();
        let (d, d2, d3, d4): (f64, f64, f64, f64) = self.plantSprite(block, metadata);
        let d5: f64 = x + 0.5 - 0.45f32 as f64;
        let d6: f64 = x + 0.5 + 0.45f32 as f64;
        let d7: f64 = z + 0.5 - 0.45f32 as f64;
        let d8: f64 = z + 0.5 + 0.45f32 as f64;
        let v = |x: f64, y: f64, z: f64, u: f64, w: f64| tessellator.addVertexWithUV(x, y, z, u, w);
        v(d5, y + 1.0, d7, d, d3);
        v(d5, y + 0.0, d7, d, d4);
        v(d6, y + 0.0, d8, d2, d4);
        v(d6, y + 1.0, d8, d2, d3);
        v(d6, y + 1.0, d8, d, d3);
        v(d6, y + 0.0, d8, d, d4);
        v(d5, y + 0.0, d7, d2, d4);
        v(d5, y + 1.0, d7, d2, d3);
        v(d5, y + 1.0, d8, d, d3);
        v(d5, y + 0.0, d8, d, d4);
        v(d6, y + 0.0, d7, d2, d4);
        v(d6, y + 1.0, d7, d2, d3);
        v(d6, y + 1.0, d7, d, d3);
        v(d6, y + 0.0, d7, d, d4);
        v(d5, y + 0.0, d8, d2, d4);
        v(d5, y + 1.0, d8, d2, d3);
    }

    pub fn tesselatePlant(&mut self, block: &Rc<Block>, metadata: i32, x: f64, y: f64, z: f64)
    {
        let tessellator: Rc<Tessellator> = Tessellator::instance();
        let (d, d2, d3, d4): (f64, f64, f64, f64) = self.plantSprite(block, metadata);
        let v = |x: f64, y: f64, z: f64, u: f64, w: f64| tessellator.addVertexWithUV(x, y, z, u, w);
        let mut d5: f64 = x + 0.5 - 0.25;
        let mut d6: f64 = x + 0.5 + 0.25;
        let mut d7: f64 = z + 0.5 - 0.5;
        let mut d8: f64 = z + 0.5 + 0.5;
        v(d5, y + 1.0, d7, d, d3);
        v(d5, y + 0.0, d7, d, d4);
        v(d5, y + 0.0, d8, d2, d4);
        v(d5, y + 1.0, d8, d2, d3);
        v(d5, y + 1.0, d8, d, d3);
        v(d5, y + 0.0, d8, d, d4);
        v(d5, y + 0.0, d7, d2, d4);
        v(d5, y + 1.0, d7, d2, d3);
        v(d6, y + 1.0, d8, d, d3);
        v(d6, y + 0.0, d8, d, d4);
        v(d6, y + 0.0, d7, d2, d4);
        v(d6, y + 1.0, d7, d2, d3);
        v(d6, y + 1.0, d7, d, d3);
        v(d6, y + 0.0, d7, d, d4);
        v(d6, y + 0.0, d8, d2, d4);
        v(d6, y + 1.0, d8, d2, d3);
        d5 = x + 0.5 - 0.5;
        d6 = x + 0.5 + 0.5;
        d7 = z + 0.5 - 0.25;
        d8 = z + 0.5 + 0.25;
        v(d5, y + 1.0, d7, d, d3);
        v(d5, y + 0.0, d7, d, d4);
        v(d6, y + 0.0, d7, d2, d4);
        v(d6, y + 1.0, d7, d2, d3);
        v(d6, y + 1.0, d7, d, d3);
        v(d6, y + 0.0, d7, d, d4);
        v(d5, y + 0.0, d7, d2, d4);
        v(d5, y + 1.0, d7, d2, d3);
        v(d6, y + 1.0, d8, d, d3);
        v(d6, y + 0.0, d8, d, d4);
        v(d5, y + 0.0, d8, d2, d4);
        v(d5, y + 1.0, d8, d2, d3);
        v(d5, y + 1.0, d8, d, d3);
        v(d5, y + 0.0, d8, d, d4);
        v(d6, y + 0.0, d8, d2, d4);
        v(d6, y + 1.0, d8, d2, d3);
    }
}
