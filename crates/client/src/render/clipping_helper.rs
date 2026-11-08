use std::cell::RefCell;

pub struct ClippingHelper
{
    pub m_frustum: RefCell<[[f32; 16]; 16]>,
    pub m_projectionMatrix: RefCell<[f32; 16]>,
    pub m_modelviewMatrix: RefCell<[f32; 16]>,
    pub m_clippingMatrix: RefCell<[f32; 16]>,
}

impl ClippingHelper
{
    pub fn new() -> ClippingHelper
    {
        ClippingHelper {
            m_frustum: RefCell::new([[0.0; 16]; 16]),
            m_projectionMatrix: RefCell::new([0.0; 16]),
            m_modelviewMatrix: RefCell::new([0.0; 16]),
            m_clippingMatrix: RefCell::new([0.0; 16]),
        }
    }

    pub fn isBoxInFrustum(&self, minX: f64, minY: f64, minZ: f64, maxX: f64, maxY: f64, maxZ: f64) -> bool
    {
        let frustum: std::cell::Ref<'_, [[f32; 16]; 16]> = self.m_frustum.borrow();
        for plane in frustum.iter().take(6)
        {
            let a: f64 = plane[0] as f64;
            let b: f64 = plane[1] as f64;
            let c: f64 = plane[2] as f64;
            let d: f64 = plane[3] as f64;
            if a * minX + b * minY + c * minZ + d > 0.0
                || a * maxX + b * minY + c * minZ + d > 0.0
                || a * minX + b * maxY + c * minZ + d > 0.0
                || a * maxX + b * maxY + c * minZ + d > 0.0
                || a * minX + b * minY + c * maxZ + d > 0.0
                || a * maxX + b * minY + c * maxZ + d > 0.0
                || a * minX + b * maxY + c * maxZ + d > 0.0
                || a * maxX + b * maxY + c * maxZ + d > 0.0
            {
                continue;
            }
            return false;
        }
        true
    }
}

impl Default for ClippingHelper
{
    fn default() -> Self { Self::new() }
}
