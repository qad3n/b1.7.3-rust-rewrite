use crate::entity::Entity;
use crate::math::Vec3D;
use crate::pathfinding::PathPoint;
use std::cell::Cell;
use std::rc::Rc;

pub struct PathEntity
{
    m_points: Vec<Rc<PathPoint>>,
    pub m_pathLength: i32,
    m_pathIndex: Cell<i32>,
}

impl PathEntity
{
    pub fn new(nodes: Vec<Rc<PathPoint>>) -> Rc<PathEntity>
    {
        let pathLength: i32 = nodes.len() as i32;
        Rc::new(PathEntity { m_points: nodes, m_pathLength: pathLength, m_pathIndex: Cell::new(0) })
    }

    fn point(&self, index: i32) -> &Rc<PathPoint>
    {
        if !(0..self.m_points.len() as i32).contains(&index)
        {
            jvm::throwable::throw(&jvm::throwable::ARRAY_INDEX, &index.to_string());
        }
        &self.m_points[index as usize]
    }

    pub fn incrementPathIndex(&self) { self.m_pathIndex.set(self.m_pathIndex.get() + 1); }

    pub fn isFinished(&self) -> bool { self.m_pathIndex.get() >= self.m_points.len() as i32 }

    pub fn getLastNode(&self) -> Option<Rc<PathPoint>>
    {
        if self.m_pathLength > 0
        {
            return Some(Rc::clone(self.point(self.m_pathLength - 1)));
        }
        None
    }

    pub fn getPosition(&self, target: &Entity) -> Rc<Vec3D>
    {
        let point: &Rc<PathPoint> = self.point(self.m_pathIndex.get());
        let d: f64 = point.m_xCoord as f64 + ((target.m_width.get() + 1.0) as i32) as f64 * 0.5;
        let d2: f64 = point.m_yCoord as f64;
        let d3: f64 = point.m_zCoord as f64 + ((target.m_width.get() + 1.0) as i32) as f64 * 0.5;
        Vec3D::createVector(d, d2, d3)
    }
}
