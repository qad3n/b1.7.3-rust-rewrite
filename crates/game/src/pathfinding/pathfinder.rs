use crate::block::{Block, Material};
use crate::entity::Entity;
use crate::math::math_helper::{floor_double, floor_float};
use crate::pathfinding::{MCHash, Path, PathEntity, PathPoint};
use crate::world::IBlockAccess;
use jvm::throwable::OrNpe;
use std::cell::RefCell;
use std::rc::Rc;

pub struct Pathfinder
{
    m_worldMap: Rc<dyn IBlockAccess>,
    m_path: Path,
    m_pointMap: RefCell<MCHash<Rc<PathPoint>>>,
    m_pathOptions: RefCell<Vec<Option<Rc<PathPoint>>>>,
}

impl Pathfinder
{
    pub fn new(world: Rc<dyn IBlockAccess>) -> Pathfinder
    {
        Pathfinder {
            m_path: Path::new(),
            m_pointMap: RefCell::new(MCHash::new()),
            m_pathOptions: RefCell::new(vec![None; 32]),
            m_worldMap: world,
        }
    }

    pub fn createEntityPathTo(&self, entity: &Rc<Entity>, target: &Rc<Entity>, range: f32) -> Option<Rc<PathEntity>>
    {
        self.createEntityPathToPos(entity, target.posX(), target.m_boundingBox.minY(), target.posZ(), range)
    }

    pub fn createEntityPathToXYZ(
        &self,
        entity: &Rc<Entity>,
        x: i32,
        y: i32,
        z: i32,
        range: f32,
    ) -> Option<Rc<PathEntity>>
    {
        self.createEntityPathToPos(
            entity,
            (x as f32 + 0.5f32) as f64,
            (y as f32 + 0.5f32) as f64,
            (z as f32 + 0.5f32) as f64,
            range,
        )
    }

    fn createEntityPathToPos(&self, entity: &Rc<Entity>, x: f64, y: f64, z: f64, range: f32) -> Option<Rc<PathEntity>>
    {
        self.m_path.clearPath();
        self.m_pointMap.borrow_mut().clearMap();
        let bounds: &crate::math::AxisAlignedBB = &entity.m_boundingBox;
        let pathPoint: Rc<PathPoint> =
            self.openPoint(floor_double(bounds.minX()), floor_double(bounds.minY()), floor_double(bounds.minZ()));
        let width: f32 = entity.m_width.get();
        let pathPoint2: Rc<PathPoint> = self.openPoint(
            floor_double(x - (width / 2.0) as f64),
            floor_double(y),
            floor_double(z - (width / 2.0) as f64),
        );
        let pathPoint3: Rc<PathPoint> = PathPoint::new(
            floor_float(width + 1.0),
            floor_float(entity.m_height.get() + 1.0),
            floor_float(width + 1.0),
        );
        self.addToPath(entity, &pathPoint, &pathPoint2, &pathPoint3, range)
    }

    fn addToPath(
        &self,
        entity: &Rc<Entity>,
        start: &Rc<PathPoint>,
        target: &Rc<PathPoint>,
        bounds: &PathPoint,
        range: f32,
    ) -> Option<Rc<PathEntity>>
    {
        start.m_totalPathDistance.set(0.0);
        start.m_distanceToNext.set(start.distanceTo(target));
        start.m_distanceToTarget.set(start.m_distanceToNext.get());
        self.m_path.clearPath();
        self.m_path.addPoint(start);
        let mut target2: Rc<PathPoint> = Rc::clone(start);
        while !self.m_path.isPathEmpty()
        {
            let dequeue: Rc<PathPoint> = self.m_path.dequeue();
            if dequeue.equals(target)
            {
                return Some(self.createEntityPath(start, target));
            }

            if dequeue.distanceTo(target) < target2.distanceTo(target)
            {
                target2 = Rc::clone(&dequeue);
            }

            dequeue.m_isFirst.set(true);
            let pathOptions: i32 = self.findPathOptions(entity, &dequeue, bounds, target, range);
            for i in 0..pathOptions
            {
                let node: Rc<PathPoint> = self.m_pathOptions.borrow()[i as usize].clone().orNpe();
                let totalPathDistance: f32 = dequeue.m_totalPathDistance.get() + dequeue.distanceTo(&node);
                if !node.isAssigned() || totalPathDistance < node.m_totalPathDistance.get()
                {
                    node.m_previous.set(Some(Rc::clone(&dequeue)));
                    node.m_totalPathDistance.set(totalPathDistance);
                    node.m_distanceToNext.set(node.distanceTo(target));
                    if node.isAssigned()
                    {
                        self.m_path.changeDistance(&node, node.m_totalPathDistance.get() + node.m_distanceToNext.get());
                    }
                    else
                    {
                        node.m_distanceToTarget.set(node.m_totalPathDistance.get() + node.m_distanceToNext.get());
                        self.m_path.addPoint(&node);
                    }
                }
            }
        }

        if Rc::ptr_eq(&target2, start)
        {
            return None;
        }
        Some(self.createEntityPath(start, &target2))
    }

    fn findPathOptions(
        &self,
        entity: &Rc<Entity>,
        node: &PathPoint,
        bounds: &PathPoint,
        target: &PathPoint,
        range: f32,
    ) -> i32
    {
        let mut i: i32 = 0;
        let mut b0: i32 = 0;
        if self.getVerticalOffset(entity, node.m_xCoord, node.m_yCoord + 1, node.m_zCoord, bounds) == 1
        {
            b0 = 1;
        }

        let candidates: [Option<Rc<PathPoint>>; 4] = [
            self.getSafePoint(entity, node.m_xCoord, node.m_yCoord, node.m_zCoord + 1, bounds, b0),
            self.getSafePoint(entity, node.m_xCoord - 1, node.m_yCoord, node.m_zCoord, bounds, b0),
            self.getSafePoint(entity, node.m_xCoord + 1, node.m_yCoord, node.m_zCoord, bounds, b0),
            self.getSafePoint(entity, node.m_xCoord, node.m_yCoord, node.m_zCoord - 1, bounds, b0),
        ];
        for pathpoint in candidates.into_iter().flatten()
        {
            if !pathpoint.m_isFirst.get() && pathpoint.distanceTo(target) < range
            {
                self.m_pathOptions.borrow_mut()[i as usize] = Some(pathpoint);
                i += 1;
            }
        }

        i
    }

    fn getSafePoint(
        &self,
        entity: &Rc<Entity>,
        x: i32,
        mut y: i32,
        z: i32,
        bounds: &PathPoint,
        neighborBlockingType: i32,
    ) -> Option<Rc<PathPoint>>
    {
        let mut pathpoint: Option<Rc<PathPoint>> = None;
        if self.getVerticalOffset(entity, x, y, z, bounds) == 1
        {
            pathpoint = Some(self.openPoint(x, y, z));
        }

        if pathpoint.is_none()
            && neighborBlockingType > 0
            && self.getVerticalOffset(entity, x, y + neighborBlockingType, z, bounds) == 1
        {
            pathpoint = Some(self.openPoint(x, y + neighborBlockingType, z));
            y += neighborBlockingType;
        }

        if pathpoint.is_some()
        {
            let mut i: i32 = 0;
            let mut j: i32 = 0;
            while y > 0
            {
                j = self.getVerticalOffset(entity, x, y - 1, z, bounds);
                if j != 1
                {
                    break;
                }
                i += 1;
                if i >= 4
                {
                    return None;
                }
                y -= 1;
                if y > 0
                {
                    pathpoint = Some(self.openPoint(x, y, z));
                }
            }

            if j == -2
            {
                return None;
            }
        }

        pathpoint
    }

    fn openPoint(&self, x: i32, y: i32, z: i32) -> Rc<PathPoint>
    {
        let i: i32 = PathPoint::hash(x, y, z);
        let found: Option<Rc<PathPoint>> = self.m_pointMap.borrow().lookup(i);
        match found
        {
            Some(pathpoint) => pathpoint,
            None =>
            {
                let pathpoint: Rc<PathPoint> = PathPoint::new(x, y, z);
                self.m_pointMap.borrow_mut().addKey(i, Rc::clone(&pathpoint));
                pathpoint
            }
        }
    }

    fn getVerticalOffset(&self, _entity: &Rc<Entity>, x: i32, y: i32, z: i32, bounds: &PathPoint) -> i32
    {
        for i in x..x + bounds.m_xCoord
        {
            for j in y..y + bounds.m_yCoord
            {
                for k in z..z + bounds.m_zCoord
                {
                    let l: i32 = self.m_worldMap.getBlockId(i, j, k);
                    if l <= 0
                    {
                        continue;
                    }
                    if l != Block::doorSteel().m_blockID && l != Block::doorWood().m_blockID
                    {
                        let material: Rc<Material> = Rc::clone(&Block::byId(l).orNpe().m_blockMaterial);
                        if material.getIsSolid()
                        {
                            return 0;
                        }
                        if Rc::ptr_eq(&material, &Material::water())
                        {
                            return -1;
                        }
                        if Rc::ptr_eq(&material, &Material::lava())
                        {
                            return -2;
                        }
                    }
                    else
                    {
                        let i1: i32 = self.m_worldMap.getBlockMetadata(i, j, k);
                        if !crate::block::block_door::isOpen(i1)
                        {
                            return 0;
                        }
                    }
                }
            }
        }

        1
    }

    fn createEntityPath(&self, _start: &Rc<PathPoint>, target: &Rc<PathPoint>) -> Rc<PathEntity>
    {
        let mut i: usize = 1;
        let mut pathpoint: Rc<PathPoint> = Rc::clone(target);
        while let Some(previous) = pathpoint.m_previous.get()
        {
            i += 1;
            pathpoint = previous;
        }

        let mut apathpoint: Vec<Option<Rc<PathPoint>>> = vec![None; i];
        let mut pathpoint1: Rc<PathPoint> = Rc::clone(target);
        i -= 1;
        apathpoint[i] = Some(Rc::clone(&pathpoint1));
        while let Some(previous) = pathpoint1.m_previous.get()
        {
            pathpoint1 = previous;
            i -= 1;
            apathpoint[i] = Some(Rc::clone(&pathpoint1));
        }

        PathEntity::new(apathpoint.into_iter().map(OrNpe::orNpe).collect())
    }
}
