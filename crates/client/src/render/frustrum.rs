use crate::render::{ClippingHelperImpl, ICamera};
use game::math::AxisAlignedBB;
use std::cell::Cell;
use std::rc::Rc;

pub struct Frustrum
{
    m_clippingHelper: Rc<ClippingHelperImpl>,
    m_xPosition: Cell<f64>,
    m_yPosition: Cell<f64>,
    m_zPosition: Cell<f64>,
}

impl Frustrum
{
    pub fn new() -> Frustrum
    {
        Frustrum {
            m_clippingHelper: ClippingHelperImpl::getInstance(),
            m_xPosition: Cell::new(0.0),
            m_yPosition: Cell::new(0.0),
            m_zPosition: Cell::new(0.0),
        }
    }

    pub fn isBoxInFrustum(&self, minX: f64, minY: f64, minZ: f64, maxX: f64, maxY: f64, maxZ: f64) -> bool
    {
        self.m_clippingHelper.m_helper.isBoxInFrustum(
            minX - self.m_xPosition.get(),
            minY - self.m_yPosition.get(),
            minZ - self.m_zPosition.get(),
            maxX - self.m_xPosition.get(),
            maxY - self.m_yPosition.get(),
            maxZ - self.m_zPosition.get(),
        )
    }
}

impl Default for Frustrum
{
    fn default() -> Self { Self::new() }
}

impl ICamera for Frustrum
{
    fn isBoundingBoxInFrustum(&self, shape: &AxisAlignedBB) -> bool
    {
        self.isBoxInFrustum(
            shape.m_minX.get(),
            shape.m_minY.get(),
            shape.m_minZ.get(),
            shape.m_maxX.get(),
            shape.m_maxY.get(),
            shape.m_maxZ.get(),
        )
    }

    fn setPosition(&self, offsetX: f64, offsetY: f64, offsetZ: f64)
    {
        self.m_xPosition.set(offsetX);
        self.m_yPosition.set(offsetY);
        self.m_zPosition.set(offsetZ);
    }
}
