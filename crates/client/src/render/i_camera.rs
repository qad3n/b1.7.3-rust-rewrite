use game::math::AxisAlignedBB;

pub trait ICamera
{
    fn isBoxInFrustum(&self, minX: f64, minY: f64, minZ: f64, maxX: f64, maxY: f64, maxZ: f64) -> bool;

    fn isBoundingBoxInFrustum(&self, shape: &AxisAlignedBB) -> bool;

    fn setPosition(&self, x: f64, y: f64, z: f64);
}
