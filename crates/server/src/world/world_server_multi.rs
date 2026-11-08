use crate::server::MinecraftServer;
use crate::world::world_server;
use game::world::{ISaveHandler, World, WorldClass};
use std::rc::Rc;

pub fn new(
    server: &Rc<MinecraftServer>,
    storage: Rc<dyn ISaveHandler>,
    saveName: &str,
    dimension: i32,
    seed: i64,
    delegate: &Rc<World>,
) -> Rc<World>
{
    let this: Rc<World> = world_server::alloc(&CLASS);
    world_server::init(&this, server, storage, saveName, dimension, seed);
    this.m_savedDataStorage.set(delegate.m_savedDataStorage.get());
    this
}

pub const CLASS: WorldClass =
    WorldClass { m_name: "WorldServerMulti", m_parent: Some(&world_server::CLASS), ..world_server::CLASS };
