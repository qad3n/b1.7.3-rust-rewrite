use crate::render::WorldRenderer;
use game::entity::Entity;
use std::rc::Rc;

pub struct RenderSorter
{
    m_baseEntity: Rc<Entity>,
}

impl RenderSorter
{
    pub fn new(camera: &Rc<Entity>) -> RenderSorter { RenderSorter { m_baseEntity: Rc::clone(camera) } }

    pub fn compare(&self, var1: &WorldRenderer, var2: &WorldRenderer) -> i32
    {
        let flag: bool = var1.m_isInFrustum.get();
        let flag1: bool = var2.m_isInFrustum.get();
        if flag && !flag1
        {
            return 1;
        }
        if flag1 && !flag
        {
            return -1;
        }
        let d0: f64 = var1.distanceToEntitySquared(&self.m_baseEntity) as f64;
        let d1: f64 = var2.distanceToEntitySquared(&self.m_baseEntity) as f64;
        if d0 < d1
        {
            return 1;
        }
        if d0 > d1
        {
            return -1;
        }
        if var1.m_chunkIndex.get() < var2.m_chunkIndex.get()
        {
            1
        }
        else
        {
            -1
        }
    }
}
