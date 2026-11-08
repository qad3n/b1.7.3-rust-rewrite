use super::{RenderBlocks, spriteOrigin};
use crate::render::Tessellator;
use game::block::{Block, block_piston_base, block_piston_extension};
use game::world::IBlockAccess;
use std::rc::Rc;

#[derive(Clone, Copy)]
enum PistonSide
{
    X,
    Y,
    Z,
}

impl RenderBlocks
{
    fn setPistonFaceRotations(&mut self, facing: i32)
    {
        match facing
        {
            0 =>
            {
                self.m_northFaceRotation = 3;
                self.m_southFaceRotation = 3;
                self.m_westFaceRotation = 3;
                self.m_eastFaceRotation = 3;
            }
            2 =>
            {
                self.m_westFaceRotation = 1;
                self.m_eastFaceRotation = 2;
            }
            3 =>
            {
                self.m_westFaceRotation = 2;
                self.m_eastFaceRotation = 1;
                self.m_topFaceRotation = 3;
                self.m_bottomFaceRotation = 3;
            }
            4 =>
            {
                self.m_northFaceRotation = 1;
                self.m_southFaceRotation = 2;
                self.m_topFaceRotation = 2;
                self.m_bottomFaceRotation = 1;
            }
            5 =>
            {
                self.m_northFaceRotation = 2;
                self.m_southFaceRotation = 1;
                self.m_topFaceRotation = 1;
                self.m_bottomFaceRotation = 2;
            }
            _ =>
            {}
        }
    }

    fn resetFaceRotations(&mut self)
    {
        self.m_northFaceRotation = 0;
        self.m_southFaceRotation = 0;
        self.m_westFaceRotation = 0;
        self.m_eastFaceRotation = 0;
        self.m_topFaceRotation = 0;
        self.m_bottomFaceRotation = 0;
    }

    pub fn renderExtendedPistonBase(&mut self, block: &Rc<Block>, x: i32, y: i32, z: i32)
    {
        self.m_renderAllFaces = true;
        self.tesselatePistonBase(block, x, y, z, true);
        self.m_renderAllFaces = false;
    }

    pub(super) fn tesselatePistonBase(&mut self, block: &Rc<Block>, x: i32, y: i32, z: i32, extended: bool) -> bool
    {
        let n: i32 = self.world().getBlockMetadata(x, y, z);
        let bl: bool = extended || (n & 8) != 0;
        let n2: i32 = block_piston_base::getFacing(n);
        self.setPistonFaceRotations(n2);
        if bl
        {
            match n2
            {
                0 => block.setBlockBounds(0.0, 0.25, 0.0, 1.0, 1.0, 1.0),
                1 => block.setBlockBounds(0.0, 0.0, 0.0, 1.0, 0.75, 1.0),
                2 => block.setBlockBounds(0.0, 0.0, 0.25, 1.0, 1.0, 1.0),
                3 => block.setBlockBounds(0.0, 0.0, 0.0, 1.0, 1.0, 0.75),
                4 => block.setBlockBounds(0.25, 0.0, 0.0, 1.0, 1.0, 1.0),
                5 => block.setBlockBounds(0.0, 0.0, 0.0, 0.75, 1.0, 1.0),
                _ =>
                {}
            }
        }
        self.renderStandardBlock(block, x, y, z);
        self.resetFaceRotations();
        if bl
        {
            block.setBlockBounds(0.0, 0.0, 0.0, 1.0, 1.0, 1.0);
        }
        true
    }

    #[allow(clippy::too_many_arguments)]
    fn tesselatePistonSide(
        &self,
        side: PistonSide,
        minX: f32,
        maxX: f32,
        minY: f32,
        maxY: f32,
        minZ: f32,
        maxZ: f32,
        color: f32,
        length: f64,
    )
    {
        let mut n: i32 = 108;
        if self.m_overrideBlockTexture >= 0
        {
            n = self.m_overrideBlockTexture;
        }
        let (n2, n3): (i32, i32) = spriteOrigin(n);
        let tessellator: Rc<Tessellator> = Tessellator::instance();
        let d: f64 = (n2 as f32 / 256.0) as f64;
        let d2: f64 = (n3 as f32 / 256.0) as f64;
        let d3: f64 = (n2 as f64 + length - 0.01) / 256.0;
        let d4: f64 = ((n3 as f32 + 4.0) as f64 - 0.01) / 256.0;
        let (minX, maxX, minY, maxY, minZ, maxZ): (f64, f64, f64, f64, f64, f64) =
            (minX as f64, maxX as f64, minY as f64, maxY as f64, minZ as f64, maxZ as f64);
        tessellator.setColorOpaque_F(color, color, color);
        match side
        {
            PistonSide::X =>
            {
                tessellator.addVertexWithUV(minX, maxY, minZ, d3, d2);
                tessellator.addVertexWithUV(minX, minY, minZ, d, d2);
                tessellator.addVertexWithUV(maxX, minY, maxZ, d, d4);
                tessellator.addVertexWithUV(maxX, maxY, maxZ, d3, d4);
            }
            PistonSide::Y =>
            {
                tessellator.addVertexWithUV(minX, minY, maxZ, d3, d2);
                tessellator.addVertexWithUV(minX, minY, minZ, d, d2);
                tessellator.addVertexWithUV(maxX, maxY, minZ, d, d4);
                tessellator.addVertexWithUV(maxX, maxY, maxZ, d3, d4);
            }
            PistonSide::Z =>
            {
                tessellator.addVertexWithUV(maxX, minY, minZ, d3, d2);
                tessellator.addVertexWithUV(minX, minY, minZ, d, d2);
                tessellator.addVertexWithUV(minX, maxY, maxZ, d, d4);
                tessellator.addVertexWithUV(maxX, maxY, maxZ, d3, d4);
            }
        }
    }

    pub fn renderPistonHead(&mut self, block: &Rc<Block>, x: i32, y: i32, z: i32, isShort: bool)
    {
        self.m_renderAllFaces = true;
        self.tesselatePistonHead(block, x, y, z, isShort);
        self.m_renderAllFaces = false;
    }

    pub(super) fn tesselatePistonHead(&mut self, block: &Rc<Block>, x: i32, y: i32, z: i32, isShort: bool) -> bool
    {
        let world: Rc<dyn IBlockAccess> = self.world();
        let n: i32 = world.getBlockMetadata(x, y, z);
        let n2: i32 = block_piston_extension::getFacing(n);
        let f: f32 = block.getBlockBrightness(&*world, x, y, z);
        let f2: f32 = if isShort { 1.0 } else { 0.5 };
        let d: f64 = if isShort { 16.0 } else { 8.0 };
        let (x0, y0, z0): (f32, f32, f32) = (x as f32, y as f32, z as f32);
        self.setPistonFaceRotations(n2);
        match n2
        {
            0 =>
            {
                block.setBlockBounds(0.0, 0.0, 0.0, 1.0, 0.25, 1.0);
                self.renderStandardBlock(block, x, y, z);
                let (lo, hi): (f32, f32) = (y0 + 0.25, y0 + 0.25 + f2);
                self.tesselatePistonSide(
                    PistonSide::X,
                    x0 + 0.375,
                    x0 + 0.625,
                    lo,
                    hi,
                    z0 + 0.625,
                    z0 + 0.625,
                    f * 0.8,
                    d,
                );
                self.tesselatePistonSide(
                    PistonSide::X,
                    x0 + 0.625,
                    x0 + 0.375,
                    lo,
                    hi,
                    z0 + 0.375,
                    z0 + 0.375,
                    f * 0.8,
                    d,
                );
                self.tesselatePistonSide(
                    PistonSide::X,
                    x0 + 0.375,
                    x0 + 0.375,
                    lo,
                    hi,
                    z0 + 0.375,
                    z0 + 0.625,
                    f * 0.6,
                    d,
                );
                self.tesselatePistonSide(
                    PistonSide::X,
                    x0 + 0.625,
                    x0 + 0.625,
                    lo,
                    hi,
                    z0 + 0.625,
                    z0 + 0.375,
                    f * 0.6,
                    d,
                );
            }
            1 =>
            {
                block.setBlockBounds(0.0, 0.75, 0.0, 1.0, 1.0, 1.0);
                self.renderStandardBlock(block, x, y, z);
                let (lo, hi): (f32, f32) = (y0 - 0.25 + 1.0 - f2, y0 - 0.25 + 1.0);
                self.tesselatePistonSide(
                    PistonSide::X,
                    x0 + 0.375,
                    x0 + 0.625,
                    lo,
                    hi,
                    z0 + 0.625,
                    z0 + 0.625,
                    f * 0.8,
                    d,
                );
                self.tesselatePistonSide(
                    PistonSide::X,
                    x0 + 0.625,
                    x0 + 0.375,
                    lo,
                    hi,
                    z0 + 0.375,
                    z0 + 0.375,
                    f * 0.8,
                    d,
                );
                self.tesselatePistonSide(
                    PistonSide::X,
                    x0 + 0.375,
                    x0 + 0.375,
                    lo,
                    hi,
                    z0 + 0.375,
                    z0 + 0.625,
                    f * 0.6,
                    d,
                );
                self.tesselatePistonSide(
                    PistonSide::X,
                    x0 + 0.625,
                    x0 + 0.625,
                    lo,
                    hi,
                    z0 + 0.625,
                    z0 + 0.375,
                    f * 0.6,
                    d,
                );
            }
            2 =>
            {
                block.setBlockBounds(0.0, 0.0, 0.0, 1.0, 1.0, 0.25);
                self.renderStandardBlock(block, x, y, z);
                let (lo, hi): (f32, f32) = (z0 + 0.25, z0 + 0.25 + f2);
                self.tesselatePistonSide(
                    PistonSide::Y,
                    x0 + 0.375,
                    x0 + 0.375,
                    y0 + 0.625,
                    y0 + 0.375,
                    lo,
                    hi,
                    f * 0.6,
                    d,
                );
                self.tesselatePistonSide(
                    PistonSide::Y,
                    x0 + 0.625,
                    x0 + 0.625,
                    y0 + 0.375,
                    y0 + 0.625,
                    lo,
                    hi,
                    f * 0.6,
                    d,
                );
                self.tesselatePistonSide(
                    PistonSide::Y,
                    x0 + 0.375,
                    x0 + 0.625,
                    y0 + 0.375,
                    y0 + 0.375,
                    lo,
                    hi,
                    f * 0.5,
                    d,
                );
                self.tesselatePistonSide(PistonSide::Y, x0 + 0.625, x0 + 0.375, y0 + 0.625, y0 + 0.625, lo, hi, f, d);
            }
            3 =>
            {
                block.setBlockBounds(0.0, 0.0, 0.75, 1.0, 1.0, 1.0);
                self.renderStandardBlock(block, x, y, z);
                let (lo, hi): (f32, f32) = (z0 - 0.25 + 1.0 - f2, z0 - 0.25 + 1.0);
                self.tesselatePistonSide(
                    PistonSide::Y,
                    x0 + 0.375,
                    x0 + 0.375,
                    y0 + 0.625,
                    y0 + 0.375,
                    lo,
                    hi,
                    f * 0.6,
                    d,
                );
                self.tesselatePistonSide(
                    PistonSide::Y,
                    x0 + 0.625,
                    x0 + 0.625,
                    y0 + 0.375,
                    y0 + 0.625,
                    lo,
                    hi,
                    f * 0.6,
                    d,
                );
                self.tesselatePistonSide(
                    PistonSide::Y,
                    x0 + 0.375,
                    x0 + 0.625,
                    y0 + 0.375,
                    y0 + 0.375,
                    lo,
                    hi,
                    f * 0.5,
                    d,
                );
                self.tesselatePistonSide(PistonSide::Y, x0 + 0.625, x0 + 0.375, y0 + 0.625, y0 + 0.625, lo, hi, f, d);
            }
            4 =>
            {
                block.setBlockBounds(0.0, 0.0, 0.0, 0.25, 1.0, 1.0);
                self.renderStandardBlock(block, x, y, z);
                let (lo, hi): (f32, f32) = (x0 + 0.25, x0 + 0.25 + f2);
                self.tesselatePistonSide(
                    PistonSide::Z,
                    lo,
                    hi,
                    y0 + 0.375,
                    y0 + 0.375,
                    z0 + 0.625,
                    z0 + 0.375,
                    f * 0.5,
                    d,
                );
                self.tesselatePistonSide(PistonSide::Z, lo, hi, y0 + 0.625, y0 + 0.625, z0 + 0.375, z0 + 0.625, f, d);
                self.tesselatePistonSide(
                    PistonSide::Z,
                    lo,
                    hi,
                    y0 + 0.375,
                    y0 + 0.625,
                    z0 + 0.375,
                    z0 + 0.375,
                    f * 0.6,
                    d,
                );
                self.tesselatePistonSide(
                    PistonSide::Z,
                    lo,
                    hi,
                    y0 + 0.625,
                    y0 + 0.375,
                    z0 + 0.625,
                    z0 + 0.625,
                    f * 0.6,
                    d,
                );
            }
            5 =>
            {
                block.setBlockBounds(0.75, 0.0, 0.0, 1.0, 1.0, 1.0);
                self.renderStandardBlock(block, x, y, z);
                let (lo, hi): (f32, f32) = (x0 - 0.25 + 1.0 - f2, x0 - 0.25 + 1.0);
                self.tesselatePistonSide(
                    PistonSide::Z,
                    lo,
                    hi,
                    y0 + 0.375,
                    y0 + 0.375,
                    z0 + 0.625,
                    z0 + 0.375,
                    f * 0.5,
                    d,
                );
                self.tesselatePistonSide(PistonSide::Z, lo, hi, y0 + 0.625, y0 + 0.625, z0 + 0.375, z0 + 0.625, f, d);
                self.tesselatePistonSide(
                    PistonSide::Z,
                    lo,
                    hi,
                    y0 + 0.375,
                    y0 + 0.625,
                    z0 + 0.375,
                    z0 + 0.375,
                    f * 0.6,
                    d,
                );
                self.tesselatePistonSide(
                    PistonSide::Z,
                    lo,
                    hi,
                    y0 + 0.625,
                    y0 + 0.375,
                    z0 + 0.625,
                    z0 + 0.625,
                    f * 0.6,
                    d,
                );
            }
            _ =>
            {}
        }
        self.resetFaceRotations();
        block.setBlockBounds(0.0, 0.0, 0.0, 1.0, 1.0, 1.0);
        true
    }
}
