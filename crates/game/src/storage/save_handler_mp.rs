use crate::entity::Entity;
use crate::world::{IChunkLoader, ISaveHandler, WorldInfo, WorldProvider};
use std::path::PathBuf;
use std::rc::Rc;

pub struct SaveHandlerMP;

impl ISaveHandler for SaveHandlerMP
{
    fn loadWorldInfo(&self) -> Option<Rc<WorldInfo>> { None }

    fn checkSessionLock(&self) {}

    // java returns null here and rust throws the npe its caller would hit
    fn getChunkLoader(&self, _provider: &Rc<WorldProvider>) -> Rc<dyn IChunkLoader> { jvm::throwable::npe() }

    fn saveWorldInfoAndPlayer(&self, _info: &Rc<WorldInfo>, _players: &[Rc<Entity>]) {}

    fn saveWorldInfo(&self, _info: &Rc<WorldInfo>) {}

    fn getDataFile(&self, _name: &str) -> Option<PathBuf> { None }
}
