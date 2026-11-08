use super::{blocks, entities, ticks, weather};
use crate::entity::Entity;
use crate::world::{ChunkProvider, Explosion, IChunkProvider, World};
use std::rc::Rc;

crate::vtable! {
    WorldClass for World {
        fn TickUpdates(flush: bool) -> bool;
        fn addWeatherEffect(entity: Rc<Entity>) -> bool;
        fn canModify(player: &Rc<Entity>, x: i32, y: i32, z: i32) -> bool;
        fn doEntityEvent(entity: &Rc<Entity>, event: i8);
        fn entityJoinedWorld(entity: &Rc<Entity>) -> bool;
        fn getChunkProvider() -> Rc<dyn IChunkProvider>;
        fn newExplosion(source: Option<&Rc<Entity>>, x: f64, y: f64, z: f64, power: f32, createFire: bool) -> Rc<Explosion>;
        fn obtainEntitySkin(entity: &Rc<Entity>);
        fn playNoteAt(x: i32, y: i32, z: i32, kind: i32, data: i32);
        fn releaseEntitySkin(entity: &Rc<Entity>);
        fn scheduleBlockUpdate(x: i32, y: i32, z: i32, block: i32, delay: i32);
        fn sendQuittingDisconnectingPacket();
        fn setBlock(x: i32, y: i32, z: i32, block: i32) -> bool;
        fn setBlockAndMetadata(x: i32, y: i32, z: i32, block: i32, metadata: i32) -> bool;
        fn setBlockMetadata(x: i32, y: i32, z: i32, metadata: i32) -> bool;
        fn setEntityDead(entity: &Rc<Entity>);
        fn setSpawnLocation();
        fn tick();
        fn updateBlocksAndPlayCaveSounds();
        fn updateEntityWithOptionalForce(entity: &Rc<Entity>, requireLoaded: bool);
        fn updateWeather();
    }
}

fn getChunkProvider(this: &Rc<World>) -> Rc<dyn IChunkProvider>
{
    let provider: Rc<crate::world::WorldProvider> = this.worldProvider();
    let ichunkloader: Rc<dyn crate::world::IChunkLoader> = this.saveHandler().getChunkLoader(&provider);
    ChunkProvider::new(this, ichunkloader, Some(provider.getChunkProvider()))
}

pub const WORLD: WorldClass = WorldClass {
    m_name: "World",
    m_parent: None,
    m_interfaces: &[],
    TickUpdates: ticks::TickUpdates,
    addWeatherEffect: entities::addWeatherEffect,
    canModify: super::canModify,
    doEntityEvent: super::doEntityEvent,
    entityJoinedWorld: entities::entityJoinedWorld,
    getChunkProvider,
    newExplosion: entities::newExplosion,
    obtainEntitySkin: entities::obtainEntitySkin,
    playNoteAt: blocks::playNoteAt,
    releaseEntitySkin: entities::releaseEntitySkin,
    scheduleBlockUpdate: ticks::scheduleBlockUpdate,
    sendQuittingDisconnectingPacket: |_| {},
    setBlock: blocks::setBlock,
    setBlockAndMetadata: blocks::setBlockAndMetadata,
    setBlockMetadata: blocks::setBlockMetadata,
    setEntityDead: entities::setEntityDead,
    setSpawnLocation: weather::setSpawnLocation,
    tick: weather::tick,
    updateBlocksAndPlayCaveSounds: ticks::updateBlocksAndPlayCaveSounds,
    updateEntityWithOptionalForce: entities::updateEntityWithOptionalForce,
    updateWeather: weather::updateWeather,
};
