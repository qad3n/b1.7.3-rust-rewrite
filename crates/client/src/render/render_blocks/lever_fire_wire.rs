use super::{RenderBlocks, spriteOrigin};
use crate::render::Tessellator;
use game::block::{Block, block_fire, block_redstone_wire};
use game::math::Vec3D;
use game::world::IBlockAccess;
use std::f64::consts::PI;
use std::rc::Rc;

impl RenderBlocks
{
    pub fn renderBlockLever(&mut self, block: &Rc<Block>, x: i32, y: i32, z: i32) -> bool
    {
        let world: Rc<dyn IBlockAccess> = self.world();
        let i: i32 = world.getBlockMetadata(x, y, z);
        let j: i32 = i & 7;
        let flag: bool = (i & 8) > 0;
        let tessellator: Rc<Tessellator> = Tessellator::instance();
        let flag1: bool = self.m_overrideBlockTexture >= 0;
        if !flag1
        {
            self.m_overrideBlockTexture = Block::cobblestone().m_blockIndexInTexture.get();
        }
        let f: f32 = 0.25;
        let f1: f32 = 0.1875;
        let f2: f32 = 0.1875;
        match j
        {
            5 => block.setBlockBounds(0.5 - f1, 0.0, 0.5 - f, 0.5 + f1, f2, 0.5 + f),
            6 => block.setBlockBounds(0.5 - f, 0.0, 0.5 - f1, 0.5 + f, f2, 0.5 + f1),
            4 => block.setBlockBounds(0.5 - f1, 0.5 - f, 1.0 - f2, 0.5 + f1, 0.5 + f, 1.0),
            3 => block.setBlockBounds(0.5 - f1, 0.5 - f, 0.0, 0.5 + f1, 0.5 + f, f2),
            2 => block.setBlockBounds(1.0 - f2, 0.5 - f, 0.5 - f1, 1.0, 0.5 + f, 0.5 + f1),
            1 => block.setBlockBounds(0.0, 0.5 - f, 0.5 - f1, f2, 0.5 + f, 0.5 + f1),
            _ =>
            {}
        }
        self.renderStandardBlock(block, x, y, z);
        if !flag1
        {
            self.m_overrideBlockTexture = -1;
        }
        let mut f3: f32 = block.getBlockBrightness(&*world, x, y, z);
        if Block::lightValue(block.m_blockID) > 0
        {
            f3 = 1.0;
        }
        tessellator.setColorOpaque_F(f3, f3, f3);
        let mut k: i32 = block.getBlockTextureFromSide(0);
        if self.m_overrideBlockTexture >= 0
        {
            k = self.m_overrideBlockTexture;
        }
        let (l, i1): (i32, i32) = spriteOrigin(k);
        let mut f4: f32 = l as f32 / 256.0;
        let mut f5: f32 = (l as f32 + 15.99) / 256.0;
        let mut f6: f32 = i1 as f32 / 256.0;
        let mut f7: f32 = (i1 as f32 + 15.99) / 256.0;
        let f8: f64 = 0.0625f32 as f64;
        let f9: f64 = 0.0625f32 as f64;
        let f10: f64 = 0.625f32 as f64;
        let avec3d: [Rc<Vec3D>; 8] = [
            Vec3D::createVector(-f8, 0.0, -f9),
            Vec3D::createVector(f8, 0.0, -f9),
            Vec3D::createVector(f8, 0.0, f9),
            Vec3D::createVector(-f8, 0.0, f9),
            Vec3D::createVector(-f8, f10, -f9),
            Vec3D::createVector(f8, f10, -f9),
            Vec3D::createVector(f8, f10, f9),
            Vec3D::createVector(-f8, f10, f9),
        ];
        for vec in &avec3d
        {
            if flag
            {
                vec.m_zCoord.set(vec.m_zCoord.get() - 0.0625);
                vec.rotateAroundX(PI as f32 * 2.0 / 9.0);
            }
            else
            {
                vec.m_zCoord.set(vec.m_zCoord.get() + 0.0625);
                vec.rotateAroundX(-PI as f32 * 2.0 / 9.0);
            }
            if j == 6
            {
                vec.rotateAroundY((PI / 2.0) as f32);
            }
            if j < 5
            {
                vec.m_yCoord.set(vec.m_yCoord.get() - 0.375);
                vec.rotateAroundX((PI / 2.0) as f32);
                match j
                {
                    4 => vec.rotateAroundY(0.0),
                    3 => vec.rotateAroundY(PI as f32),
                    2 => vec.rotateAroundY((PI / 2.0) as f32),
                    1 => vec.rotateAroundY((-PI / 2.0) as f32),
                    _ =>
                    {}
                }
                vec.m_xCoord.set(vec.m_xCoord.get() + (x as f64 + 0.5));
                vec.m_yCoord.set(vec.m_yCoord.get() + (y as f32 + 0.5) as f64);
                vec.m_zCoord.set(vec.m_zCoord.get() + (z as f64 + 0.5));
            }
            else
            {
                vec.m_xCoord.set(vec.m_xCoord.get() + (x as f64 + 0.5));
                vec.m_yCoord.set(vec.m_yCoord.get() + (y as f32 + 0.125) as f64);
                vec.m_zCoord.set(vec.m_zCoord.get() + (z as f64 + 0.5));
            }
        }
        for k1 in 0..6
        {
            if k1 == 0
            {
                f4 = (l + 7) as f32 / 256.0;
                f5 = ((l + 9) as f32 - 0.01) / 256.0;
                f6 = (i1 + 6) as f32 / 256.0;
                f7 = ((i1 + 8) as f32 - 0.01) / 256.0;
            }
            else if k1 == 2
            {
                f4 = (l + 7) as f32 / 256.0;
                f5 = ((l + 9) as f32 - 0.01) / 256.0;
                f6 = (i1 + 6) as f32 / 256.0;
                f7 = ((i1 + 16) as f32 - 0.01) / 256.0;
            }
            let corners: [usize; 4] = match k1
            {
                0 => [0, 1, 2, 3],
                1 => [7, 6, 5, 4],
                2 => [1, 0, 4, 5],
                3 => [2, 1, 5, 6],
                4 => [3, 2, 6, 7],
                _ => [0, 3, 7, 4],
            };
            let uv: [(f32, f32); 4] = [(f4, f7), (f5, f7), (f5, f6), (f4, f6)];
            for (corner, (u, v)) in corners.iter().zip(uv)
            {
                let vec: &Vec3D = &avec3d[*corner];
                tessellator.addVertexWithUV(vec.x(), vec.y(), vec.z(), u as f64, v as f64);
            }
        }
        true
    }

    pub fn renderBlockFire(&mut self, block: &Rc<Block>, x: i32, mut y: i32, z: i32) -> bool
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
        let lower = |sprite: i32, row: i32| -> (f64, f64, f64, f64) {
            (
                (sprite as f32 / 256.0) as f64,
                ((sprite as f32 + 15.99) / 256.0) as f64,
                (row as f32 / 256.0) as f64,
                ((row as f32 + 15.99) / 256.0) as f64,
            )
        };
        let upper = |sprite: i32, row: i32| -> (f64, f64, f64, f64) {
            (
                (sprite as f32 / 256.0) as f64,
                ((sprite as f32 + 15.99) / 256.0) as f64,
                ((row + 16) as f32 / 256.0) as f64,
                ((row as f32 + 15.99 + 16.0) / 256.0) as f64,
            )
        };
        let (mut d, mut d2, mut d3, mut d4): (f64, f64, f64, f64) = lower(n2, n3);
        let mut f2: f32 = 1.4;
        let fire: Rc<Block> = Block::fire();
        let v = |x: f64, y: f64, z: f64, u: f64, w: f64| tessellator.addVertexWithUV(x, y, z, u, w);
        let i = |value: i32| value as f64;
        if world.isBlockNormalCube(x, y - 1, z) || block_fire::canBlockCatchFire(&fire, &*world, x, y - 1, z)
        {
            let top: f64 = (y as f32 + f2) as f64;
            let mut d5: f64 = x as f64 + 0.5 + 0.2;
            let mut d6: f64 = x as f64 + 0.5 - 0.2;
            let mut d7: f64 = z as f64 + 0.5 + 0.2;
            let mut d8: f64 = z as f64 + 0.5 - 0.2;
            let mut d9: f64 = x as f64 + 0.5 - 0.3;
            let mut d10: f64 = x as f64 + 0.5 + 0.3;
            let mut d11: f64 = z as f64 + 0.5 - 0.3;
            let mut d12: f64 = z as f64 + 0.5 + 0.3;
            v(d9, top, i(z + 1), d2, d3);
            v(d5, i(y), i(z + 1), d2, d4);
            v(d5, i(y), i(z), d, d4);
            v(d9, top, i(z), d, d3);
            v(d10, top, i(z), d2, d3);
            v(d6, i(y), i(z), d2, d4);
            v(d6, i(y), i(z + 1), d, d4);
            v(d10, top, i(z + 1), d, d3);
            (d, d2, d3, d4) = upper(n2, n3);
            v(i(x + 1), top, d12, d2, d3);
            v(i(x + 1), i(y), d8, d2, d4);
            v(i(x), i(y), d8, d, d4);
            v(i(x), top, d12, d, d3);
            v(i(x), top, d11, d2, d3);
            v(i(x), i(y), d7, d2, d4);
            v(i(x + 1), i(y), d7, d, d4);
            v(i(x + 1), top, d11, d, d3);
            d5 = x as f64 + 0.5 - 0.5;
            d6 = x as f64 + 0.5 + 0.5;
            d7 = z as f64 + 0.5 - 0.5;
            d8 = z as f64 + 0.5 + 0.5;
            d9 = x as f64 + 0.5 - 0.4;
            d10 = x as f64 + 0.5 + 0.4;
            d11 = z as f64 + 0.5 - 0.4;
            d12 = z as f64 + 0.5 + 0.4;
            v(d9, top, i(z), d, d3);
            v(d5, i(y), i(z), d, d4);
            v(d5, i(y), i(z + 1), d2, d4);
            v(d9, top, i(z + 1), d2, d3);
            v(d10, top, i(z + 1), d, d3);
            v(d6, i(y), i(z + 1), d, d4);
            v(d6, i(y), i(z), d2, d4);
            v(d10, top, i(z), d2, d3);
            (d, d2, d3, d4) = lower(n2, n3);
            v(i(x), top, d12, d, d3);
            v(i(x), i(y), d8, d, d4);
            v(i(x + 1), i(y), d8, d2, d4);
            v(i(x + 1), top, d12, d2, d3);
            v(i(x + 1), top, d11, d, d3);
            v(i(x + 1), i(y), d7, d, d4);
            v(i(x), i(y), d7, d2, d4);
            v(i(x), top, d11, d2, d3);
            return true;
        }

        let f3: f32 = 0.2;
        let f4: f32 = 0.0625;
        if ((x + y + z) & 1) == 1
        {
            (d, d2, d3, d4) = upper(n2, n3);
        }
        if ((x / 2 + y / 2 + z / 2) & 1) == 1
        {
            std::mem::swap(&mut d, &mut d2);
        }
        let top: f64 = (y as f32 + f2 + f4) as f64;
        let bottom: f64 = (y as f32 + f4) as f64;
        if block_fire::canBlockCatchFire(&fire, &*world, x - 1, y, z)
        {
            let edge: f64 = (x as f32 + f3) as f64;
            v(edge, top, i(z + 1), d2, d3);
            v(i(x), bottom, i(z + 1), d2, d4);
            v(i(x), bottom, i(z), d, d4);
            v(edge, top, i(z), d, d3);
            v(edge, top, i(z), d, d3);
            v(i(x), bottom, i(z), d, d4);
            v(i(x), bottom, i(z + 1), d2, d4);
            v(edge, top, i(z + 1), d2, d3);
        }
        if block_fire::canBlockCatchFire(&fire, &*world, x + 1, y, z)
        {
            let edge: f64 = ((x + 1) as f32 - f3) as f64;
            v(edge, top, i(z), d, d3);
            v(i(x + 1), bottom, i(z), d, d4);
            v(i(x + 1), bottom, i(z + 1), d2, d4);
            v(edge, top, i(z + 1), d2, d3);
            v(edge, top, i(z + 1), d2, d3);
            v(i(x + 1), bottom, i(z + 1), d2, d4);
            v(i(x + 1), bottom, i(z), d, d4);
            v(edge, top, i(z), d, d3);
        }
        if block_fire::canBlockCatchFire(&fire, &*world, x, y, z - 1)
        {
            let edge: f64 = (z as f32 + f3) as f64;
            v(i(x), top, edge, d2, d3);
            v(i(x), bottom, i(z), d2, d4);
            v(i(x + 1), bottom, i(z), d, d4);
            v(i(x + 1), top, edge, d, d3);
            v(i(x + 1), top, edge, d, d3);
            v(i(x + 1), bottom, i(z), d, d4);
            v(i(x), bottom, i(z), d2, d4);
            v(i(x), top, edge, d2, d3);
        }
        if block_fire::canBlockCatchFire(&fire, &*world, x, y, z + 1)
        {
            let edge: f64 = ((z + 1) as f32 - f3) as f64;
            v(i(x + 1), top, edge, d, d3);
            v(i(x + 1), bottom, i(z + 1), d, d4);
            v(i(x), bottom, i(z + 1), d2, d4);
            v(i(x), top, edge, d2, d3);
            v(i(x), top, edge, d2, d3);
            v(i(x), bottom, i(z + 1), d2, d4);
            v(i(x + 1), bottom, i(z + 1), d, d4);
            v(i(x + 1), top, edge, d, d3);
        }
        if block_fire::canBlockCatchFire(&fire, &*world, x, y + 1, z)
        {
            let d13: f64 = x as f64 + 0.5 + 0.5;
            let d14: f64 = x as f64 + 0.5 - 0.5;
            let d15: f64 = z as f64 + 0.5 + 0.5;
            let d16: f64 = z as f64 + 0.5 - 0.5;
            let d17: f64 = x as f64 + 0.5 - 0.5;
            let d18: f64 = x as f64 + 0.5 + 0.5;
            let d19: f64 = z as f64 + 0.5 - 0.5;
            let d20: f64 = z as f64 + 0.5 + 0.5;
            (d, d2, d3, d4) = lower(n2, n3);
            f2 = -0.2;
            y += 1;
            let top: f64 = (y as f32 + f2) as f64;
            if ((x + y + z) & 1) == 0
            {
                v(d17, top, i(z), d2, d3);
                v(d13, i(y), i(z), d2, d4);
                v(d13, i(y), i(z + 1), d, d4);
                v(d17, top, i(z + 1), d, d3);
                (d, d2, d3, d4) = upper(n2, n3);
                v(d18, top, i(z + 1), d2, d3);
                v(d14, i(y), i(z + 1), d2, d4);
                v(d14, i(y), i(z), d, d4);
                v(d18, top, i(z), d, d3);
            }
            else
            {
                v(i(x), top, d20, d2, d3);
                v(i(x), i(y), d16, d2, d4);
                v(i(x + 1), i(y), d16, d, d4);
                v(i(x + 1), top, d20, d, d3);
                (d, d2, d3, d4) = upper(n2, n3);
                v(i(x + 1), top, d19, d2, d3);
                v(i(x + 1), i(y), d15, d2, d4);
                v(i(x), i(y), d15, d, d4);
                v(i(x), top, d19, d, d3);
            }
        }
        true
    }

    pub fn renderBlockRedstoneWire(&mut self, block: &Rc<Block>, x: i32, y: i32, z: i32) -> bool
    {
        let world: Rc<dyn IBlockAccess> = self.world();
        let tessellator: Rc<Tessellator> = Tessellator::instance();
        let i: i32 = world.getBlockMetadata(x, y, z);
        let mut j: i32 = block.getBlockTextureFromSideAndMetadata(1, i);
        if self.m_overrideBlockTexture >= 0
        {
            j = self.m_overrideBlockTexture;
        }
        let f: f32 = block.getBlockBrightness(&*world, x, y, z);
        let f1: f32 = i as f32 / 15.0;
        let mut f2: f32 = f1 * 0.6 + 0.4;
        if i == 0
        {
            f2 = 0.3;
        }
        let mut f3: f32 = f1 * f1 * 0.7 - 0.5;
        let mut f4: f32 = f1 * f1 * 0.6 - 0.7;
        if f3 < 0.0
        {
            f3 = 0.0;
        }
        if f4 < 0.0
        {
            f4 = 0.0;
        }
        tessellator.setColorOpaque_F(f * f2, f * f3, f * f4);
        let (k, l): (i32, i32) = spriteOrigin(j);
        let mut d0: f64 = (k as f32 / 256.0) as f64;
        let mut d1: f64 = ((k as f32 + 15.99) / 256.0) as f64;
        let mut d2: f64 = (l as f32 / 256.0) as f64;
        let mut d3: f64 = ((l as f32 + 15.99) / 256.0) as f64;
        let wire =
            |x: i32, y: i32, z: i32, side: i32| block_redstone_wire::isPowerProviderOrWire(&*world, x, y, z, side);
        let mut flag: bool = wire(x - 1, y, z, 1) || !world.isBlockNormalCube(x - 1, y, z) && wire(x - 1, y - 1, z, -1);
        let mut flag1: bool =
            wire(x + 1, y, z, 3) || !world.isBlockNormalCube(x + 1, y, z) && wire(x + 1, y - 1, z, -1);
        let mut flag2: bool =
            wire(x, y, z - 1, 2) || !world.isBlockNormalCube(x, y, z - 1) && wire(x, y - 1, z - 1, -1);
        let mut flag3: bool =
            wire(x, y, z + 1, 0) || !world.isBlockNormalCube(x, y, z + 1) && wire(x, y - 1, z + 1, -1);
        if !world.isBlockNormalCube(x, y + 1, z)
        {
            if world.isBlockNormalCube(x - 1, y, z) && wire(x - 1, y + 1, z, -1)
            {
                flag = true;
            }
            if world.isBlockNormalCube(x + 1, y, z) && wire(x + 1, y + 1, z, -1)
            {
                flag1 = true;
            }
            if world.isBlockNormalCube(x, y, z - 1) && wire(x, y + 1, z - 1, -1)
            {
                flag2 = true;
            }
            if world.isBlockNormalCube(x, y, z + 1) && wire(x, y + 1, z + 1, -1)
            {
                flag3 = true;
            }
        }
        let mut f5: f32 = x as f32;
        let mut f6: f32 = (x + 1) as f32;
        let mut f7: f32 = z as f32;
        let mut f8: f32 = (z + 1) as f32;
        let mut b0: i32 = 0;
        if (flag || flag1) && !flag2 && !flag3
        {
            b0 = 1;
        }
        if (flag2 || flag3) && !flag1 && !flag
        {
            b0 = 2;
        }
        if b0 != 0
        {
            d0 = ((k + 16) as f32 / 256.0) as f64;
            d1 = (((k + 16) as f32 + 15.99) / 256.0) as f64;
            d2 = (l as f32 / 256.0) as f64;
            d3 = ((l as f32 + 15.99) / 256.0) as f64;
        }
        let top: f64 = (y as f32 + 0.015625) as f64;
        let v = |x: f32, z: f32, u: f64, w: f64| tessellator.addVertexWithUV(x as f64, top, z as f64, u, w);
        if b0 == 0
        {
            if flag1 || flag2 || flag3 || flag
            {
                if !flag
                {
                    f5 += 0.3125;
                }
                if !flag
                {
                    d0 += 0.01953125;
                }
                if !flag1
                {
                    f6 -= 0.3125;
                }
                if !flag1
                {
                    d1 -= 0.01953125;
                }
                if !flag2
                {
                    f7 += 0.3125;
                }
                if !flag2
                {
                    d2 += 0.01953125;
                }
                if !flag3
                {
                    f8 -= 0.3125;
                }
                if !flag3
                {
                    d3 -= 0.01953125;
                }
            }
        }
        if b0 == 2
        {
            v(f6, f8, d1, d3);
            v(f6, f7, d0, d3);
            v(f5, f7, d0, d2);
            v(f5, f8, d1, d2);
            tessellator.setColorOpaque_F(f, f, f);
            v(f6, f8, d1, d3 + 0.0625);
            v(f6, f7, d0, d3 + 0.0625);
            v(f5, f7, d0, d2 + 0.0625);
            v(f5, f8, d1, d2 + 0.0625);
        }
        else
        {
            v(f6, f8, d1, d3);
            v(f6, f7, d1, d2);
            v(f5, f7, d0, d2);
            v(f5, f8, d0, d3);
            tessellator.setColorOpaque_F(f, f, f);
            v(f6, f8, d1, d3 + 0.0625);
            v(f6, f7, d1, d2 + 0.0625);
            v(f5, f7, d0, d2 + 0.0625);
            v(f5, f8, d0, d3 + 0.0625);
        }
        if world.isBlockNormalCube(x, y + 1, z)
        {
            return true;
        }
        d0 = ((k + 16) as f32 / 256.0) as f64;
        d1 = (((k + 16) as f32 + 15.99) / 256.0) as f64;
        d2 = (l as f32 / 256.0) as f64;
        d3 = ((l as f32 + 15.99) / 256.0) as f64;
        let wireId: i32 = Block::redstoneWire().m_blockID;
        let high: f64 = ((y + 1) as f32 + 0.021875) as f64;
        let low: f64 = y as f64;
        let w = |x: f64, y: f64, z: f64, u: f64, t: f64| tessellator.addVertexWithUV(x, y, z, u, t);
        let i = |value: i32| value as f64;
        if world.isBlockNormalCube(x - 1, y, z) && world.getBlockId(x - 1, y + 1, z) == wireId
        {
            let side: f64 = (x as f32 + 0.015625) as f64;
            for (pass, offset) in [(0, 0.0), (1, 0.0625)]
            {
                if pass == 0
                {
                    tessellator.setColorOpaque_F(f * f2, f * f3, f * f4);
                }
                else
                {
                    tessellator.setColorOpaque_F(f, f, f);
                }
                w(side, high, i(z + 1), d1, d2 + offset);
                w(side, low, i(z + 1), d0, d2 + offset);
                w(side, low, i(z), d0, d3 + offset);
                w(side, high, i(z), d1, d3 + offset);
            }
        }
        if world.isBlockNormalCube(x + 1, y, z) && world.getBlockId(x + 1, y + 1, z) == wireId
        {
            let side: f64 = ((x + 1) as f32 - 0.015625) as f64;
            for (pass, offset) in [(0, 0.0), (1, 0.0625)]
            {
                if pass == 0
                {
                    tessellator.setColorOpaque_F(f * f2, f * f3, f * f4);
                }
                else
                {
                    tessellator.setColorOpaque_F(f, f, f);
                }
                w(side, low, i(z + 1), d0, d3 + offset);
                w(side, high, i(z + 1), d1, d3 + offset);
                w(side, high, i(z), d1, d2 + offset);
                w(side, low, i(z), d0, d2 + offset);
            }
        }
        if world.isBlockNormalCube(x, y, z - 1) && world.getBlockId(x, y + 1, z - 1) == wireId
        {
            let side: f64 = (z as f32 + 0.015625) as f64;
            for (pass, offset) in [(0, 0.0), (1, 0.0625)]
            {
                if pass == 0
                {
                    tessellator.setColorOpaque_F(f * f2, f * f3, f * f4);
                }
                else
                {
                    tessellator.setColorOpaque_F(f, f, f);
                }
                w(i(x + 1), low, side, d0, d3 + offset);
                w(i(x + 1), high, side, d1, d3 + offset);
                w(i(x), high, side, d1, d2 + offset);
                w(i(x), low, side, d0, d2 + offset);
            }
        }
        if world.isBlockNormalCube(x, y, z + 1) && world.getBlockId(x, y + 1, z + 1) == wireId
        {
            let side: f64 = ((z + 1) as f32 - 0.015625) as f64;
            for (pass, offset) in [(0, 0.0), (1, 0.0625)]
            {
                if pass == 0
                {
                    tessellator.setColorOpaque_F(f * f2, f * f3, f * f4);
                }
                else
                {
                    tessellator.setColorOpaque_F(f, f, f);
                }
                w(i(x + 1), high, side, d1, d2 + offset);
                w(i(x + 1), low, side, d0, d2 + offset);
                w(i(x), low, side, d0, d3 + offset);
                w(i(x), high, side, d1, d3 + offset);
            }
        }
        true
    }
}
