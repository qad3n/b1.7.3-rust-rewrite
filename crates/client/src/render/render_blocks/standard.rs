use super::RenderBlocks;
use crate::client::Minecraft;
use crate::render::{EntityRenderer, Tessellator};
use game::block::Block;
use game::world::IBlockAccess;
use std::rc::Rc;

fn colorChannels(color: i32) -> (f32, f32, f32)
{
    let mut f: f32 = (color >> 16 & 0xFF) as f32 / 255.0;
    let mut f2: f32 = (color >> 8 & 0xFF) as f32 / 255.0;
    let mut f3: f32 = (color & 0xFF) as f32 / 255.0;
    if EntityRenderer::anaglyphEnabled()
    {
        let f4: f32 = (f * 30.0 + f2 * 59.0 + f3 * 11.0) / 100.0;
        let f5: f32 = (f * 30.0 + f2 * 70.0) / 100.0;
        let f6: f32 = (f * 30.0 + f3 * 70.0) / 100.0;
        f = f4;
        f2 = f5;
        f3 = f6;
    }
    (f, f2, f3)
}

impl RenderBlocks
{
    pub fn renderStandardBlock(&mut self, block: &Rc<Block>, x: i32, y: i32, z: i32) -> bool
    {
        let (f, f2, f3): (f32, f32, f32) = colorChannels(block.colorMultiplier(&*self.world(), x, y, z));
        if Minecraft::isAmbientOcclusionEnabled()
        {
            return self.renderStandardBlockWithAmbientOcclusion(block, x, y, z, f, f2, f3);
        }
        self.renderStandardBlockWithColorMultiplier(block, x, y, z, f, f2, f3)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn renderStandardBlockWithColorMultiplier(
        &mut self,
        block: &Rc<Block>,
        x: i32,
        y: i32,
        z: i32,
        r: f32,
        g: f32,
        b: f32,
    ) -> bool
    {
        let world: Rc<dyn IBlockAccess> = self.world();
        self.m_enableAO = false;
        let tessellator: Rc<Tessellator> = Tessellator::instance();
        let mut bl: bool = false;
        let f2: f32 = 0.5;
        let f3: f32 = 1.0;
        let f4: f32 = 0.8;
        let f5: f32 = 0.6;
        let f6: f32 = f3 * r;
        let f7: f32 = f3 * g;
        let f8: f32 = f3 * b;
        let (mut f9, mut f10, mut f11): (f32, f32, f32) = (f2, f4, f5);
        let (mut f12, mut f13, mut f14): (f32, f32, f32) = (f2, f4, f5);
        let (mut f15, mut f16, mut f17): (f32, f32, f32) = (f2, f4, f5);
        if !Rc::ptr_eq(block, &Block::grass())
        {
            f9 *= r;
            f10 *= r;
            f11 *= r;
            f12 *= g;
            f13 *= g;
            f14 *= g;
            f15 *= b;
            f16 *= b;
            f17 *= b;
        }
        let f18: f32 = block.getBlockBrightness(&*world, x, y, z);
        let (dx, dy, dz): (f64, f64, f64) = (x as f64, y as f64, z as f64);
        if self.m_renderAllFaces || block.shouldSideBeRendered(&*world, x, y - 1, z, 0)
        {
            let f: f32 = block.getBlockBrightness(&*world, x, y - 1, z);
            tessellator.setColorOpaque_F(f9 * f, f12 * f, f15 * f);
            self.renderBottomFace(block, dx, dy, dz, block.getBlockTexture(&*world, x, y, z, 0));
            bl = true;
        }
        if self.m_renderAllFaces || block.shouldSideBeRendered(&*world, x, y + 1, z, 1)
        {
            let mut f: f32 = block.getBlockBrightness(&*world, x, y + 1, z);
            if block.m_maxY.get() != 1.0 && !block.m_blockMaterial.getIsLiquid()
            {
                f = f18;
            }
            tessellator.setColorOpaque_F(f6 * f, f7 * f, f8 * f);
            self.renderTopFace(block, dx, dy, dz, block.getBlockTexture(&*world, x, y, z, 1));
            bl = true;
        }
        if self.m_renderAllFaces || block.shouldSideBeRendered(&*world, x, y, z - 1, 2)
        {
            let mut f: f32 = block.getBlockBrightness(&*world, x, y, z - 1);
            if block.m_minZ.get() > 0.0
            {
                f = f18;
            }
            tessellator.setColorOpaque_F(f10 * f, f13 * f, f16 * f);
            let n: i32 = block.getBlockTexture(&*world, x, y, z, 2);
            self.renderEastFace(block, dx, dy, dz, n);
            if RenderBlocks::fancyGrass() && n == 3 && self.m_overrideBlockTexture < 0
            {
                tessellator.setColorOpaque_F(f10 * f * r, f13 * f * g, f16 * f * b);
                self.renderEastFace(block, dx, dy, dz, 38);
            }
            bl = true;
        }
        if self.m_renderAllFaces || block.shouldSideBeRendered(&*world, x, y, z + 1, 3)
        {
            let mut f: f32 = block.getBlockBrightness(&*world, x, y, z + 1);
            if block.m_maxZ.get() < 1.0
            {
                f = f18;
            }
            tessellator.setColorOpaque_F(f10 * f, f13 * f, f16 * f);
            let n: i32 = block.getBlockTexture(&*world, x, y, z, 3);
            self.renderWestFace(block, dx, dy, dz, n);
            if RenderBlocks::fancyGrass() && n == 3 && self.m_overrideBlockTexture < 0
            {
                tessellator.setColorOpaque_F(f10 * f * r, f13 * f * g, f16 * f * b);
                self.renderWestFace(block, dx, dy, dz, 38);
            }
            bl = true;
        }
        if self.m_renderAllFaces || block.shouldSideBeRendered(&*world, x - 1, y, z, 4)
        {
            let mut f: f32 = block.getBlockBrightness(&*world, x - 1, y, z);
            if block.m_minX.get() > 0.0
            {
                f = f18;
            }
            tessellator.setColorOpaque_F(f11 * f, f14 * f, f17 * f);
            let n: i32 = block.getBlockTexture(&*world, x, y, z, 4);
            self.renderNorthFace(block, dx, dy, dz, n);
            if RenderBlocks::fancyGrass() && n == 3 && self.m_overrideBlockTexture < 0
            {
                tessellator.setColorOpaque_F(f11 * f * r, f14 * f * g, f17 * f * b);
                self.renderNorthFace(block, dx, dy, dz, 38);
            }
            bl = true;
        }
        if self.m_renderAllFaces || block.shouldSideBeRendered(&*world, x + 1, y, z, 5)
        {
            let mut f: f32 = block.getBlockBrightness(&*world, x + 1, y, z);
            if block.m_maxX.get() < 1.0
            {
                f = f18;
            }
            tessellator.setColorOpaque_F(f11 * f, f14 * f, f17 * f);
            let n: i32 = block.getBlockTexture(&*world, x, y, z, 5);
            self.renderSouthFace(block, dx, dy, dz, n);
            if RenderBlocks::fancyGrass() && n == 3 && self.m_overrideBlockTexture < 0
            {
                tessellator.setColorOpaque_F(f11 * f * r, f14 * f * g, f17 * f * b);
                self.renderSouthFace(block, dx, dy, dz, 38);
            }
            bl = true;
        }
        bl
    }

    pub fn renderBlockCactus(&mut self, block: &Rc<Block>, x: i32, y: i32, z: i32) -> bool
    {
        let (f, f2, f3): (f32, f32, f32) = colorChannels(block.colorMultiplier(&*self.world(), x, y, z));
        self.tesselateCactus(block, x, y, z, f, f2, f3)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn tesselateCactus(&mut self, block: &Rc<Block>, x: i32, y: i32, z: i32, r: f32, g: f32, b: f32) -> bool
    {
        let world: Rc<dyn IBlockAccess> = self.world();
        let tessellator: Rc<Tessellator> = Tessellator::instance();
        let mut bl: bool = false;
        let f2: f32 = 0.5;
        let f3: f32 = 1.0;
        let f4: f32 = 0.8;
        let f5: f32 = 0.6;
        let (f6, f7, f8, f9): (f32, f32, f32, f32) = (f2 * r, f3 * r, f4 * r, f5 * r);
        let (f10, f11, f12, f13): (f32, f32, f32, f32) = (f2 * g, f3 * g, f4 * g, f5 * g);
        let (f14, f15, f16, f17): (f32, f32, f32, f32) = (f2 * b, f3 * b, f4 * b, f5 * b);
        let f18: f32 = 0.0625;
        let f19: f32 = block.getBlockBrightness(&*world, x, y, z);
        let (dx, dy, dz): (f64, f64, f64) = (x as f64, y as f64, z as f64);
        if self.m_renderAllFaces || block.shouldSideBeRendered(&*world, x, y - 1, z, 0)
        {
            let f: f32 = block.getBlockBrightness(&*world, x, y - 1, z);
            tessellator.setColorOpaque_F(f6 * f, f10 * f, f14 * f);
            self.renderBottomFace(block, dx, dy, dz, block.getBlockTexture(&*world, x, y, z, 0));
            bl = true;
        }
        if self.m_renderAllFaces || block.shouldSideBeRendered(&*world, x, y + 1, z, 1)
        {
            let mut f: f32 = block.getBlockBrightness(&*world, x, y + 1, z);
            if block.m_maxY.get() != 1.0 && !block.m_blockMaterial.getIsLiquid()
            {
                f = f19;
            }
            tessellator.setColorOpaque_F(f7 * f, f11 * f, f15 * f);
            self.renderTopFace(block, dx, dy, dz, block.getBlockTexture(&*world, x, y, z, 1));
            bl = true;
        }
        if self.m_renderAllFaces || block.shouldSideBeRendered(&*world, x, y, z - 1, 2)
        {
            let mut f: f32 = block.getBlockBrightness(&*world, x, y, z - 1);
            if block.m_minZ.get() > 0.0
            {
                f = f19;
            }
            tessellator.setColorOpaque_F(f8 * f, f12 * f, f16 * f);
            tessellator.setTranslationF(0.0, 0.0, f18);
            self.renderEastFace(block, dx, dy, dz, block.getBlockTexture(&*world, x, y, z, 2));
            tessellator.setTranslationF(0.0, 0.0, -f18);
            bl = true;
        }
        if self.m_renderAllFaces || block.shouldSideBeRendered(&*world, x, y, z + 1, 3)
        {
            let mut f: f32 = block.getBlockBrightness(&*world, x, y, z + 1);
            if block.m_maxZ.get() < 1.0
            {
                f = f19;
            }
            tessellator.setColorOpaque_F(f8 * f, f12 * f, f16 * f);
            tessellator.setTranslationF(0.0, 0.0, -f18);
            self.renderWestFace(block, dx, dy, dz, block.getBlockTexture(&*world, x, y, z, 3));
            tessellator.setTranslationF(0.0, 0.0, f18);
            bl = true;
        }
        if self.m_renderAllFaces || block.shouldSideBeRendered(&*world, x - 1, y, z, 4)
        {
            let mut f: f32 = block.getBlockBrightness(&*world, x - 1, y, z);
            if block.m_minX.get() > 0.0
            {
                f = f19;
            }
            tessellator.setColorOpaque_F(f9 * f, f13 * f, f17 * f);
            tessellator.setTranslationF(f18, 0.0, 0.0);
            self.renderNorthFace(block, dx, dy, dz, block.getBlockTexture(&*world, x, y, z, 4));
            tessellator.setTranslationF(-f18, 0.0, 0.0);
            bl = true;
        }
        if self.m_renderAllFaces || block.shouldSideBeRendered(&*world, x + 1, y, z, 5)
        {
            let mut f: f32 = block.getBlockBrightness(&*world, x + 1, y, z);
            if block.m_maxX.get() < 1.0
            {
                f = f19;
            }
            tessellator.setColorOpaque_F(f9 * f, f13 * f, f17 * f);
            tessellator.setTranslationF(-f18, 0.0, 0.0);
            self.renderSouthFace(block, dx, dy, dz, block.getBlockTexture(&*world, x, y, z, 5));
            tessellator.setTranslationF(f18, 0.0, 0.0);
            bl = true;
        }
        bl
    }
}
