use game::math::AxisAlignedBB;

pub trait ICamera
{
    fn isBoundingBoxInFrustum(&self, shape: &AxisAlignedBB) -> bool;

    fn setPosition(&self, x: f64, y: f64, z: f64);
}
