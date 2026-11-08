use super::{RenderBlocks, spriteOrigin};
use crate::render::Tessellator;
use game::block::{Block, block_bed, block_redstone_repeater, model_bed};
use game::world::IBlockAccess;
use std::rc::Rc;

impl RenderBlocks
{
    pub(super) fn renderBlockBed(&mut self, block: &Rc<Block>, x: i32, y: i32, z: i32) -> bool
    {
        let world: Rc<dyn IBlockAccess> = self.world();
        let tessellator: Rc<Tessellator> = Tessellator::instance();
        let n: i32 = world.getBlockMetadata(x, y, z);
        let n2: i32 = block_bed::getDirectionFromMetadata(n);
        let bl: bool = block_bed::isBlockFootOfBed(n);
        let f: f32 = 0.5;
        let f2: f32 = 1.0;
        let f3: f32 = 0.8;
        let f4: f32 = 0.6;
        let (f5, f6, f7): (f32, f32, f32) = (f2, f2, f2);
        let (f8, f9, f10): (f32, f32, f32) = (f, f3, f4);
        let (f11, f12, f13): (f32, f32, f32) = (f, f3, f4);
        let (f14, f15, f16): (f32, f32, f32) = (f, f3, f4);
        let f17: f32 = block.getBlockBrightness(&*world, x, y, z);
        tessellator.setColorOpaque_F(f8 * f17, f11 * f17, f14 * f17);
        let (n4, n5): (i32, i32) = spriteOrigin(block.getBlockTexture(&*world, x, y, z, 0));
        let d: f64 = (n4 as f32 / 256.0) as f64;
        let d2: f64 = ((n4 + 16) as f64 - 0.01) / 256.0;
        let d3: f64 = (n5 as f32 / 256.0) as f64;
        let d4: f64 = ((n5 + 16) as f64 - 0.01) / 256.0;
        let d5: f64 = x as f64 + block.m_minX.get();
        let d6: f64 = x as f64 + block.m_maxX.get();
        let d7: f64 = y as f64 + block.m_minY.get() + 0.1875;
        let d8: f64 = z as f64 + block.m_minZ.get();
        let d9: f64 = z as f64 + block.m_maxZ.get();
        tessellator.addVertexWithUV(d5, d7, d9, d, d4);
        tessellator.addVertexWithUV(d5, d7, d8, d, d3);
        tessellator.addVertexWithUV(d6, d7, d8, d2, d3);
        tessellator.addVertexWithUV(d6, d7, d9, d2, d4);
        let f18: f32 = block.getBlockBrightness(&*world, x, y + 1, z);
        tessellator.setColorOpaque_F(f5 * f18, f6 * f18, f7 * f18);
        let (n5, n6): (i32, i32) = spriteOrigin(block.getBlockTexture(&*world, x, y, z, 1));
        let d10: f64 = (n5 as f32 / 256.0) as f64;
        let d11: f64 = ((n5 + 16) as f64 - 0.01) / 256.0;
        let d12: f64 = (n6 as f32 / 256.0) as f64;
        let d13: f64 = ((n6 + 16) as f64 - 0.01) / 256.0;
        let (mut d14, mut d15, mut d16, mut d17): (f64, f64, f64, f64) = (d10, d11, d12, d12);
        let (mut d18, mut d19, mut d20, mut d21): (f64, f64, f64, f64) = (d10, d11, d13, d13);
        if n2 == 0
        {
            d15 = d10;
            d16 = d13;
            d18 = d11;
            d21 = d12;
        }
        else if n2 == 2
        {
            d14 = d11;
            d17 = d13;
            d19 = d10;
            d20 = d12;
        }
        else if n2 == 3
        {
            d14 = d11;
            d17 = d13;
            d19 = d10;
            d20 = d12;
            d15 = d10;
            d16 = d13;
            d18 = d11;
            d21 = d12;
        }
        let d22: f64 = x as f64 + block.m_minX.get();
        let d23: f64 = x as f64 + block.m_maxX.get();
        let d24: f64 = y as f64 + block.m_maxY.get();
        let d25: f64 = z as f64 + block.m_minZ.get();
        let d26: f64 = z as f64 + block.m_maxZ.get();
        tessellator.addVertexWithUV(d23, d24, d26, d18, d20);
        tessellator.addVertexWithUV(d23, d24, d25, d14, d16);
        tessellator.addVertexWithUV(d22, d24, d25, d15, d17);
        tessellator.addVertexWithUV(d22, d24, d26, d19, d21);
        let mut n7: i32 = model_bed::TO_DIR[n2 as usize];
        if bl
        {
            n7 = model_bed::TO_DIR[model_bed::OPPOSITE[n2 as usize] as usize];
        }
        let n4: i32 = match n2
        {
            0 => 5,
            3 => 2,
            1 => 3,
            _ => 4,
        };
        if n7 != 2 && (self.m_renderAllFaces || block.shouldSideBeRendered(&*world, x, y, z - 1, 2))
        {
            let mut f19: f32 = block.getBlockBrightness(&*world, x, y, z - 1);
            if block.m_minZ.get() > 0.0
            {
                f19 = f17;
            }
            tessellator.setColorOpaque_F(f9 * f19, f12 * f19, f15 * f19);
            self.m_flipTexture = n4 == 2;
            self.renderEastFace(block, x as f64, y as f64, z as f64, block.getBlockTexture(&*world, x, y, z, 2));
        }
        if n7 != 3 && (self.m_renderAllFaces || block.shouldSideBeRendered(&*world, x, y, z + 1, 3))
        {
            let mut f20: f32 = block.getBlockBrightness(&*world, x, y, z + 1);
            if block.m_maxZ.get() < 1.0
            {
                f20 = f17;
            }
            tessellator.setColorOpaque_F(f9 * f20, f12 * f20, f15 * f20);
            self.m_flipTexture = n4 == 3;
            self.renderWestFace(block, x as f64, y as f64, z as f64, block.getBlockTexture(&*world, x, y, z, 3));
        }
        if n7 != 4 && (self.m_renderAllFaces || block.shouldSideBeRendered(&*world, x - 1, y, z, 4))
        {
            let mut f21: f32 = block.getBlockBrightness(&*world, x - 1, y, z);
            if block.m_minX.get() > 0.0
            {
                f21 = f17;
            }
            tessellator.setColorOpaque_F(f10 * f21, f13 * f21, f16 * f21);
            self.m_flipTexture = n4 == 4;
            self.renderNorthFace(block, x as f64, y as f64, z as f64, block.getBlockTexture(&*world, x, y, z, 4));
        }
        if n7 != 5 && (self.m_renderAllFaces || block.shouldSideBeRendered(&*world, x + 1, y, z, 5))
        {
            let mut f22: f32 = block.getBlockBrightness(&*world, x + 1, y, z);
            if block.m_maxX.get() < 1.0
            {
                f22 = f17;
            }
            tessellator.setColorOpaque_F(f10 * f22, f13 * f22, f16 * f22);
            self.m_flipTexture = n4 == 5;
            self.renderSouthFace(block, x as f64, y as f64, z as f64, block.getBlockTexture(&*world, x, y, z, 5));
        }
        self.m_flipTexture = false;
        true
    }

    pub fn renderBlockTorch(&mut self, block: &Rc<Block>, x: i32, y: i32, z: i32) -> bool
    {
        let world: Rc<dyn IBlockAccess> = self.world();
        let n: i32 = world.getBlockMetadata(x, y, z);
        let tessellator: Rc<Tessellator> = Tessellator::instance();
        let mut f: f32 = block.getBlockBrightness(&*world, x, y, z);
        if Block::lightValue(block.m_blockID) > 0
        {
            f = 1.0;
        }
        tessellator.setColorOpaque_F(f, f, f);
        let d: f64 = 0.4f32 as f64;
        let d2: f64 = 0.5 - d;
        let d3: f64 = 0.2f32 as f64;
        let (x, y, z): (f64, f64, f64) = (x as f64, y as f64, z as f64);
        match n
        {
            1 => self.renderTorchAtAngle(block, x - d2, y + d3, z, -d, 0.0),
            2 => self.renderTorchAtAngle(block, x + d2, y + d3, z, d, 0.0),
            3 => self.renderTorchAtAngle(block, x, y + d3, z - d2, 0.0, -d),
            4 => self.renderTorchAtAngle(block, x, y + d3, z + d2, 0.0, d),
            _ => self.renderTorchAtAngle(block, x, y, z, 0.0, 0.0),
        }
        true
    }

    pub(super) fn renderBlockRepeater(&mut self, block: &Rc<Block>, x: i32, y: i32, z: i32) -> bool
    {
        let world: Rc<dyn IBlockAccess> = self.world();
        let n: i32 = world.getBlockMetadata(x, y, z);
        let n2: i32 = n & 3;
        let n3: usize = ((n & 0xC) >> 2) as usize;
        self.renderStandardBlock(block, x, y, z);
        let tessellator: Rc<Tessellator> = Tessellator::instance();
        let mut f: f32 = block.getBlockBrightness(&*world, x, y, z);
        if Block::lightValue(block.m_blockID) > 0
        {
            f = (f + 1.0) * 0.5;
        }
        tessellator.setColorOpaque_F(f, f, f);
        let d: f64 = -0.1875;
        let mut d2: f64 = 0.0;
        let mut d3: f64 = 0.0;
        let mut d4: f64 = 0.0;
        let mut d5: f64 = 0.0;
        match n2
        {
            0 =>
            {
                d5 = -0.3125;
                d3 = block_redstone_repeater::RANDOM_DISPLAY[n3];
            }
            2 =>
            {
                d5 = 0.3125;
                d3 = -block_redstone_repeater::RANDOM_DISPLAY[n3];
            }
            3 =>
            {
                d4 = -0.3125;
                d2 = block_redstone_repeater::RANDOM_DISPLAY[n3];
            }
            1 =>
            {
                d4 = 0.3125;
                d2 = -block_redstone_repeater::RANDOM_DISPLAY[n3];
            }
            _ =>
            {}
        }
        self.renderTorchAtAngle(block, x as f64 + d2, y as f64 + d, z as f64 + d3, 0.0, 0.0);
        self.renderTorchAtAngle(block, x as f64 + d4, y as f64 + d, z as f64 + d5, 0.0, 0.0);
        let (n5, n6): (i32, i32) = spriteOrigin(block.getBlockTextureFromSide(1));
        let d6: f64 = (n5 as f32 / 256.0) as f64;
        let d7: f64 = ((n5 as f32 + 15.99) / 256.0) as f64;
        let d8: f64 = (n6 as f32 / 256.0) as f64;
        let d9: f64 = ((n6 as f32 + 15.99) / 256.0) as f64;
        let f2: f32 = 0.125;
        let mut f3: f32 = (x + 1) as f32;
        let mut f4: f32 = (x + 1) as f32;
        let mut f5: f32 = x as f32;
        let mut f6: f32 = x as f32;
        let mut f7: f32 = z as f32;
        let mut f8: f32 = (z + 1) as f32;
        let mut f9: f32 = (z + 1) as f32;
        let mut f10: f32 = z as f32;
        let f11: f32 = y as f32 + f2;
        if n2 == 2
        {
            f3 = x as f32;
            f4 = f3;
            f5 = (x + 1) as f32;
            f6 = f5;
            f7 = (z + 1) as f32;
            f10 = f7;
            f8 = z as f32;
            f9 = f8;
        }
        else if n2 == 3
        {
            f3 = x as f32;
            f6 = f3;
            f4 = (x + 1) as f32;
            f5 = f4;
            f7 = z as f32;
            f8 = f7;
            f9 = (z + 1) as f32;
            f10 = f9;
        }
        else if n2 == 1
        {
            f3 = (x + 1) as f32;
            f6 = f3;
            f4 = x as f32;
            f5 = f4;
            f7 = (z + 1) as f32;
            f8 = f7;
            f9 = z as f32;
            f10 = f9;
        }
        tessellator.addVertexWithUV(f6 as f64, f11 as f64, f10 as f64, d6, d8);
        tessellator.addVertexWithUV(f5 as f64, f11 as f64, f9 as f64, d6, d9);
        tessellator.addVertexWithUV(f4 as f64, f11 as f64, f8 as f64, d7, d9);
        tessellator.addVertexWithUV(f3 as f64, f11 as f64, f7 as f64, d7, d8);
        true
    }

    pub fn renderTorchAtAngle(&mut self, block: &Rc<Block>, mut x: f64, y: f64, mut z: f64, dx: f64, dz: f64)
    {
        let tessellator: Rc<Tessellator> = Tessellator::instance();
        let mut i: i32 = block.getBlockTextureFromSide(0);
        if self.m_overrideBlockTexture >= 0
        {
            i = self.m_overrideBlockTexture;
        }
        let (j, k): (i32, i32) = spriteOrigin(i);
        let f: f64 = (j as f32 / 256.0) as f64;
        let f1: f64 = ((j as f32 + 15.99) / 256.0) as f64;
        let f2: f64 = (k as f32 / 256.0) as f64;
        let f3: f64 = ((k as f32 + 15.99) / 256.0) as f64;
        let d0: f64 = f + 0.02734375;
        let d1: f64 = f2 + 0.0234375;
        let d2: f64 = f + 0.03515625;
        let d3: f64 = f2 + 0.03125;
        x += 0.5;
        z += 0.5;
        let d4: f64 = x - 0.5;
        let d5: f64 = x + 0.5;
        let d6: f64 = z - 0.5;
        let d7: f64 = z + 0.5;
        let d8: f64 = 0.0625;
        let d9: f64 = 0.625;
        let v = |x, y, z, u, w| tessellator.addVertexWithUV(x, y, z, u, w);
        v(x + dx * (1.0 - d9) - d8, y + d9, z + dz * (1.0 - d9) - d8, d0, d1);
        v(x + dx * (1.0 - d9) - d8, y + d9, z + dz * (1.0 - d9) + d8, d0, d3);
        v(x + dx * (1.0 - d9) + d8, y + d9, z + dz * (1.0 - d9) + d8, d2, d3);
        v(x + dx * (1.0 - d9) + d8, y + d9, z + dz * (1.0 - d9) - d8, d2, d1);
        v(x - d8, y + 1.0, d6, f, f2);
        v(x - d8 + dx, y + 0.0, d6 + dz, f, f3);
        v(x - d8 + dx, y + 0.0, d7 + dz, f1, f3);
        v(x - d8, y + 1.0, d7, f1, f2);
        v(x + d8, y + 1.0, d7, f, f2);
        v(x + dx + d8, y + 0.0, d7 + dz, f, f3);
        v(x + dx + d8, y + 0.0, d6 + dz, f1, f3);
        v(x + d8, y + 1.0, d6, f1, f2);
        v(d4, y + 1.0, z + d8, f, f2);
        v(d4 + dx, y + 0.0, z + d8 + dz, f, f3);
        v(d5 + dx, y + 0.0, z + d8 + dz, f1, f3);
        v(d5, y + 1.0, z + d8, f1, f2);
        v(d5, y + 1.0, z - d8, f, f2);
        v(d5 + dx, y + 0.0, z - d8 + dz, f, f3);
        v(d4 + dx, y + 0.0, z - d8 + dz, f1, f3);
        v(d4, y + 1.0, z - d8, f1, f2);
    }
}
