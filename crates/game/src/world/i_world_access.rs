use crate::entity::Entity;
use crate::tileentity::TileEntity;
use jvm::object::JavaObject;
use std::rc::Rc;

pub trait IWorldAccess
{
    fn markBlockAndNeighborsNeedsUpdate(&self, x: i32, y: i32, z: i32);
    fn markBlockRangeNeedsUpdate(&self, minX: i32, minY: i32, minZ: i32, maxX: i32, maxY: i32, maxZ: i32);
    fn playSound(&self, sound: &str, x: f64, y: f64, z: f64, volume: f32, pitch: f32);
    fn spawnParticle(&self, kind: &str, x: f64, y: f64, z: f64, velocityX: f64, velocityY: f64, velocityZ: f64);
    fn obtainEntitySkin(&self, entity: &Rc<Entity>);
    fn releaseEntitySkin(&self, entity: &Rc<Entity>);
    fn updateAllRenderers(&self);
    fn playRecord(&self, record: Option<&str>, x: i32, y: i32, z: i32);
    fn doNothingWithTileEntity(&self, x: i32, y: i32, z: i32, blockEntity: &Rc<TileEntity>);
    fn doEvent(&self, source: Option<&Rc<Entity>>, kind: i32, x: i32, y: i32, z: i32, data: i32);
}

impl JavaObject for dyn IWorldAccess
{
    fn javaEquals(&self, other: &Self) -> bool { std::ptr::addr_eq(self, other) }

    fn javaHash(&self) -> i32 { (self as *const Self as *const u8 as usize) as i32 }
}
