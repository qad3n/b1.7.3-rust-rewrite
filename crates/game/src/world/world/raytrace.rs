use crate::block::Block;
use crate::math::math_helper::floor_double;
use crate::math::{MovingObjectPosition, Vec3D};
use crate::world::World;
use std::rc::Rc;

impl World
{
    pub fn rayTraceBlocks(self: &Rc<Self>, from: &Vec3D, to: &Vec3D) -> Option<MovingObjectPosition>
    {
        self.rayTrace(from, to, false, false)
    }

    pub fn rayTraceBlocks_do(
        self: &Rc<Self>,
        from: &Vec3D,
        to: &Vec3D,
        allowLiquids: bool,
    ) -> Option<MovingObjectPosition>
    {
        self.rayTrace(from, to, allowLiquids, false)
    }

    fn hitBlock(
        self: &Rc<Self>,
        x: i32,
        y: i32,
        z: i32,
        from: &Vec3D,
        to: &Vec3D,
        allowLiquids: bool,
        ignore: bool,
    ) -> Option<MovingObjectPosition>
    {
        let id: i32 = self.getBlockId(x, y, z);
        let metadata: i32 = self.getBlockMetadata(x, y, z);
        let block: Option<Rc<Block>> = Block::byId(id);
        let passes: bool = match &block
        {
            Some(block) if ignore => block.getCollisionBoundingBoxFromPool(self, x, y, z).is_some(),
            _ => true,
        };
        if !(passes && id > 0)
        {
            return None;
        }
        let block: Rc<Block> = block.unwrap_or_else(|| jvm::throwable::npe());
        if !block.canCollideCheck(metadata, allowLiquids)
        {
            return None;
        }
        block.collisionRayTrace(self, x, y, z, from, to)
    }

    pub fn rayTrace(
        self: &Rc<Self>,
        from: &Vec3D,
        to: &Vec3D,
        allowLiquids: bool,
        ignoreBlocksWithoutCollision: bool,
    ) -> Option<MovingObjectPosition>
    {
        if from.x().is_nan() || from.y().is_nan() || from.z().is_nan()
        {
            return None;
        }
        if to.x().is_nan() || to.y().is_nan() || to.z().is_nan()
        {
            return None;
        }

        let i: i32 = floor_double(to.x());
        let j: i32 = floor_double(to.y());
        let k: i32 = floor_double(to.z());
        let mut l: i32 = floor_double(from.x());
        let mut i1: i32 = floor_double(from.y());
        let mut j1: i32 = floor_double(from.z());
        if let Some(hit) = self.hitBlock(l, i1, j1, from, to, allowLiquids, ignoreBlocksWithoutCollision)
        {
            return Some(hit);
        }

        let mut k1: i32 = 200;
        loop
        {
            let previous: i32 = k1;
            k1 -= 1;
            if previous < 0
            {
                return None;
            }

            if from.x().is_nan() || from.y().is_nan() || from.z().is_nan()
            {
                return None;
            }
            if l == i && i1 == j && j1 == k
            {
                return None;
            }

            let mut flag: bool = true;
            let mut flag1: bool = true;
            let mut flag2: bool = true;
            let mut d0: f64 = 999.0;
            let mut d1: f64 = 999.0;
            let mut d2: f64 = 999.0;
            if i > l
            {
                d0 = l as f64 + 1.0;
            }
            else if i < l
            {
                d0 = l as f64 + 0.0;
            }
            else
            {
                flag = false;
            }
            if j > i1
            {
                d1 = i1 as f64 + 1.0;
            }
            else if j < i1
            {
                d1 = i1 as f64 + 0.0;
            }
            else
            {
                flag1 = false;
            }
            if k > j1
            {
                d2 = j1 as f64 + 1.0;
            }
            else if k < j1
            {
                d2 = j1 as f64 + 0.0;
            }
            else
            {
                flag2 = false;
            }

            let mut d3: f64 = 999.0;
            let mut d4: f64 = 999.0;
            let mut d5: f64 = 999.0;
            let d6: f64 = to.x() - from.x();
            let d7: f64 = to.y() - from.y();
            let d8: f64 = to.z() - from.z();
            if flag
            {
                d3 = (d0 - from.x()) / d6;
            }
            if flag1
            {
                d4 = (d1 - from.y()) / d7;
            }
            if flag2
            {
                d5 = (d2 - from.z()) / d8;
            }

            let b0: i8;
            if d3 < d4 && d3 < d5
            {
                b0 = if i > l { 4 } else { 5 };
                from.m_xCoord.set(d0);
                from.m_yCoord.set(from.y() + d7 * d3);
                from.m_zCoord.set(from.z() + d8 * d3);
            }
            else if d4 < d5
            {
                b0 = if j > i1 { 0 } else { 1 };
                from.m_xCoord.set(from.x() + d6 * d4);
                from.m_yCoord.set(d1);
                from.m_zCoord.set(from.z() + d8 * d4);
            }
            else
            {
                b0 = if k > j1 { 2 } else { 3 };
                from.m_xCoord.set(from.x() + d6 * d5);
                from.m_yCoord.set(from.y() + d7 * d5);
                from.m_zCoord.set(d2);
            }

            let vec3d: Rc<Vec3D> = Vec3D::createVector(from.x(), from.y(), from.z());
            vec3d.m_xCoord.set(floor_double(from.x()) as f64);
            l = vec3d.x() as i32;
            if b0 == 5
            {
                l -= 1;
                vec3d.m_xCoord.set(vec3d.x() + 1.0);
            }
            vec3d.m_yCoord.set(floor_double(from.y()) as f64);
            i1 = vec3d.y() as i32;
            if b0 == 1
            {
                i1 -= 1;
                vec3d.m_yCoord.set(vec3d.y() + 1.0);
            }
            vec3d.m_zCoord.set(floor_double(from.z()) as f64);
            j1 = vec3d.z() as i32;
            if b0 == 3
            {
                j1 -= 1;
                vec3d.m_zCoord.set(vec3d.z() + 1.0);
            }

            if let Some(hit) = self.hitBlock(l, i1, j1, from, to, allowLiquids, ignoreBlocksWithoutCollision)
            {
                return Some(hit);
            }
        }
    }
}
