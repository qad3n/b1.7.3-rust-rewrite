use crate::math::math_helper;
use jvm::array_list::JArrayList;
use std::cell::Cell;
use std::rc::Rc;

thread_local! {
    static VECTOR_LIST: JArrayList<Rc<Vec3D>> = JArrayList::new();
    static NEXT_VECTOR: Cell<i32> = const { Cell::new(0) };
}

pub struct Vec3D
{
    pub m_xCoord: Cell<f64>,
    pub m_yCoord: Cell<f64>,
    pub m_zCoord: Cell<f64>,
}

impl Vec3D
{
    pub fn createVectorHelper(x: f64, y: f64, z: f64) -> Rc<Self> { Rc::new(Self::new(x, y, z)) }

    pub fn clearPool()
    {
        VECTOR_LIST.with(JArrayList::clear);
        NEXT_VECTOR.with(|next| next.set(0));
    }

    pub fn initialize() { NEXT_VECTOR.with(|next| next.set(0)); }

    pub fn createVector(x: f64, y: f64, z: f64) -> Rc<Self>
    {
        VECTOR_LIST.with(|list| {
            let next: i32 = NEXT_VECTOR.with(Cell::get);
            if next >= list.size()
            {
                list.add(Self::createVectorHelper(0.0, 0.0, 0.0));
            }
            NEXT_VECTOR.with(|cell| cell.set(next + 1));
            list.get(next).setComponents(x, y, z)
        })
    }

    fn new(x: f64, y: f64, z: f64) -> Self
    {
        let x: f64 = if x == -0.0 { 0.0 } else { x };
        let y: f64 = if y == -0.0 { 0.0 } else { y };
        let z: f64 = if z == -0.0 { 0.0 } else { z };
        Self { m_xCoord: Cell::new(x), m_yCoord: Cell::new(y), m_zCoord: Cell::new(z) }
    }

    fn setComponents(self: Rc<Self>, x: f64, y: f64, z: f64) -> Rc<Self>
    {
        self.m_xCoord.set(x);
        self.m_yCoord.set(y);
        self.m_zCoord.set(z);
        self
    }

    pub fn x(&self) -> f64 { self.m_xCoord.get() }

    pub fn y(&self) -> f64 { self.m_yCoord.get() }

    pub fn z(&self) -> f64 { self.m_zCoord.get() }

    pub fn subtract(&self, vec: &Vec3D) -> Rc<Self>
    {
        Self::createVector(vec.x() - self.x(), vec.y() - self.y(), vec.z() - self.z())
    }

    pub fn normalize(&self) -> Rc<Self>
    {
        let d: f64 = math_helper::sqrt_double(self.x() * self.x() + self.y() * self.y() + self.z() * self.z()) as f64;
        if d < 1.0E-4
        {
            return Self::createVector(0.0, 0.0, 0.0);
        }
        Self::createVector(self.x() / d, self.y() / d, self.z() / d)
    }

    pub fn crossProduct(&self, vec: &Vec3D) -> Rc<Self>
    {
        Self::createVector(
            self.y() * vec.z() - self.z() * vec.y(),
            self.z() * vec.x() - self.x() * vec.z(),
            self.x() * vec.y() - self.y() * vec.x(),
        )
    }

    pub fn addVector(&self, x: f64, y: f64, z: f64) -> Rc<Self>
    {
        Self::createVector(self.x() + x, self.y() + y, self.z() + z)
    }

    pub fn distanceTo(&self, vec: &Vec3D) -> f64
    {
        let d: f64 = vec.x() - self.x();
        let d2: f64 = vec.y() - self.y();
        let d3: f64 = vec.z() - self.z();
        math_helper::sqrt_double(d * d + d2 * d2 + d3 * d3) as f64
    }

    pub fn squareDistanceTo(&self, vec: &Vec3D) -> f64
    {
        let d: f64 = vec.x() - self.x();
        let d2: f64 = vec.y() - self.y();
        let d3: f64 = vec.z() - self.z();
        d * d + d2 * d2 + d3 * d3
    }

    pub fn squareDistanceToPoint(&self, x: f64, y: f64, z: f64) -> f64
    {
        let d: f64 = x - self.x();
        let d2: f64 = y - self.y();
        let d3: f64 = z - self.z();
        d * d + d2 * d2 + d3 * d3
    }

    pub fn lengthVector(&self) -> f64
    {
        math_helper::sqrt_double(self.x() * self.x() + self.y() * self.y() + self.z() * self.z()) as f64
    }

    pub fn getIntermediateWithXValue(&self, vec: &Vec3D, x: f64) -> Option<Rc<Self>>
    {
        let d: f64 = vec.x() - self.x();
        let d2: f64 = vec.y() - self.y();
        let d3: f64 = vec.z() - self.z();
        if d * d < 1.0E-7f32 as f64
        {
            return None;
        }
        let d4: f64 = (x - self.x()) / d;
        if d4 < 0.0 || d4 > 1.0
        {
            return None;
        }
        Some(Self::createVector(self.x() + d * d4, self.y() + d2 * d4, self.z() + d3 * d4))
    }

    pub fn getIntermediateWithYValue(&self, vec: &Vec3D, y: f64) -> Option<Rc<Self>>
    {
        let d: f64 = vec.x() - self.x();
        let d2: f64 = vec.y() - self.y();
        let d3: f64 = vec.z() - self.z();
        if d2 * d2 < 1.0E-7f32 as f64
        {
            return None;
        }
        let d4: f64 = (y - self.y()) / d2;
        if d4 < 0.0 || d4 > 1.0
        {
            return None;
        }
        Some(Self::createVector(self.x() + d * d4, self.y() + d2 * d4, self.z() + d3 * d4))
    }

    pub fn getIntermediateWithZValue(&self, vec: &Vec3D, z: f64) -> Option<Rc<Self>>
    {
        let d: f64 = vec.x() - self.x();
        let d2: f64 = vec.y() - self.y();
        let d3: f64 = vec.z() - self.z();
        if d3 * d3 < 1.0E-7f32 as f64
        {
            return None;
        }
        let d4: f64 = (z - self.z()) / d3;
        if d4 < 0.0 || d4 > 1.0
        {
            return None;
        }
        Some(Self::createVector(self.x() + d * d4, self.y() + d2 * d4, self.z() + d3 * d4))
    }

    pub fn toString(&self) -> String
    {
        use jvm::floating_decimal::doubleToString as d;
        format!("({}, {}, {})", d(self.x()), d(self.y()), d(self.z()))
    }

    pub fn rotateAroundX(&self, angle: f32)
    {
        let f: f32 = math_helper::cos(angle);
        let f2: f32 = math_helper::sin(angle);
        let d: f64 = self.x();
        let d2: f64 = self.y() * f as f64 + self.z() * f2 as f64;
        let d3: f64 = self.z() * f as f64 - self.y() * f2 as f64;
        self.m_xCoord.set(d);
        self.m_yCoord.set(d2);
        self.m_zCoord.set(d3);
    }

    pub fn rotateAroundY(&self, angle: f32)
    {
        let f: f32 = math_helper::cos(angle);
        let f2: f32 = math_helper::sin(angle);
        let d: f64 = self.x() * f as f64 + self.z() * f2 as f64;
        let d2: f64 = self.y();
        let d3: f64 = self.z() * f as f64 - self.x() * f2 as f64;
        self.m_xCoord.set(d);
        self.m_yCoord.set(d2);
        self.m_zCoord.set(d3);
    }
}
