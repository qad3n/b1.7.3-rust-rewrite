// java.io.File calls are port stand-ins
use crate::port::file;
use crate::storage::chunk_loader::catchException;
use crate::storage::save_handler::{SaveHandler, readWorldInfo};
use crate::storage::{ISaveFormat, SaveFormatComparator};
use crate::world::{ISaveHandler, WorldInfo};
use common::nbt::{CompressedStreamTools, NBTBase};
use common::world::IProgressUpdate;
use jvm::io::{FileInputStream, FileOutputStream};
use jvm::string::JString;
use jvm::throwable::OrNpe;
use std::path::{Path, PathBuf};
use std::rc::Rc;

const OLD_WORLD_SLOTS: i32 = 5;

pub struct SaveFormatOld
{
    pub m_dir: PathBuf,
}

impl SaveFormatOld
{
    pub fn new(dir: &Path) -> SaveFormatOld
    {
        if !file::exists(dir)
        {
            file::mkdirs(dir);
        }
        SaveFormatOld { m_dir: dir.to_path_buf() }
    }

    pub fn deleteFilesAndDirs(files: &[PathBuf])
    {
        for path in files
        {
            if file::isDirectory(path)
            {
                Self::deleteFilesAndDirs(&file::listFiles(path).orNpe());
            }
            file::delete(path);
        }
    }
}

impl ISaveFormat for SaveFormatOld
{
    fn getName(&self) -> String { String::from("Old Format") }

    fn getSaveLoader(&self, saveName: &str, createPlayerDataDir: bool) -> Rc<dyn ISaveHandler>
    {
        Rc::new(SaveHandler::new(&self.m_dir, saveName, createPlayerDataDir))
    }

    fn getAll(&self) -> Vec<SaveFormatComparator>
    {
        let mut arraylist: Vec<SaveFormatComparator> = Vec::new();
        for i in 0..OLD_WORLD_SLOTS
        {
            let s: String = format!("World{}", i + 1);
            if let Some(worldinfo) = self.getData(&s)
            {
                arraylist.push(SaveFormatComparator::new(
                    s,
                    String::new(),
                    worldinfo.getLastTimePlayed(),
                    worldinfo.getSizeOnDisk(),
                    false,
                ));
            }
        }
        arraylist
    }

    fn flushCache(&self) {}

    fn getData(&self, saveName: &str) -> Option<Rc<WorldInfo>>
    {
        let file1: PathBuf = self.m_dir.join(saveName);
        if !file::exists(&file1)
        {
            return None;
        }
        readWorldInfo(&file1)
    }

    fn delete(&self, saveName: &str)
    {
        let path: PathBuf = self.m_dir.join(saveName);
        if !file::exists(&path)
        {
            return;
        }
        Self::deleteFilesAndDirs(&file::listFiles(&path).orNpe());
        file::delete(&path);
    }

    fn rename(&self, saveName: &str, newName: &str)
    {
        let path: PathBuf = self.m_dir.join(saveName);
        if !file::exists(&path)
        {
            return;
        }
        let file2: PathBuf = path.join("level.dat");
        if file::exists(&file2)
        {
            catchException(|| {
                let nbtTagCompound: Rc<NBTBase> =
                    CompressedStreamTools::readCompressed(FileInputStream::open(&file2)?)?;
                let nbtTagCompound2: Rc<NBTBase> = nbtTagCompound.compound().getCompoundTag("Data");
                nbtTagCompound2.compound().setString("LevelName", Some(JString::from(newName)));
                CompressedStreamTools::writeGzippedCompoundToOutputStream(
                    &nbtTagCompound,
                    FileOutputStream::create(&file2, false)?,
                )
            });
        }
    }

    fn isOldMapFormat(&self, _saveName: &str) -> bool { false }

    fn convertMapFormat(&self, _saveName: &str, _listener: &dyn IProgressUpdate) -> bool { false }
}
