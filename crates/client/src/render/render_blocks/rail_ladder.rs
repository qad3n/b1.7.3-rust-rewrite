use super::{RenderBlocks, spriteOrigin};
use crate::render::Tessellator;
use game::block::{Block, block_rail};
use game::world::IBlockAccess;
use std::rc::Rc;

impl RenderBlocks
{
    pub fn renderBlockMinecartTrack(&mut self, block: &Rc<Block>, x: i32, y: i32, z: i32) -> bool
    {
        let world: Rc<dyn IBlockAccess> = self.world();
        let tessellator: Rc<Tessellator> = Tessellator::instance();
        let mut n: i32 = world.getBlockMetadata(x, y, z);
        let mut n2: i32 = block.getBlockTextureFromSideAndMetadata(0, n);
        if self.m_overrideBlockTexture >= 0
        {
            n2 = self.m_overrideBlockTexture;
        }
        if block_rail::getIsPowered(block)
        {
            n &= 7;
        }
        let f: f32 = block.getBlockBrightness(&*world, x, y, z);
        tessellator.setColorOpaque_F(f, f, f);
        let (n3, n4): (i32, i32) = spriteOrigin(n2);
        let d: f64 = (n3 as f32 / 256.0) as f64;
        let d2: f64 = ((n3 as f32 + 15.99) / 256.0) as f64;
        let d3: f64 = (n4 as f32 / 256.0) as f64;
        let d4: f64 = ((n4 as f32 + 15.99) / 256.0) as f64;
        let f2: f32 = 0.0625;
        let (x0, x1, z0, z1): (f32, f32, f32, f32) = (x as f32, (x + 1) as f32, z as f32, (z + 1) as f32);
        let (mut f3, mut f4, mut f5, mut f6): (f32, f32, f32, f32) = (x1, x1, x0, x0);
        let (mut f7, mut f8, mut f9, mut f10): (f32, f32, f32, f32) = (z0, z1, z1, z0);
        let mut f11: f32 = y as f32 + f2;
        let mut f12: f32 = y as f32 + f2;
        let mut f13: f32 = y as f32 + f2;
        let mut f14: f32 = y as f32 + f2;
        if n == 1 || n == 2 || n == 3 || n == 7
        {
            (f3, f6, f4, f5) = (x1, x1, x0, x0);
            (f7, f8, f9, f10) = (z1, z1, z0, z0);
        }
        else if n == 8
        {
            (f3, f4, f5, f6) = (x0, x0, x1, x1);
            (f7, f10, f8, f9) = (z1, z1, z0, z0);
        }
        else if n == 9
        {
            (f3, f6, f4, f5) = (x0, x0, x1, x1);
            (f7, f8, f9, f10) = (z0, z0, z1, z1);
        }
        if n == 2 || n == 4
        {
            f11 += 1.0;
            f14 += 1.0;
        }
        else if n == 3 || n == 5
        {
            f12 += 1.0;
            f13 += 1.0;
        }
        let v =
            |x: f32, y: f32, z: f32, u: f64, w: f64| tessellator.addVertexWithUV(x as f64, y as f64, z as f64, u, w);
        v(f3, f11, f7, d2, d3);
        v(f4, f12, f8, d2, d4);
        v(f5, f13, f9, d, d4);
        v(f6, f14, f10, d, d3);
        v(f6, f14, f10, d, d3);
        v(f5, f13, f9, d, d4);
        v(f4, f12, f8, d2, d4);
        v(f3, f11, f7, d2, d3);
        true
    }

    pub fn renderBlockLadder(&mut self, block: &Rc<Block>, x: i32, y: i32, z: i32) -> bool
    {
        let world: Rc<dyn IBlockAccess> = self.world();
        let tessellator: Rc<Tessellator> = Tessellator::instance();
        let mut n: i32 = block.getBlockTextureFromSide(0);
        if self.m_overrideBlockTexture >= 0
        {
            n = self.m_overrideBlockTexture;
        }
        let f: f32 = block.getBlockBrightness(&*world, x, y, z);
        tessellator.setColorOpaque_F(f, f, f);
        let (n2, n3): (i32, i32) = spriteOrigin(n);
        let d: f64 = (n2 as f32 / 256.0) as f64;
        let d2: f64 = ((n2 as f32 + 15.99) / 256.0) as f64;
        let d3: f64 = (n3 as f32 / 256.0) as f64;
        let d4: f64 = ((n3 as f32 + 15.99) / 256.0) as f64;
        let n4: i32 = world.getBlockMetadata(x, y, z);
        let f2: f32 = 0.0;
        let f3: f32 = 0.05;
        let x0: f32 = x as f32 - f2;
        let x1: f32 = (x + 1) as f32 + f2;
        let y0: f32 = y as f32 - f2;
        let y1: f32 = (y + 1) as f32 + f2;
        let z0: f32 = z as f32 - f2;
        let z1: f32 = (z + 1) as f32 + f2;
        let v =
            |x: f32, y: f32, z: f32, u: f64, w: f64| tessellator.addVertexWithUV(x as f64, y as f64, z as f64, u, w);
        if n4 == 5
        {
            let side: f32 = x as f32 + f3;
            v(side, y1, z1, d, d3);
            v(side, y0, z1, d, d4);
            v(side, y0, z0, d2, d4);
            v(side, y1, z0, d2, d3);
        }
        if n4 == 4
        {
            let side: f32 = (x + 1) as f32 - f3;
            v(side, y0, z1, d2, d4);
            v(side, y1, z1, d2, d3);
            v(side, y1, z0, d, d3);
            v(side, y0, z0, d, d4);
        }
        if n4 == 3
        {
            let side: f32 = z as f32 + f3;
            v(x1, y0, side, d2, d4);
            v(x1, y1, side, d2, d3);
            v(x0, y1, side, d, d3);
            v(x0, y0, side, d, d4);
        }
        if n4 == 2
        {
            let side: f32 = (z + 1) as f32 - f3;
            v(x1, y1, side, d, d3);
            v(x1, y0, side, d, d4);
            v(x0, y0, side, d2, d4);
            v(x0, y1, side, d2, d3);
        }
        true
    }
}
