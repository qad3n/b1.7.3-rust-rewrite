use crate::entity::Entity;
use crate::math::Vec3D;
use std::rc::Rc;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EnumMovingObjectType
{
    TILE,
    ENTITY,
}

#[derive(Clone)]
pub struct MovingObjectPosition
{
    pub m_typeOfHit: EnumMovingObjectType,
    pub m_blockX: i32,
    pub m_blockY: i32,
    pub m_blockZ: i32,
    pub m_sideHit: i32,
    pub m_hitVec: Rc<Vec3D>,
    pub m_entityHit: Option<Rc<Entity>>,
}

impl MovingObjectPosition
{
    pub fn newTile(x: i32, y: i32, z: i32, face: i32, facePos: &Vec3D) -> Self
    {
        Self {
            m_typeOfHit: EnumMovingObjectType::TILE,
            m_blockX: x,
            m_blockY: y,
            m_blockZ: z,
            m_sideHit: face,
            m_hitVec: Vec3D::createVector(facePos.x(), facePos.y(), facePos.z()),
            m_entityHit: None,
        }
    }

    pub fn newEntity(entity: &Rc<Entity>) -> Self
    {
        Self {
            m_typeOfHit: EnumMovingObjectType::ENTITY,
            m_blockX: 0,
            m_blockY: 0,
            m_blockZ: 0,
            m_sideHit: 0,
            m_hitVec: Vec3D::createVector(entity.posX(), entity.posY(), entity.posZ()),
            m_entityHit: Some(Rc::clone(entity)),
        }
    }
}
