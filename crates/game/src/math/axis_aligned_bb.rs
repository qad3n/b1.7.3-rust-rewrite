use crate::math::{MovingObjectPosition, Vec3D};
use jvm::array_list::JArrayList;
use std::cell::Cell;
use std::rc::Rc;

thread_local! {
    static BOUNDING_BOXES: JArrayList<Rc<AxisAlignedBB>> = JArrayList::new();
    static NUM_BOUNDING_BOXES_IN_USE: Cell<i32> = const { Cell::new(0) };
}

pub struct AxisAlignedBB
{
    pub m_minX: Cell<f64>,
    pub m_minY: Cell<f64>,
    pub m_minZ: Cell<f64>,
    pub m_maxX: Cell<f64>,
    pub m_maxY: Cell<f64>,
    pub m_maxZ: Cell<f64>,
}

fn sameVec(a: &Option<Rc<Vec3D>>, b: &Option<Rc<Vec3D>>) -> bool
{
    match (a, b)
    {
        (Some(a), Some(b)) => Rc::ptr_eq(a, b),
        (None, None) => true,
        _ => false,
    }
}

impl AxisAlignedBB
{
    pub fn getBoundingBox(minX: f64, minY: f64, minZ: f64, maxX: f64, maxY: f64, maxZ: f64) -> Rc<Self>
    {
        Rc::new(Self::new(minX, minY, minZ, maxX, maxY, maxZ))
    }

    pub fn clearCache()
    {
        BOUNDING_BOXES.with(JArrayList::clear);
        NUM_BOUNDING_BOXES_IN_USE.with(|count| count.set(0));
    }

    pub fn clearBoundingBoxPool() { NUM_BOUNDING_BOXES_IN_USE.with(|count| count.set(0)); }

    pub fn getBoundingBoxFromPool(minX: f64, minY: f64, minZ: f64, maxX: f64, maxY: f64, maxZ: f64) -> Rc<Self>
    {
        BOUNDING_BOXES.with(|boxes| {
            let used: i32 = NUM_BOUNDING_BOXES_IN_USE.with(Cell::get);
            if used >= boxes.size()
            {
                boxes.add(Self::getBoundingBox(0.0, 0.0, 0.0, 0.0, 0.0, 0.0));
            }
            NUM_BOUNDING_BOXES_IN_USE.with(|count| count.set(used + 1));
            let bb: Rc<Self> = boxes.get(used);
            bb.setBounds(minX, minY, minZ, maxX, maxY, maxZ);
            bb
        })
    }

    fn new(minX: f64, minY: f64, minZ: f64, maxX: f64, maxY: f64, maxZ: f64) -> Self
    {
        Self {
            m_minX: Cell::new(minX),
            m_minY: Cell::new(minY),
            m_minZ: Cell::new(minZ),
            m_maxX: Cell::new(maxX),
            m_maxY: Cell::new(maxY),
            m_maxZ: Cell::new(maxZ),
        }
    }

    pub fn setBounds(&self, minX: f64, minY: f64, minZ: f64, maxX: f64, maxY: f64, maxZ: f64)
    {
        self.m_minX.set(minX);
        self.m_minY.set(minY);
        self.m_minZ.set(minZ);
        self.m_maxX.set(maxX);
        self.m_maxY.set(maxY);
        self.m_maxZ.set(maxZ);
    }

    pub fn minX(&self) -> f64 { self.m_minX.get() }

    pub fn minY(&self) -> f64 { self.m_minY.get() }

    pub fn minZ(&self) -> f64 { self.m_minZ.get() }

    pub fn maxX(&self) -> f64 { self.m_maxX.get() }

    pub fn maxY(&self) -> f64 { self.m_maxY.get() }

    pub fn maxZ(&self) -> f64 { self.m_maxZ.get() }

    pub fn addCoord(&self, dx: f64, dy: f64, dz: f64) -> Rc<Self>
    {
        let mut d: f64 = self.minX();
        let mut d2: f64 = self.minY();
        let mut d3: f64 = self.minZ();
        let mut d4: f64 = self.maxX();
        let mut d5: f64 = self.maxY();
        let mut d6: f64 = self.maxZ();
        if dx < 0.0
        {
            d += dx;
        }
        if dx > 0.0
        {
            d4 += dx;
        }
        if dy < 0.0
        {
            d2 += dy;
        }
        if dy > 0.0
        {
            d5 += dy;
        }
        if dz < 0.0
        {
            d3 += dz;
        }
        if dz > 0.0
        {
            d6 += dz;
        }
        Self::getBoundingBoxFromPool(d, d2, d3, d4, d5, d6)
    }

    pub fn expand(&self, dx: f64, dy: f64, dz: f64) -> Rc<Self>
    {
        Self::getBoundingBoxFromPool(
            self.minX() - dx,
            self.minY() - dy,
            self.minZ() - dz,
            self.maxX() + dx,
            self.maxY() + dy,
            self.maxZ() + dz,
        )
    }

    pub fn getOffsetBoundingBox(&self, dx: f64, dy: f64, dz: f64) -> Rc<Self>
    {
        Self::getBoundingBoxFromPool(
            self.minX() + dx,
            self.minY() + dy,
            self.minZ() + dz,
            self.maxX() + dx,
            self.maxY() + dy,
            self.maxZ() + dz,
        )
    }

    pub fn calculateXOffset(&self, bb: &AxisAlignedBB, limit: f64) -> f64
    {
        let mut limit: f64 = limit;
        if bb.maxY() <= self.minY() || bb.minY() >= self.maxY()
        {
            return limit;
        }
        if bb.maxZ() <= self.minZ() || bb.minZ() >= self.maxZ()
        {
            return limit;
        }
        if limit > 0.0 && bb.maxX() <= self.minX()
        {
            let n: f64 = self.minX() - bb.maxX();
            if n < limit
            {
                limit = n;
            }
        }
        if limit < 0.0 && bb.minX() >= self.maxX()
        {
            let n2: f64 = self.maxX() - bb.minX();
            if n2 > limit
            {
                limit = n2;
            }
        }
        limit
    }

    pub fn calculateYOffset(&self, bb: &AxisAlignedBB, limit: f64) -> f64
    {
        let mut limit: f64 = limit;
        if bb.maxX() <= self.minX() || bb.minX() >= self.maxX()
        {
            return limit;
        }
        if bb.maxZ() <= self.minZ() || bb.minZ() >= self.maxZ()
        {
            return limit;
        }
        if limit > 0.0 && bb.maxY() <= self.minY()
        {
            let n: f64 = self.minY() - bb.maxY();
            if n < limit
            {
                limit = n;
            }
        }
        if limit < 0.0 && bb.minY() >= self.maxY()
        {
            let n2: f64 = self.maxY() - bb.minY();
            if n2 > limit
            {
                limit = n2;
            }
        }
        limit
    }

    pub fn calculateZOffset(&self, bb: &AxisAlignedBB, limit: f64) -> f64
    {
        let mut limit: f64 = limit;
        if bb.maxX() <= self.minX() || bb.minX() >= self.maxX()
        {
            return limit;
        }
        if bb.maxY() <= self.minY() || bb.minY() >= self.maxY()
        {
            return limit;
        }
        if limit > 0.0 && bb.maxZ() <= self.minZ()
        {
            let n: f64 = self.minZ() - bb.maxZ();
            if n < limit
            {
                limit = n;
            }
        }
        if limit < 0.0 && bb.minZ() >= self.maxZ()
        {
            let n2: f64 = self.maxZ() - bb.minZ();
            if n2 > limit
            {
                limit = n2;
            }
        }
        limit
    }

    pub fn intersectsWith(&self, bb: &AxisAlignedBB) -> bool
    {
        if bb.maxX() <= self.minX() || bb.minX() >= self.maxX()
        {
            return false;
        }
        if bb.maxY() <= self.minY() || bb.minY() >= self.maxY()
        {
            return false;
        }
        !(bb.maxZ() <= self.minZ()) && !(bb.minZ() >= self.maxZ())
    }

    pub fn offset(self: &Rc<Self>, dx: f64, dy: f64, dz: f64) -> Rc<Self>
    {
        self.m_minX.set(self.minX() + dx);
        self.m_minY.set(self.minY() + dy);
        self.m_minZ.set(self.minZ() + dz);
        self.m_maxX.set(self.maxX() + dx);
        self.m_maxY.set(self.maxY() + dy);
        self.m_maxZ.set(self.maxZ() + dz);
        Rc::clone(self)
    }

    pub fn isVecInside(&self, vec: &Vec3D) -> bool
    {
        if vec.x() <= self.minX() || vec.x() >= self.maxX()
        {
            return false;
        }
        if vec.y() <= self.minY() || vec.y() >= self.maxY()
        {
            return false;
        }
        !(vec.z() <= self.minZ()) && !(vec.z() >= self.maxZ())
    }

    pub fn getAverageEdgeLength(&self) -> f64
    {
        let d: f64 = self.maxX() - self.minX();
        let d2: f64 = self.maxY() - self.minY();
        let d3: f64 = self.maxZ() - self.minZ();
        (d + d2 + d3) / 3.0
    }

    pub fn contract(&self, dx: f64, dy: f64, dz: f64) -> Rc<Self>
    {
        Self::getBoundingBoxFromPool(
            self.minX() + dx,
            self.minY() + dy,
            self.minZ() + dz,
            self.maxX() - dx,
            self.maxY() - dy,
            self.maxZ() - dz,
        )
    }

    pub fn copy(&self) -> Rc<Self>
    {
        Self::getBoundingBoxFromPool(self.minX(), self.minY(), self.minZ(), self.maxX(), self.maxY(), self.maxZ())
    }

    pub fn clip(&self, from: &Vec3D, to: &Vec3D) -> Option<MovingObjectPosition>
    {
        let mut vec3D: Option<Rc<Vec3D>> = from.getIntermediateWithXValue(to, self.minX());
        let mut vec3D2: Option<Rc<Vec3D>> = from.getIntermediateWithXValue(to, self.maxX());
        let mut vec3D3: Option<Rc<Vec3D>> = from.getIntermediateWithYValue(to, self.minY());
        let mut vec3D4: Option<Rc<Vec3D>> = from.getIntermediateWithYValue(to, self.maxY());
        let mut vec3D5: Option<Rc<Vec3D>> = from.getIntermediateWithZValue(to, self.minZ());
        let mut vec3D6: Option<Rc<Vec3D>> = from.getIntermediateWithZValue(to, self.maxZ());
        if !self.isVecInYZ(vec3D.as_deref())
        {
            vec3D = None;
        }
        if !self.isVecInYZ(vec3D2.as_deref())
        {
            vec3D2 = None;
        }
        if !self.isVecInXZ(vec3D3.as_deref())
        {
            vec3D3 = None;
        }
        if !self.isVecInXZ(vec3D4.as_deref())
        {
            vec3D4 = None;
        }
        if !self.isVecInXY(vec3D5.as_deref())
        {
            vec3D5 = None;
        }
        if !self.isVecInXY(vec3D6.as_deref())
        {
            vec3D6 = None;
        }

        let mut vec3D7: Option<Rc<Vec3D>> = None;
        for candidate in [&vec3D, &vec3D2, &vec3D3, &vec3D4, &vec3D5, &vec3D6]
        {
            if let Some(candidate) = candidate
            {
                let closer: bool = match &vec3D7
                {
                    None => true,
                    Some(best) => from.squareDistanceTo(candidate) < from.squareDistanceTo(best),
                };
                if closer
                {
                    vec3D7 = Some(Rc::clone(candidate));
                }
            }
        }

        let hit: Rc<Vec3D> = vec3D7.clone()?;
        let mut n: i32 = -1;
        if sameVec(&vec3D7, &vec3D)
        {
            n = 4;
        }
        if sameVec(&vec3D7, &vec3D2)
        {
            n = 5;
        }
        if sameVec(&vec3D7, &vec3D3)
        {
            n = 0;
        }
        if sameVec(&vec3D7, &vec3D4)
        {
            n = 1;
        }
        if sameVec(&vec3D7, &vec3D5)
        {
            n = 2;
        }
        if sameVec(&vec3D7, &vec3D6)
        {
            n = 3;
        }
        Some(MovingObjectPosition::newTile(0, 0, 0, n, &hit))
    }

    fn isVecInYZ(&self, vec: Option<&Vec3D>) -> bool
    {
        let Some(vec) = vec
        else
        {
            return false;
        };
        vec.y() >= self.minY() && vec.y() <= self.maxY() && vec.z() >= self.minZ() && vec.z() <= self.maxZ()
    }

    fn isVecInXZ(&self, vec: Option<&Vec3D>) -> bool
    {
        let Some(vec) = vec
        else
        {
            return false;
        };
        vec.x() >= self.minX() && vec.x() <= self.maxX() && vec.z() >= self.minZ() && vec.z() <= self.maxZ()
    }

    fn isVecInXY(&self, vec: Option<&Vec3D>) -> bool
    {
        let Some(vec) = vec
        else
        {
            return false;
        };
        vec.x() >= self.minX() && vec.x() <= self.maxX() && vec.y() >= self.minY() && vec.y() <= self.maxY()
    }

    pub fn setBB(&self, bb: &AxisAlignedBB)
    {
        self.setBounds(bb.minX(), bb.minY(), bb.minZ(), bb.maxX(), bb.maxY(), bb.maxZ());
    }

    pub fn toString(&self) -> String
    {
        use jvm::floating_decimal::doubleToString as d;
        format!(
            "box[{}, {}, {} -> {}, {}, {}]",
            d(self.minX()),
            d(self.minY()),
            d(self.minZ()),
            d(self.maxX()),
            d(self.maxY()),
            d(self.maxZ())
        )
    }
}
