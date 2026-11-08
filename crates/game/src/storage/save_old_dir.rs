use crate::entity::Entity;
// java.io.File calls are port stand-ins
use crate::port::file;
use crate::storage::mc_region_chunk_loader::McRegionChunkLoader;
use crate::storage::save_handler::SaveHandler;
use crate::storage::{IPlayerFileData, RegionFileCache};
use crate::world::{IChunkLoader, ISaveHandler, WorldInfo, WorldProvider};
use std::path::{Path, PathBuf};
use std::rc::Rc;

pub const MC_REGION_VERSION: i32 = 19132;

pub struct SaveOldDir
{
    m_base: SaveHandler,
}

impl SaveOldDir
{
    pub fn new(file: &Path, name: &str, createPlayerDataDir: bool) -> SaveOldDir
    {
        SaveOldDir { m_base: SaveHandler::new(file, name, createPlayerDataDir) }
    }
}

impl ISaveHandler for SaveOldDir
{
    fn loadWorldInfo(&self) -> Option<Rc<WorldInfo>> { self.m_base.loadWorldInfo() }

    fn checkSessionLock(&self) { self.m_base.checkSessionLock(); }

    fn getChunkLoader(&self, provider: &Rc<WorldProvider>) -> Rc<dyn IChunkLoader>
    {
        let file: &Path = self.m_base.getSaveDirectory();
        if provider.instanceOf("WorldProviderHell")
        {
            let file2: PathBuf = file.join("DIM-1");
            file::mkdirs(&file2);
            return Rc::new(McRegionChunkLoader::new(&file2));
        }
        Rc::new(McRegionChunkLoader::new(file))
    }

    fn saveWorldInfoAndPlayer(&self, info: &Rc<WorldInfo>, players: &[Rc<Entity>])
    {
        info.setSaveVersion(MC_REGION_VERSION);
        self.m_base.saveWorldInfoAndPlayer(info, players);
    }

    fn saveWorldInfo(&self, info: &Rc<WorldInfo>) { self.m_base.saveWorldInfo(info); }

    fn getPlayerDataStorage(self: Rc<Self>) -> Rc<dyn IPlayerFileData> { self }

    fn forceSave(&self) { RegionFileCache::flush(); }

    fn getDataFile(&self, name: &str) -> Option<PathBuf> { self.m_base.getDataFile(name) }
}

impl IPlayerFileData for SaveOldDir
{
    fn writePlayerData(&self, player: &Rc<Entity>) { self.m_base.writePlayerData(player); }

    fn readPlayerData(&self, player: &Rc<Entity>) { self.m_base.readPlayerData(player); }
}
