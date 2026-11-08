use crate::storage::SaveFormatComparator;
use crate::world::{ISaveHandler, WorldInfo};
use common::world::IProgressUpdate;
use std::rc::Rc;

pub trait ISaveFormat
{
    fn getName(&self) -> String;
    fn getSaveLoader(&self, saveName: &str, createPlayerDataDir: bool) -> Rc<dyn ISaveHandler>;
    fn getAll(&self) -> Vec<SaveFormatComparator>;
    fn flushCache(&self);
    fn getData(&self, saveName: &str) -> Option<Rc<WorldInfo>>;
    fn delete(&self, saveName: &str);
    fn rename(&self, saveName: &str, newName: &str);
    fn isOldMapFormat(&self, saveName: &str) -> bool;
    fn convertMapFormat(&self, saveName: &str, listener: &dyn IProgressUpdate) -> bool;
}
