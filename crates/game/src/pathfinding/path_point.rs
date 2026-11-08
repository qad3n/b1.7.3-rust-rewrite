use crate::math::math_helper::sqrt_float;
use jvm::field::Field;
use std::cell::Cell;
use std::rc::Rc;

pub struct PathPoint
{
    pub m_xCoord: i32,
    pub m_yCoord: i32,
    pub m_zCoord: i32,
    m_hash: i32,
    pub(super) m_index: Cell<i32>,
    pub(super) m_totalPathDistance: Cell<f32>,
    pub(super) m_distanceToNext: Cell<f32>,
    pub(super) m_distanceToTarget: Cell<f32>,
    pub(super) m_previous: Field<Option<Rc<PathPoint>>>,
    pub m_isFirst: Cell<bool>,
}

impl PathPoint
{
    pub fn new(x: i32, y: i32, z: i32) -> Rc<PathPoint>
    {
        Rc::new(PathPoint {
            m_xCoord: x,
            m_yCoord: y,
            m_zCoord: z,
            m_hash: PathPoint::hash(x, y, z),
            m_index: Cell::new(-1),
            m_totalPathDistance: Cell::new(0.0),
            m_distanceToNext: Cell::new(0.0),
            m_distanceToTarget: Cell::new(0.0),
            m_previous: Field::new(None),
            m_isFirst: Cell::new(false),
        })
    }

    pub fn hash(x: i32, y: i32, z: i32) -> i32
    {
        y & 0xFF
            | (x & 0x7FFF) << 8
            | (z & 0x7FFF) << 24
            | if x < 0 { i32::MIN } else { 0 }
            | if z < 0 { 32768 } else { 0 }
    }

    pub fn distanceTo(&self, other: &PathPoint) -> f32
    {
        let f: f32 = other.m_xCoord.wrapping_sub(self.m_xCoord) as f32;
        let f2: f32 = other.m_yCoord.wrapping_sub(self.m_yCoord) as f32;
        let f3: f32 = other.m_zCoord.wrapping_sub(self.m_zCoord) as f32;
        sqrt_float(f * f + f2 * f2 + f3 * f3)
    }

    pub fn equals(&self, other: &PathPoint) -> bool
    {
        self.m_hash == other.m_hash
            && self.m_xCoord == other.m_xCoord
            && self.m_yCoord == other.m_yCoord
            && self.m_zCoord == other.m_zCoord
    }

    pub fn hashCode(&self) -> i32 { self.m_hash }

    pub fn isAssigned(&self) -> bool { self.m_index.get() >= 0 }

    pub fn toString(&self) -> String { format!("{}, {}, {}", self.m_xCoord, self.m_yCoord, self.m_zCoord) }
}
