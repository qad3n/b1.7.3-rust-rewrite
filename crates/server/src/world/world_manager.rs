use crate::server::MinecraftServer;
use game::entity::Entity;
use game::network::packets::Packet61DoorChange;
use game::tileentity::TileEntity;
use game::world::{IWorldAccess, World};
use std::rc::Rc;

pub struct WorldManager
{
    m_mcServer: Rc<MinecraftServer>,
    m_world: Rc<World>,
}

impl WorldManager
{
    pub fn new(server: &Rc<MinecraftServer>, world: &Rc<World>) -> WorldManager
    {
        WorldManager { m_mcServer: Rc::clone(server), m_world: Rc::clone(world) }
    }

    fn worldType(&self) -> i32 { self.m_world.worldProvider().m_worldType.get() }
}

impl IWorldAccess for WorldManager
{
    fn spawnParticle(&self, _kind: &str, _x: f64, _y: f64, _z: f64, _velocityX: f64, _velocityY: f64, _velocityZ: f64)
    {
    }

    fn obtainEntitySkin(&self, entity: &Rc<Entity>)
    {
        self.m_mcServer.getEntityTracker(self.worldType()).trackEntity(entity);
    }

    fn releaseEntitySkin(&self, entity: &Rc<Entity>)
    {
        self.m_mcServer.getEntityTracker(self.worldType()).untrackEntity(entity);
    }

    fn playSound(&self, _sound: &str, _x: f64, _y: f64, _z: f64, _volume: f32, _pitch: f32) {}

    fn markBlockRangeNeedsUpdate(&self, _minX: i32, _minY: i32, _minZ: i32, _maxX: i32, _maxY: i32, _maxZ: i32) {}

    fn updateAllRenderers(&self) {}

    fn markBlockAndNeighborsNeedsUpdate(&self, x: i32, y: i32, z: i32)
    {
        self.m_mcServer.configManager().markBlockNeedsUpdate(x, y, z, self.worldType());
    }

    fn playRecord(&self, _record: Option<&str>, _x: i32, _y: i32, _z: i32) {}

    fn doNothingWithTileEntity(&self, x: i32, y: i32, z: i32, blockEntity: &Rc<TileEntity>)
    {
        self.m_mcServer.configManager().sentTileEntityToPlayer(x, y, z, blockEntity);
    }

    fn doEvent(&self, source: Option<&Rc<Entity>>, kind: i32, x: i32, y: i32, z: i32, data: i32)
    {
        self.m_mcServer.configManager().sendPacket(
            source,
            x as f64,
            y as f64,
            z as f64,
            64.0,
            self.worldType(),
            Box::new(Packet61DoorChange::new(kind, x, y, z, data)),
        );
    }
}
