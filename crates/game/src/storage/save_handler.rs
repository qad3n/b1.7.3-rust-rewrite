use crate::entity::{Entity, entity_player};
// java.io.File calls are port stand-ins
use crate::port::file;
use crate::storage::IPlayerFileData;
use crate::storage::chunk_loader::{ChunkLoader, catchException};
use crate::world::{IChunkLoader, ISaveHandler, WorldInfo, WorldProvider};
use common::nbt::{CompressedStreamTools, NBTBase, NBTTagCompound};
use common::world::MINECRAFT_EXCEPTION;
use jvm::io::{DataInputStream, DataOutputStream, FileInputStream, FileOutputStream, InputStream, IoResult};
use jvm::logging::Logger;
use jvm::string::JString;
use jvm::throwable;
use std::path::{Path, PathBuf};
use std::rc::Rc;

pub struct SaveHandler
{
    m_saveDirectory: PathBuf,
    m_playersDirectory: PathBuf,
    m_dataDir: PathBuf,
    m_now: i64,
}

pub(super) fn readWorldInfo(dir: &Path) -> Option<Rc<WorldInfo>>
{
    for name in ["level.dat", "level.dat_old"]
    {
        let file1: PathBuf = dir.join(name);
        if !file::exists(&file1)
        {
            continue;
        }
        let info: Option<Rc<WorldInfo>> = catchException(|| {
            let nbttagcompound: Rc<NBTBase> = CompressedStreamTools::readCompressed(FileInputStream::open(&file1)?)?;
            let data: Rc<NBTBase> = nbttagcompound.compound().getCompoundTag("Data");
            Ok(Rc::new(WorldInfo::fromNBT(data.compound())))
        });
        if info.is_some()
        {
            return info;
        }
    }
    None
}

impl SaveHandler
{
    pub fn new(savesDir: &Path, name: &str, createPlayerDataDir: bool) -> SaveHandler
    {
        let now: i64 = jvm::system::currentTimeMillis();
        let saveDirectory: PathBuf = savesDir.join(name);
        file::mkdirs(&saveDirectory);
        let playersDirectory: PathBuf = saveDirectory.join("players");
        let dataDir: PathBuf = saveDirectory.join("data");
        file::mkdirs(&dataDir);
        if createPlayerDataDir
        {
            file::mkdirs(&playersDirectory);
        }
        let handler: SaveHandler = SaveHandler {
            m_saveDirectory: saveDirectory,
            m_playersDirectory: playersDirectory,
            m_dataDir: dataDir,
            m_now: now,
        };
        handler.writeSessionLock();
        handler
    }

    fn writeSessionLock(&self)
    {
        let result: IoResult<()> = (|| {
            let file1: PathBuf = self.m_saveDirectory.join("session.lock");
            let mut dataoutputstream: DataOutputStream<FileOutputStream> =
                DataOutputStream::new(FileOutputStream::create(&file1, false)?);
            let written: IoResult<()> = dataoutputstream.writeLong(self.m_now);
            let closed: IoResult<()> = dataoutputstream.close();
            written?;
            closed
        })();
        if let Err(error) = result
        {
            error.printStackTrace();
            throwable::throw(&throwable::RUNTIME, "Failed to check session lock, aborting");
        }
    }

    pub fn getSaveDirectory(&self) -> &Path { &self.m_saveDirectory }

    pub fn getPlayerData(&self, playerName: Option<&JString>) -> Option<Rc<NBTBase>>
    {
        let result: Result<IoResult<Option<Rc<NBTBase>>>, throwable::Throwable> =
            throwable::catchThrowable(&throwable::EXCEPTION, || {
                let file1: PathBuf = self.m_playersDirectory.join(format!("{}.dat", JString::display(playerName)));
                if file::exists(&file1)
                {
                    return Ok(Some(CompressedStreamTools::readCompressed(FileInputStream::open(&file1)?)?));
                }
                Ok(None)
            });
        match result.and_then(|inner| inner)
        {
            Ok(Some(nbt)) => return Some(nbt),
            Ok(None) =>
            {}
            Err(_) => Logger::getLogger("Minecraft")
                .warning(&format!("Failed to load player data for {}", JString::display(playerName))),
        }
        None
    }

    fn writeLevel(&self, nbttagcompound: Rc<NBTBase>)
    {
        let nbttagcompound1: Rc<NBTBase> = NBTTagCompound::create();
        nbttagcompound1.compound().setTag("Data", &nbttagcompound);
        catchException(|| {
            let file1: PathBuf = self.m_saveDirectory.join("level.dat_new");
            let file2: PathBuf = self.m_saveDirectory.join("level.dat_old");
            let file3: PathBuf = self.m_saveDirectory.join("level.dat");
            CompressedStreamTools::writeGzippedCompoundToOutputStream(
                &nbttagcompound1,
                FileOutputStream::create(&file1, false)?,
            )?;
            if file::exists(&file2)
            {
                file::delete(&file2);
            }
            file::renameTo(&file3, &file2);
            if file::exists(&file3)
            {
                file::delete(&file3);
            }
            file::renameTo(&file1, &file3);
            if file::exists(&file1)
            {
                file::delete(&file1);
            }
            Ok(())
        });
    }
}

impl ISaveHandler for SaveHandler
{
    fn loadWorldInfo(&self) -> Option<Rc<WorldInfo>> { readWorldInfo(&self.m_saveDirectory) }

    fn checkSessionLock(&self)
    {
        let result: IoResult<()> = (|| {
            let file1: PathBuf = self.m_saveDirectory.join("session.lock");
            let mut datainputstream: DataInputStream<FileInputStream> =
                DataInputStream::new(FileInputStream::open(&file1)?);
            let lock: IoResult<i64> = datainputstream.readLong();
            datainputstream.m_in.close()?;
            if lock? != self.m_now
            {
                throwable::throw(&MINECRAFT_EXCEPTION, "The save is being accessed from another location, aborting");
            }
            Ok(())
        })();
        if result.is_err()
        {
            throwable::throw(&MINECRAFT_EXCEPTION, "Failed to check session lock, aborting");
        }
    }

    fn getChunkLoader(&self, provider: &Rc<WorldProvider>) -> Rc<dyn IChunkLoader>
    {
        if provider.instanceOf("WorldProviderHell")
        {
            let file1: PathBuf = self.m_saveDirectory.join("DIM-1");
            file::mkdirs(&file1);
            return Rc::new(ChunkLoader::new(&file1, true));
        }
        Rc::new(ChunkLoader::new(&self.m_saveDirectory, true))
    }

    fn saveWorldInfoAndPlayer(&self, info: &Rc<WorldInfo>, players: &[Rc<Entity>])
    {
        self.writeLevel(info.getNBTTagCompoundWithPlayer(players));
    }

    fn saveWorldInfo(&self, info: &Rc<WorldInfo>) { self.writeLevel(info.getNBTTagCompound()); }

    fn getPlayerDataStorage(self: Rc<Self>) -> Rc<dyn IPlayerFileData> { self }

    fn forceSave(&self) {}

    fn getDataFile(&self, name: &str) -> Option<PathBuf> { Some(self.m_dataDir.join(format!("{name}.dat"))) }
}

impl IPlayerFileData for SaveHandler
{
    fn writePlayerData(&self, player: &Rc<Entity>)
    {
        let result: Result<IoResult<()>, throwable::Throwable> =
            throwable::catchThrowable(&throwable::EXCEPTION, || {
                let nbttagcompound: Rc<NBTBase> = NBTTagCompound::create();
                player.writeToNBT(nbttagcompound.compound());
                let file1: PathBuf = self.m_playersDirectory.join("_tmp_.dat");
                let file2: PathBuf = self
                    .m_playersDirectory
                    .join(format!("{}.dat", JString::display(entity_player::fields(player).m_username.get().as_ref())));
                CompressedStreamTools::writeGzippedCompoundToOutputStream(
                    &nbttagcompound,
                    FileOutputStream::create(&file1, false)?,
                )?;
                if file::exists(&file2)
                {
                    file::delete(&file2);
                }
                file::renameTo(&file1, &file2);
                Ok(())
            });
        if result.and_then(|inner| inner).is_err()
        {
            Logger::getLogger("Minecraft").warning(&format!(
                "Failed to save player data for {}",
                JString::display(entity_player::fields(player).m_username.get().as_ref())
            ));
        }
    }

    fn readPlayerData(&self, player: &Rc<Entity>)
    {
        let nbttagcompound: Option<Rc<NBTBase>> =
            self.getPlayerData(entity_player::fields(player).m_username.get().as_ref());
        if let Some(nbttagcompound) = nbttagcompound
        {
            player.readFromNBT(nbttagcompound.compound());
        }
    }
}
