use super::{RenderBlocks, spriteOrigin};
use crate::render::Tessellator;
use game::block::Block;
use std::rc::Rc;

type Vertex = (f64, f64, f64, f64, f64);

type FaceUv = [f64; 8];

fn fullSpan(origin: f64) -> (f64, f64) { ((origin as f32 / 256.0) as f64, ((origin as f32 + 15.99) / 256.0) as f64) }

struct SideRotations
{
    m_rotationA: i32,
    m_edgeA: (f64, f64),
    m_rotationB: i32,
    m_edgeB: (f64, f64),
}

impl RenderBlocks
{
    fn spriteFor(&self, sprite: i32) -> (f64, f64)
    {
        let (n, n2): (i32, i32) = spriteOrigin(
            if self.m_overrideBlockTexture >= 0
            {
                self.m_overrideBlockTexture
            }
            else
            {
                sprite
            },
        );
        (n as f64, n2 as f64)
    }

    fn emitFace(&self, vertices: [Vertex; 4])
    {
        let tessellator: Rc<Tessellator> = Tessellator::instance();
        let ao: &super::AoState = &self.m_ao;
        let colors: [(f32, f32, f32); 4] = [
            (ao.m_colorRedTopLeft, ao.m_colorGreenTopLeft, ao.m_colorBlueTopLeft),
            (ao.m_colorRedBottomLeft, ao.m_colorGreenBottomLeft, ao.m_colorBlueBottomLeft),
            (ao.m_colorRedBottomRight, ao.m_colorGreenBottomRight, ao.m_colorBlueBottomRight),
            (ao.m_colorRedTopRight, ao.m_colorGreenTopRight, ao.m_colorBlueTopRight),
        ];
        for ((x, y, z, u, v), (r, g, b)) in vertices.into_iter().zip(colors)
        {
            if self.m_enableAO
            {
                tessellator.setColorOpaque_F(r, g, b);
            }
            tessellator.addVertexWithUV(x, y, z, u, v);
        }
    }

    fn horizontalUv(&self, block: &Block, sprite: i32, rotation: i32, rotationA: i32) -> FaceUv
    {
        let (n, n2): (f64, f64) = self.spriteFor(sprite);
        let (minX, maxX, minZ, maxZ): (f64, f64, f64, f64) =
            (block.m_minX.get(), block.m_maxX.get(), block.m_minZ.get(), block.m_maxZ.get());
        let mut d: f64 = (n + minX * 16.0) / 256.0;
        let mut d2: f64 = (n + maxX * 16.0 - 0.01) / 256.0;
        let mut d3: f64 = (n2 + minZ * 16.0) / 256.0;
        let mut d4: f64 = (n2 + maxZ * 16.0 - 0.01) / 256.0;
        if minX < 0.0 || maxX > 1.0
        {
            (d, d2) = fullSpan(n);
        }
        if minZ < 0.0 || maxZ > 1.0
        {
            (d3, d4) = fullSpan(n2);
        }
        let (mut d5, mut d6, mut d7, mut d8): (f64, f64, f64, f64) = (d2, d, d3, d4);
        if rotation == rotationA
        {
            d = (n + minZ * 16.0) / 256.0;
            let d3o: f64 = (n2 + 16.0 - maxX * 16.0) / 256.0;
            d2 = (n + maxZ * 16.0) / 256.0;
            let d4o: f64 = (n2 + 16.0 - minX * 16.0) / 256.0;
            (d5, d6, d7, d8) = (d, d2, d3o, d4o);
            (d3, d4) = (d4o, d3o);
        }
        else if rotation == 3 - rotationA
        {
            let d1o: f64 = (n + 16.0 - maxZ * 16.0) / 256.0;
            d3 = (n2 + minX * 16.0) / 256.0;
            let d2o: f64 = (n + 16.0 - minZ * 16.0) / 256.0;
            d4 = (n2 + maxX * 16.0) / 256.0;
            (d, d2, d5, d6) = (d2o, d1o, d2o, d1o);
            (d7, d8) = (d4, d3);
        }
        else if rotation == 3
        {
            d = (n + 16.0 - minX * 16.0) / 256.0;
            d2 = (n + 16.0 - maxX * 16.0 - 0.01) / 256.0;
            d3 = (n2 + 16.0 - minZ * 16.0) / 256.0;
            d4 = (n2 + 16.0 - maxZ * 16.0 - 0.01) / 256.0;
            (d5, d6, d7, d8) = (d2, d, d3, d4);
        }
        [d, d2, d3, d4, d5, d6, d7, d8]
    }

    fn sideUv(&self, block: &Block, sprite: i32, rotation: i32, axis: (f64, f64), rotations: SideRotations) -> FaceUv
    {
        let (n, n2): (f64, f64) = self.spriteFor(sprite);
        let (minA, maxA): (f64, f64) = axis;
        let (minY, maxY): (f64, f64) = (block.m_minY.get(), block.m_maxY.get());
        let mut d2: f64 = (n + minA * 16.0) / 256.0;
        let mut d3: f64 = (n + maxA * 16.0 - 0.01) / 256.0;
        let mut d4: f64 = (n2 + 16.0 - maxY * 16.0) / 256.0;
        let mut d5: f64 = (n2 + 16.0 - minY * 16.0 - 0.01) / 256.0;
        if self.m_flipTexture
        {
            std::mem::swap(&mut d2, &mut d3);
        }
        if minA < 0.0 || maxA > 1.0
        {
            (d2, d3) = fullSpan(n);
        }
        if minY < 0.0 || maxY > 1.0
        {
            (d4, d5) = fullSpan(n2);
        }
        let (mut d, mut d6, mut d7, mut d8): (f64, f64, f64, f64) = (d3, d2, d4, d5);
        if rotation == rotations.m_rotationA
        {
            d2 = (n + minY * 16.0) / 256.0;
            let d4o: f64 = (n2 + 16.0 - rotations.m_edgeA.0 * 16.0) / 256.0;
            d3 = (n + maxY * 16.0) / 256.0;
            let d5o: f64 = (n2 + 16.0 - rotations.m_edgeA.1 * 16.0) / 256.0;
            (d, d6, d7, d8) = (d2, d3, d4o, d5o);
            (d4, d5) = (d5o, d4o);
        }
        else if rotation == rotations.m_rotationB
        {
            let d2o: f64 = (n + 16.0 - maxY * 16.0) / 256.0;
            d4 = (n2 + rotations.m_edgeB.0 * 16.0) / 256.0;
            let d3o: f64 = (n + 16.0 - minY * 16.0) / 256.0;
            d5 = (n2 + rotations.m_edgeB.1 * 16.0) / 256.0;
            (d, d6, d2, d3) = (d3o, d2o, d3o, d2o);
            (d7, d8) = (d5, d4);
        }
        else if rotation == 3
        {
            d2 = (n + 16.0 - minA * 16.0) / 256.0;
            d3 = (n + 16.0 - maxA * 16.0 - 0.01) / 256.0;
            d4 = (n2 + maxY * 16.0) / 256.0;
            d5 = (n2 + minY * 16.0 - 0.01) / 256.0;
            (d, d6, d7, d8) = (d3, d2, d4, d5);
        }
        [d, d2, d3, d4, d5, d6, d7, d8]
    }

    pub fn renderBottomFace(&self, block: &Rc<Block>, x: f64, y: f64, z: f64, sprite: i32)
    {
        let [d, d2, d3, d4, d5, d6, d7, d8]: FaceUv = self.horizontalUv(block, sprite, self.m_bottomFaceRotation, 2);
        let d9: f64 = x + block.m_minX.get();
        let d10: f64 = x + block.m_maxX.get();
        let d11: f64 = y + block.m_minY.get();
        let d12: f64 = z + block.m_minZ.get();
        let d13: f64 = z + block.m_maxZ.get();
        self.emitFace([
            (d9, d11, d13, d6, d8),
            (d9, d11, d12, d, d3),
            (d10, d11, d12, d5, d7),
            (d10, d11, d13, d2, d4),
        ]);
    }

    pub fn renderTopFace(&self, block: &Rc<Block>, x: f64, y: f64, z: f64, sprite: i32)
    {
        let [d, d2, d3, d4, d5, d6, d7, d8]: FaceUv = self.horizontalUv(block, sprite, self.m_topFaceRotation, 1);
        let d9: f64 = x + block.m_minX.get();
        let d10: f64 = x + block.m_maxX.get();
        let d11: f64 = y + block.m_maxY.get();
        let d12: f64 = z + block.m_minZ.get();
        let d13: f64 = z + block.m_maxZ.get();
        self.emitFace([
            (d10, d11, d13, d2, d4),
            (d10, d11, d12, d5, d7),
            (d9, d11, d12, d, d3),
            (d9, d11, d13, d6, d8),
        ]);
    }

    pub fn renderEastFace(&self, block: &Rc<Block>, x: f64, y: f64, z: f64, sprite: i32)
    {
        let (minX, maxX): (f64, f64) = (block.m_minX.get(), block.m_maxX.get());
        let rotations: SideRotations =
            SideRotations { m_rotationA: 2, m_edgeA: (minX, maxX), m_rotationB: 1, m_edgeB: (maxX, minX) };
        let [d, d2, d3, d4, d5, d6, d7, d8]: FaceUv =
            self.sideUv(block, sprite, self.m_northFaceRotation, (minX, maxX), rotations);
        let d9: f64 = x + minX;
        let d10: f64 = x + maxX;
        let d11: f64 = y + block.m_minY.get();
        let d12: f64 = y + block.m_maxY.get();
        let d13: f64 = z + block.m_minZ.get();
        self.emitFace([
            (d9, d12, d13, d, d7),
            (d10, d12, d13, d2, d4),
            (d10, d11, d13, d6, d8),
            (d9, d11, d13, d3, d5),
        ]);
    }

    pub fn renderWestFace(&self, block: &Rc<Block>, x: f64, y: f64, z: f64, sprite: i32)
    {
        let (minX, maxX): (f64, f64) = (block.m_minX.get(), block.m_maxX.get());
        let rotations: SideRotations =
            SideRotations { m_rotationA: 1, m_edgeA: (maxX, minX), m_rotationB: 2, m_edgeB: (minX, maxX) };
        let [d, d2, d3, d4, d5, d6, d7, d8]: FaceUv =
            self.sideUv(block, sprite, self.m_southFaceRotation, (minX, maxX), rotations);
        let d9: f64 = x + minX;
        let d10: f64 = x + maxX;
        let d11: f64 = y + block.m_minY.get();
        let d12: f64 = y + block.m_maxY.get();
        let d13: f64 = z + block.m_maxZ.get();
        self.emitFace([
            (d9, d12, d13, d2, d4),
            (d9, d11, d13, d6, d8),
            (d10, d11, d13, d3, d5),
            (d10, d12, d13, d, d7),
        ]);
    }

    pub fn renderNorthFace(&self, block: &Rc<Block>, x: f64, y: f64, z: f64, sprite: i32)
    {
        let (minZ, maxZ): (f64, f64) = (block.m_minZ.get(), block.m_maxZ.get());
        let rotations: SideRotations =
            SideRotations { m_rotationA: 1, m_edgeA: (maxZ, minZ), m_rotationB: 2, m_edgeB: (minZ, maxZ) };
        let [d, d2, d3, d4, d5, d6, d7, d8]: FaceUv =
            self.sideUv(block, sprite, self.m_eastFaceRotation, (minZ, maxZ), rotations);
        let d9: f64 = x + block.m_minX.get();
        let d10: f64 = y + block.m_minY.get();
        let d11: f64 = y + block.m_maxY.get();
        let d12: f64 = z + minZ;
        let d13: f64 = z + maxZ;
        self.emitFace([(d9, d11, d13, d, d7), (d9, d11, d12, d2, d4), (d9, d10, d12, d6, d8), (d9, d10, d13, d3, d5)]);
    }

    pub fn renderSouthFace(&self, block: &Rc<Block>, x: f64, y: f64, z: f64, sprite: i32)
    {
        let (minZ, maxZ): (f64, f64) = (block.m_minZ.get(), block.m_maxZ.get());
        let rotations: SideRotations =
            SideRotations { m_rotationA: 2, m_edgeA: (minZ, maxZ), m_rotationB: 1, m_edgeB: (maxZ, minZ) };
        let [d, d2, d3, d4, d5, d6, d7, d8]: FaceUv =
            self.sideUv(block, sprite, self.m_westFaceRotation, (minZ, maxZ), rotations);
        let d9: f64 = x + block.m_maxX.get();
        let d10: f64 = y + block.m_minY.get();
        let d11: f64 = y + block.m_maxY.get();
        let d12: f64 = z + minZ;
        let d13: f64 = z + maxZ;
        self.emitFace([(d9, d10, d13, d6, d8), (d9, d10, d12, d3, d5), (d9, d11, d12, d, d7), (d9, d11, d13, d2, d4)]);
    }
}
