use game::math::Vec3D;
use std::rc::Rc;

#[derive(Clone)]
pub struct PositionTextureVertex
{
    pub m_vector3D: Rc<Vec3D>,
    pub m_texturePositionX: f32,
    pub m_texturePositionY: f32,
}

impl PositionTextureVertex
{
    pub fn new(x: f32, y: f32, z: f32, u: f32, v: f32) -> PositionTextureVertex
    {
        Self::withVector(Vec3D::createVectorHelper(x as f64, y as f64, z as f64), u, v)
    }

    pub fn withVector(pos: Rc<Vec3D>, u: f32, v: f32) -> PositionTextureVertex
    {
        PositionTextureVertex { m_vector3D: pos, m_texturePositionX: u, m_texturePositionY: v }
    }

    pub fn setTexturePosition(&self, u: f32, v: f32) -> PositionTextureVertex
    {
        Self::withVector(Rc::clone(&self.m_vector3D), u, v)
    }
}
