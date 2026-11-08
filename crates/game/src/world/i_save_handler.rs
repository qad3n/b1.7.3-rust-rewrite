use crate::entity::Entity;
use crate::storage::IPlayerFileData;
use crate::world::{IChunkLoader, WorldInfo, WorldProvider};
use std::path::PathBuf;
use std::rc::Rc;

pub trait ISaveHandler
{
    fn loadWorldInfo(&self) -> Option<Rc<WorldInfo>>;
    fn checkSessionLock(&self);
    fn getChunkLoader(&self, provider: &Rc<WorldProvider>) -> Rc<dyn IChunkLoader>;
    fn saveWorldInfoAndPlayer(&self, info: &Rc<WorldInfo>, players: &[Rc<Entity>]);
    fn saveWorldInfo(&self, info: &Rc<WorldInfo>);
    fn getPlayerDataStorage(self: Rc<Self>) -> Rc<dyn IPlayerFileData>
    {
        crate::port::abstractMethod("getPlayerDataStorage")
    }

    fn forceSave(&self) { crate::port::abstractMethod("forceSave") }

    fn getDataFile(&self, name: &str) -> Option<PathBuf>;
}
