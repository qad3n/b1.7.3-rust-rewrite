use crate::pathfinding::PathPoint;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

pub struct Path
{
    m_pathPoints: RefCell<Vec<Option<Rc<PathPoint>>>>,
    m_count: Cell<i32>,
}

impl Path
{
    pub fn new() -> Path { Path { m_pathPoints: RefCell::new(vec![None; 1024]), m_count: Cell::new(0) } }

    fn point(&self, index: i32) -> Rc<PathPoint>
    {
        self.m_pathPoints.borrow()[index as usize].clone().unwrap_or_else(|| jvm::throwable::npe())
    }

    fn store(&self, index: i32, point: Option<Rc<PathPoint>>)
    {
        self.m_pathPoints.borrow_mut()[index as usize] = point;
    }

    pub fn addPoint(&self, node: &Rc<PathPoint>) -> Rc<PathPoint>
    {
        if node.m_index.get() >= 0
        {
            jvm::throwable::throw(&jvm::throwable::ILLEGAL_STATE, "OW KNOWS!");
        }

        let count: i32 = self.m_count.get();
        if count as usize == self.m_pathPoints.borrow().len()
        {
            self.m_pathPoints.borrow_mut().resize((count << 1) as usize, None);
        }

        self.store(count, Some(Rc::clone(node)));
        node.m_index.set(count);
        self.m_count.set(count + 1);
        self.sortBack(count);
        Rc::clone(node)
    }

    pub fn clearPath(&self) { self.m_count.set(0); }

    pub fn dequeue(&self) -> Rc<PathPoint>
    {
        let pathPoint: Rc<PathPoint> = self.point(0);
        let count: i32 = self.m_count.get() - 1;
        self.m_count.set(count);
        let last: Option<Rc<PathPoint>> = self.m_pathPoints.borrow()[count as usize].clone();
        self.store(0, last);
        self.store(count, None);
        if count > 0
        {
            self.sortForward(0);
        }

        pathPoint.m_index.set(-1);
        pathPoint
    }

    pub fn changeDistance(&self, node: &PathPoint, weight: f32)
    {
        let f: f32 = node.m_distanceToTarget.get();
        node.m_distanceToTarget.set(weight);
        if weight < f
        {
            self.sortBack(node.m_index.get());
        }
        else
        {
            self.sortForward(node.m_index.get());
        }
    }

    fn sortBack(&self, mut index: i32)
    {
        let pathPoint: Rc<PathPoint> = self.point(index);
        let f: f32 = pathPoint.m_distanceToTarget.get();
        while index > 0
        {
            let n: i32 = (index - 1) >> 1;
            let pathPoint2: Rc<PathPoint> = self.point(n);
            if !(f < pathPoint2.m_distanceToTarget.get())
            {
                break;
            }
            pathPoint2.m_index.set(index);
            self.store(index, Some(pathPoint2));
            index = n;
        }

        pathPoint.m_index.set(index);
        self.store(index, Some(pathPoint));
    }

    fn sortForward(&self, mut index: i32)
    {
        let pathPoint: Rc<PathPoint> = self.point(index);
        let f: f32 = pathPoint.m_distanceToTarget.get();
        loop
        {
            let n: i32 = 1 + (index << 1);
            let n2: i32 = n + 1;
            if n >= self.m_count.get()
            {
                break;
            }
            let pathPoint3: Rc<PathPoint> = self.point(n);
            let f3: f32 = pathPoint3.m_distanceToTarget.get();
            let (pathPoint2, f2): (Option<Rc<PathPoint>>, f32) = if n2 >= self.m_count.get()
            {
                (None, f32::INFINITY)
            }
            else
            {
                let pathPoint2: Rc<PathPoint> = self.point(n2);
                let f2: f32 = pathPoint2.m_distanceToTarget.get();
                (Some(pathPoint2), f2)
            };
            if f3 < f2
            {
                if !(f3 < f)
                {
                    break;
                }
                pathPoint3.m_index.set(index);
                self.store(index, Some(pathPoint3));
                index = n;
                continue;
            }
            if !(f2 < f)
            {
                break;
            }
            let pathPoint2: Rc<PathPoint> = pathPoint2.unwrap_or_else(|| jvm::throwable::npe());
            pathPoint2.m_index.set(index);
            self.store(index, Some(pathPoint2));
            index = n2;
        }

        pathPoint.m_index.set(index);
        self.store(index, Some(pathPoint));
    }

    pub fn isPathEmpty(&self) -> bool { self.m_count.get() == 0 }
}

impl Default for Path
{
    fn default() -> Self { Self::new() }
}
