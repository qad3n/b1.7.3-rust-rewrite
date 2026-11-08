use super::{RenderBlocks, spriteOrigin};
use crate::render::Tessellator;
use game::block::{Block, Material, block_fluid};
use game::math::math_helper::{cos, sin};
use game::world::{IBlockAccess, World};
use std::rc::Rc;

impl RenderBlocks
{
    pub fn renderBlockFluids(&mut self, block: &Rc<Block>, x: i32, y: i32, z: i32) -> bool
    {
        let world: Rc<dyn IBlockAccess> = self.world();
        let tessellator: Rc<Tessellator> = Tessellator::instance();
        let n3: i32 = block.colorMultiplier(&*world, x, y, z);
        let f4: f32 = (n3 >> 16 & 0xFF) as f32 / 255.0;
        let f5: f32 = (n3 >> 8 & 0xFF) as f32 / 255.0;
        let f6: f32 = (n3 & 0xFF) as f32 / 255.0;
        let bl: bool = block.shouldSideBeRendered(&*world, x, y + 1, z, 1);
        let bl2: bool = block.shouldSideBeRendered(&*world, x, y - 1, z, 0);
        let blArray: [bool; 4] = [
            block.shouldSideBeRendered(&*world, x, y, z - 1, 2),
            block.shouldSideBeRendered(&*world, x, y, z + 1, 3),
            block.shouldSideBeRendered(&*world, x - 1, y, z, 4),
            block.shouldSideBeRendered(&*world, x + 1, y, z, 5),
        ];
        if !(bl || bl2 || blArray[0] || blArray[1] || blArray[2] || blArray[3])
        {
            return false;
        }
        let mut bl3: bool = false;
        let f7: f32 = 0.5;
        let f8: f32 = 1.0;
        let f9: f32 = 0.8;
        let f10: f32 = 0.6;
        let d: f64 = 0.0;
        let d2: f64 = 1.0;
        let material: Rc<Material> = Rc::clone(&block.m_blockMaterial);
        let n4: i32 = world.getBlockMetadata(x, y, z);
        let f11: f32 = self.getLiquidHeight(x, y, z, &material);
        let f12: f32 = self.getLiquidHeight(x, y, z + 1, &material);
        let f13: f32 = self.getLiquidHeight(x + 1, y, z + 1, &material);
        let f14: f32 = self.getLiquidHeight(x + 1, y, z, &material);
        if self.m_renderAllFaces || bl
        {
            bl3 = true;
            let mut n5: i32 = block.getBlockTextureFromSideAndMetadata(1, n4);
            let mut f15: f32 = block_fluid::getFlowAngle(&*world, x, y, z, &material) as f32;
            if f15 > -999.0
            {
                n5 = block.getBlockTextureFromSideAndMetadata(2, n4);
            }
            let (n2, n): (i32, i32) = spriteOrigin(n5);
            let mut d3: f64 = (n2 as f64 + 8.0) / 256.0;
            let mut d4: f64 = (n as f64 + 8.0) / 256.0;
            if f15 < -999.0
            {
                f15 = 0.0;
            }
            else
            {
                d3 = ((n2 + 16) as f32 / 256.0) as f64;
                d4 = ((n + 16) as f32 / 256.0) as f64;
            }
            let f3: f64 = (sin(f15) * 8.0 / 256.0) as f64;
            let f2: f64 = (cos(f15) * 8.0 / 256.0) as f64;
            let f: f32 = block.getBlockBrightness(&*world, x, y, z);
            tessellator.setColorOpaque_F(f8 * f * f4, f8 * f * f5, f8 * f * f6);
            let top = |height: f32| (y as f32 + height) as f64;
            tessellator.addVertexWithUV(x as f64, top(f11), z as f64, d3 - f2 - f3, d4 - f2 + f3);
            tessellator.addVertexWithUV(x as f64, top(f12), (z + 1) as f64, d3 - f2 + f3, d4 + f2 + f3);
            tessellator.addVertexWithUV((x + 1) as f64, top(f13), (z + 1) as f64, d3 + f2 + f3, d4 + f2 - f3);
            tessellator.addVertexWithUV((x + 1) as f64, top(f14), z as f64, d3 + f2 - f3, d4 - f2 - f3);
        }
        if self.m_renderAllFaces || bl2
        {
            let f16: f32 = block.getBlockBrightness(&*world, x, y - 1, z);
            tessellator.setColorOpaque_F(f7 * f16, f7 * f16, f7 * f16);
            self.renderBottomFace(block, x as f64, y as f64, z as f64, block.getBlockTextureFromSide(0));
            bl3 = true;
        }
        for i in 0..4
        {
            let mut n6: i32 = x;
            let n2: i32 = y;
            let mut n: i32 = z;
            match i
            {
                0 => n -= 1,
                1 => n += 1,
                2 => n6 -= 1,
                _ => n6 += 1,
            }
            let n7: i32 = block.getBlockTextureFromSideAndMetadata(i + 2, n4);
            let (n8, n9): (i32, i32) = spriteOrigin(n7);
            if !self.m_renderAllFaces && !blArray[i as usize]
            {
                continue;
            }
            let (f19, f3, f2, f18, f, f17): (f32, f32, i32, i32, i32, i32) = match i
            {
                0 => (f11, f14, x, x + 1, z, z),
                1 => (f13, f12, x + 1, x, z + 1, z + 1),
                2 => (f12, f11, x, x, z + 1, z),
                _ => (f14, f13, x + 1, x + 1, z, z + 1),
            };
            bl3 = true;
            let d5: f64 = (n8 as f32 / 256.0) as f64;
            let d6: f64 = ((n8 + 16) as f64 - 0.01) / 256.0;
            let d7: f64 = ((n9 as f32 + (1.0 - f19) * 16.0) / 256.0) as f64;
            let d8: f64 = ((n9 as f32 + (1.0 - f3) * 16.0) / 256.0) as f64;
            let d9: f64 = ((n9 + 16) as f64 - 0.01) / 256.0;
            let mut f20: f32 = block.getBlockBrightness(&*world, n6, n2, n);
            f20 *= if i < 2 { f9 } else { f10 };
            tessellator.setColorOpaque_F(f8 * f20 * f4, f8 * f20 * f5, f8 * f20 * f6);
            tessellator.addVertexWithUV(f2 as f32 as f64, (y as f32 + f19) as f64, f as f32 as f64, d5, d7);
            tessellator.addVertexWithUV(f18 as f32 as f64, (y as f32 + f3) as f64, f17 as f32 as f64, d6, d8);
            tessellator.addVertexWithUV(f18 as f32 as f64, y as f64, f17 as f32 as f64, d6, d9);
            tessellator.addVertexWithUV(f2 as f32 as f64, y as f64, f as f32 as f64, d5, d9);
        }
        block.m_minY.set(d);
        block.m_maxY.set(d2);
        bl3
    }

    fn getLiquidHeight(&self, x: i32, y: i32, z: i32, liquid: &Rc<Material>) -> f32
    {
        let world: Rc<dyn IBlockAccess> = self.world();
        let mut i: i32 = 0;
        let mut f: f32 = 0.0;
        for j in 0..4
        {
            let k: i32 = x - (j & 1);
            let l: i32 = y;
            let i1: i32 = z - (j >> 1 & 1);
            if Rc::ptr_eq(&world.getBlockMaterial(k, l + 1, i1), liquid)
            {
                return 1.0;
            }
            let material: Rc<Material> = world.getBlockMaterial(k, l, i1);
            if Rc::ptr_eq(&material, liquid)
            {
                let j1: i32 = world.getBlockMetadata(k, l, i1);
                if j1 >= 8 || j1 == 0
                {
                    f += block_fluid::getPercentAir(j1) * 10.0;
                    i += 10;
                }
                f += block_fluid::getPercentAir(j1);
                i += 1;
            }
            else if !material.isSolid()
            {
                f += 1.0;
                i += 1;
            }
        }
        1.0 - f / i as f32
    }

    pub fn renderBlockFallingSand(&mut self, block: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32)
    {
        let f: f32 = 0.5;
        let f2: f32 = 1.0;
        let f3: f32 = 0.8;
        let f4: f32 = 0.6;
        let tessellator: Rc<Tessellator> = Tessellator::instance();
        tessellator.startDrawingQuads();
        let world: &dyn IBlockAccess = &**world;
        let f5: f32 = block.getBlockBrightness(world, x, y, z);
        let lit = |x: i32, y: i32, z: i32| {
            let f6: f32 = block.getBlockBrightness(world, x, y, z);
            if f6 < f5 { f5 } else { f6 }
        };
        let mut f6: f32 = lit(x, y - 1, z);
        tessellator.setColorOpaque_F(f * f6, f * f6, f * f6);
        self.renderBottomFace(block, -0.5, -0.5, -0.5, block.getBlockTextureFromSide(0));
        f6 = lit(x, y + 1, z);
        tessellator.setColorOpaque_F(f2 * f6, f2 * f6, f2 * f6);
        self.renderTopFace(block, -0.5, -0.5, -0.5, block.getBlockTextureFromSide(1));
        f6 = lit(x, y, z - 1);
        tessellator.setColorOpaque_F(f3 * f6, f3 * f6, f3 * f6);
        self.renderEastFace(block, -0.5, -0.5, -0.5, block.getBlockTextureFromSide(2));
        f6 = lit(x, y, z + 1);
        tessellator.setColorOpaque_F(f3 * f6, f3 * f6, f3 * f6);
        self.renderWestFace(block, -0.5, -0.5, -0.5, block.getBlockTextureFromSide(3));
        f6 = lit(x - 1, y, z);
        tessellator.setColorOpaque_F(f4 * f6, f4 * f6, f4 * f6);
        self.renderNorthFace(block, -0.5, -0.5, -0.5, block.getBlockTextureFromSide(4));
        f6 = lit(x + 1, y, z);
        tessellator.setColorOpaque_F(f4 * f6, f4 * f6, f4 * f6);
        self.renderSouthFace(block, -0.5, -0.5, -0.5, block.getBlockTextureFromSide(5));
        tessellator.draw();
    }
}
