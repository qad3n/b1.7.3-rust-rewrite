use super::RenderBlocks;
use crate::render::Tessellator;
use game::block::Block;
use game::world::IBlockAccess;
use std::rc::Rc;

impl RenderBlocks
{
    pub fn renderBlockFence(&mut self, block: &Rc<Block>, x: i32, y: i32, z: i32) -> bool
    {
        let world: Rc<dyn IBlockAccess> = self.world();
        let mut f: f32 = 0.375;
        let mut f1: f32 = 0.625;
        block.setBlockBounds(f, 0.0, f, f1, 1.0, f1);
        self.renderStandardBlock(block, x, y, z);
        let mut flag: bool = true;
        let id: i32 = block.m_blockID;
        let mut flag1: bool = world.getBlockId(x - 1, y, z) == id || world.getBlockId(x + 1, y, z) == id;
        let flag2: bool = world.getBlockId(x, y, z - 1) == id || world.getBlockId(x, y, z + 1) == id;
        let flag3: bool = world.getBlockId(x - 1, y, z) == id;
        let flag4: bool = world.getBlockId(x + 1, y, z) == id;
        let flag5: bool = world.getBlockId(x, y, z - 1) == id;
        let flag6: bool = world.getBlockId(x, y, z + 1) == id;
        if !flag1 && !flag2
        {
            flag1 = true;
        }
        f = 0.4375;
        f1 = 0.5625;
        let f4: f32 = if flag3 { 0.0 } else { f };
        let f5: f32 = if flag4 { 1.0 } else { f1 };
        let f6: f32 = if flag5 { 0.0 } else { f };
        let f7: f32 = if flag6 { 1.0 } else { f1 };
        for (f2, f3) in [(0.75, 0.9375), (0.375, 0.5625)]
        {
            if flag1
            {
                block.setBlockBounds(f4, f2, f, f5, f3, f1);
                self.renderStandardBlock(block, x, y, z);
                flag = true;
            }
            if flag2
            {
                block.setBlockBounds(f, f2, f6, f1, f3, f7);
                self.renderStandardBlock(block, x, y, z);
                flag = true;
            }
        }
        block.setBlockBounds(0.0, 0.0, 0.0, 1.0, 1.0, 1.0);
        flag
    }

    pub fn renderBlockStairs(&mut self, block: &Rc<Block>, x: i32, y: i32, z: i32) -> bool
    {
        let n: i32 = self.world().getBlockMetadata(x, y, z);
        let halves: Option<[[f32; 6]; 2]> = match n
        {
            0 => Some([[0.0, 0.0, 0.0, 0.5, 0.5, 1.0], [0.5, 0.0, 0.0, 1.0, 1.0, 1.0]]),
            1 => Some([[0.0, 0.0, 0.0, 0.5, 1.0, 1.0], [0.5, 0.0, 0.0, 1.0, 0.5, 1.0]]),
            2 => Some([[0.0, 0.0, 0.0, 1.0, 0.5, 0.5], [0.0, 0.0, 0.5, 1.0, 1.0, 1.0]]),
            3 => Some([[0.0, 0.0, 0.0, 1.0, 1.0, 0.5], [0.0, 0.0, 0.5, 1.0, 0.5, 1.0]]),
            _ => None,
        };
        let bl: bool = halves.is_some();
        for bounds in halves.into_iter().flatten()
        {
            block.setBlockBounds(bounds[0], bounds[1], bounds[2], bounds[3], bounds[4], bounds[5]);
            self.renderStandardBlock(block, x, y, z);
        }
        block.setBlockBounds(0.0, 0.0, 0.0, 1.0, 1.0, 1.0);
        bl
    }

    pub fn renderBlockDoor(&mut self, block: &Rc<Block>, x: i32, y: i32, z: i32) -> bool
    {
        let world: Rc<dyn IBlockAccess> = self.world();
        let tessellator: Rc<Tessellator> = Tessellator::instance();
        let f: f32 = 0.5;
        let f2: f32 = 1.0;
        let f3: f32 = 0.8;
        let f4: f32 = 0.6;
        let f5: f32 = block.getBlockBrightness(&*world, x, y, z);
        let lit: bool = Block::lightValue(block.m_blockID) > 0;
        let shade = |brightness: f32, ownLight: bool| -> f32 {
            let f6: f32 = if ownLight { f5 } else { brightness };
            if lit { 1.0 } else { f6 }
        };
        let (dx, dy, dz): (f64, f64, f64) = (x as f64, y as f64, z as f64);
        let mut f6: f32 = shade(block.getBlockBrightness(&*world, x, y - 1, z), block.m_minY.get() > 0.0);
        tessellator.setColorOpaque_F(f * f6, f * f6, f * f6);
        self.renderBottomFace(block, dx, dy, dz, block.getBlockTexture(&*world, x, y, z, 0));
        f6 = shade(block.getBlockBrightness(&*world, x, y + 1, z), block.m_maxY.get() < 1.0);
        tessellator.setColorOpaque_F(f2 * f6, f2 * f6, f2 * f6);
        self.renderTopFace(block, dx, dy, dz, block.getBlockTexture(&*world, x, y, z, 1));
        let sides: [(i32, i32, i32, bool, f32, i32); 4] = [
            (x, y, z - 1, block.m_minZ.get() > 0.0, f3, 2),
            (x, y, z + 1, block.m_maxZ.get() < 1.0, f3, 3),
            (x - 1, y, z, block.m_minX.get() > 0.0, f4, 4),
            (x + 1, y, z, block.m_maxX.get() < 1.0, f4, 5),
        ];
        for (nx, ny, nz, inset, tone, face) in sides
        {
            f6 = shade(block.getBlockBrightness(&*world, nx, ny, nz), inset);
            tessellator.setColorOpaque_F(tone * f6, tone * f6, tone * f6);
            let mut n: i32 = block.getBlockTexture(&*world, x, y, z, face);
            if n < 0
            {
                self.m_flipTexture = true;
                n = -n;
            }
            match face
            {
                2 => self.renderEastFace(block, dx, dy, dz, n),
                3 => self.renderWestFace(block, dx, dy, dz, n),
                4 => self.renderNorthFace(block, dx, dy, dz, n),
                _ => self.renderSouthFace(block, dx, dy, dz, n),
            }
            self.m_flipTexture = false;
        }
        true
    }
}
