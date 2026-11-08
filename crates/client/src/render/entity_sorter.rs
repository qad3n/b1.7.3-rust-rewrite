use crate::render::WorldRenderer;
use game::entity::Entity;

pub struct EntitySorter
{
    m_cameraX: f64,
    m_cameraY: f64,
    m_cameraZ: f64,
}

impl EntitySorter
{
    pub fn new(camera: &Entity) -> EntitySorter
    {
        EntitySorter { m_cameraX: -camera.posX(), m_cameraY: -camera.posY(), m_cameraZ: -camera.posZ() }
    }

    pub fn compare(&self, worldRenderer: &WorldRenderer, worldRenderer2: &WorldRenderer) -> i32
    {
        let d: f64 = worldRenderer.m_posXPlus.get() as f64 + self.m_cameraX;
        let d2: f64 = worldRenderer.m_posYPlus.get() as f64 + self.m_cameraY;
        let d3: f64 = worldRenderer.m_posZPlus.get() as f64 + self.m_cameraZ;
        let d4: f64 = worldRenderer2.m_posXPlus.get() as f64 + self.m_cameraX;
        let d5: f64 = worldRenderer2.m_posYPlus.get() as f64 + self.m_cameraY;
        let d6: f64 = worldRenderer2.m_posZPlus.get() as f64 + self.m_cameraZ;
        ((d * d + d2 * d2 + d3 * d3 - (d4 * d4 + d5 * d5 + d6 * d6)) * 1024.0) as i32
    }
}
