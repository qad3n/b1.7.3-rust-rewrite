use crate::client::Session;
use crate::stats::stat_file_writer::{StatFileWriter, StatMap};
use game::port::text;
use jvm::string::JString;
use jvm::throwable::{EXCEPTION, Throwable};
use std::cell::{Cell, RefCell};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, PoisonError};
use std::thread::JoinHandle;

fn renameTo(from: &Path, to: &Path) { let _renamed: bool = std::fs::rename(from, to).is_ok(); }

#[derive(Clone)]
struct StatFiles
{
    m_file: PathBuf,
    m_tempFile: PathBuf,
    m_oldFile: PathBuf,
}

struct Shared
{
    m_busy: AtomicBool,
    m_stats: Mutex<Option<String>>,
}

pub struct StatsSyncher
{
    m_shared: Arc<Shared>,
    m_unsentStats: RefCell<Option<StatMap>>,
    m_unsent: StatFiles,
    m_stats: StatFiles,
    m_username: Option<String>,
    m_saveCooldown: Cell<i32>,
    m_saveUnsentCooldown: Cell<i32>,
}

fn renameFile(dir: &Path, name: &str, to: &Path)
{
    let file: PathBuf = dir.join(JString::from(name).toOsString());
    if file.exists() && !file.is_dir() && !to.exists()
    {
        renameTo(&file, to);
    }
}

fn pickStatsFile(files: &StatFiles) -> Option<&PathBuf>
{
    [&files.m_file, &files.m_oldFile, &files.m_tempFile].into_iter().find(|file| file.exists())
}

fn readStatsText(file: &Path) -> Option<String>
{
    match std::fs::read(file)
    {
        // port helpers decode in the platform charset and split lines like FileReader with readLine
        Ok(bytes) => Some(text::readLines(&text::decodePlatform(&bytes)).concat()),
        Err(error) =>
        {
            jvm::io::fromStd(&error).printStackTrace();
            None
        }
    }
}

fn deserializeCaught(text: &str) -> Option<StatMap>
{
    match jvm::throwable::catchThrowable(&EXCEPTION, || StatFileWriter::deserialize(text))
    {
        Ok(stats) => stats,
        Err(exception) =>
        {
            exception.printStackTrace();
            None
        }
    }
}

fn loadStats(files: &StatFiles) -> Option<StatMap>
{
    let text: String = readStatsText(pickStatsFile(files)?)?;
    deserializeCaught(&text)
}

fn saveText(text: &str, files: &StatFiles) -> Result<(), Throwable>
{
    let mut file: std::fs::File = std::fs::File::create(&files.m_tempFile).map_err(|error| jvm::io::fromStd(&error))?;
    let _ = std::io::Write::write_all(&mut file, &JString::from(text).getBytes());
    drop(file);
    if files.m_oldFile.exists()
    {
        let _deleted: bool = std::fs::remove_file(&files.m_oldFile).is_ok();
    }
    if files.m_file.exists()
    {
        renameTo(&files.m_file, &files.m_oldFile);
    }
    renameTo(&files.m_tempFile, &files.m_file);
    Ok(())
}

impl StatsSyncher
{
    pub fn new(session: &Rc<Session>, stats: &StatFileWriter, dir: &Path) -> StatsSyncher
    {
        let username: Option<String> = session.m_username.get();
        let name: String = username.clone().unwrap_or_else(|| jvm::throwable::npe());
        let lower: JString = JString::from(name.as_str()).toLowerCase();
        let path = |suffix: &str| {
            let mut fileName: Vec<u16> = JString::from("stats_").chars().to_vec();
            fileName.extend_from_slice(lower.chars());
            fileName.extend(suffix.encode_utf16());
            dir.join(JString::fromUtf16(fileName).toOsString())
        };
        let unsent: StatFiles =
            StatFiles { m_file: path("_unsent.dat"), m_tempFile: path("_unsent.tmp"), m_oldFile: path("_unsent.old") };
        let all: StatFiles = StatFiles { m_file: path(".dat"), m_tempFile: path(".tmp"), m_oldFile: path(".old") };
        if lower != JString::from(name.as_str())
        {
            renameFile(dir, &format!("stats_{name}_unsent.dat"), &unsent.m_file);
            renameFile(dir, &format!("stats_{name}.dat"), &all.m_file);
            renameFile(dir, &format!("stats_{name}_unsent.old"), &unsent.m_oldFile);
            renameFile(dir, &format!("stats_{name}.old"), &all.m_oldFile);
            renameFile(dir, &format!("stats_{name}_unsent.tmp"), &unsent.m_tempFile);
            renameFile(dir, &format!("stats_{name}.tmp"), &all.m_tempFile);
        }
        let syncher: StatsSyncher = StatsSyncher {
            m_shared: Arc::new(Shared { m_busy: AtomicBool::new(false), m_stats: Mutex::new(None) }),
            m_unsentStats: RefCell::new(None),
            m_unsent: unsent,
            m_stats: all,
            m_username: username,
            m_saveCooldown: Cell::new(0),
            m_saveUnsentCooldown: Cell::new(0),
        };
        if syncher.m_unsent.m_file.exists()
        {
            stats.increment(loadStats(&syncher.m_unsent));
        }
        syncher.load();
        syncher
    }

    fn startWorker(&self, body: impl FnOnce(&Shared) -> Result<(), Throwable> + Send + 'static)
    {
        let shared: Arc<Shared> = Arc::clone(&self.m_shared);
        let started: std::io::Result<JoinHandle<()>> = jvm::thread::start(jvm::thread::nextName(), move || {
            let result: Result<Result<(), Throwable>, Throwable> =
                jvm::throwable::catchThrowable(&EXCEPTION, || body(&shared));
            if let Err(exception) | Ok(Err(exception)) = result
            {
                exception.printStackTrace();
            }
            shared.m_busy.store(false, Ordering::SeqCst);
        });
        if let Err(error) = started
        {
            jvm::io::fromStd(&error).printStackTrace();
        }
    }

    pub fn load(&self)
    {
        if self.m_shared.m_busy.load(Ordering::SeqCst)
        {
            jvm::throwable::throw(
                &jvm::throwable::ILLEGAL_STATE,
                "Can't get stats from server while StatsSyncher is busy!",
            );
        }
        self.m_saveCooldown.set(100);
        self.m_shared.m_busy.store(true, Ordering::SeqCst);
        let files: StatFiles = self.m_stats.clone();
        self.startWorker(move |shared| {
            if files.m_file.exists()
                && let Some(file) = pickStatsFile(&files)
            {
                *shared.m_stats.lock().unwrap_or_else(PoisonError::into_inner) = readStatsText(file);
            }
            Ok(())
        });
    }

    pub fn save(&self, stats: &StatMap)
    {
        if self.m_shared.m_busy.load(Ordering::SeqCst)
        {
            jvm::throwable::throw(&jvm::throwable::ILLEGAL_STATE, "Can't save stats while StatsSyncher is busy!");
        }
        self.m_saveCooldown.set(100);
        self.m_shared.m_busy.store(true, Ordering::SeqCst);
        let text: String = StatFileWriter::serialize(self.m_username.as_deref(), Some("local"), stats);
        let files: StatFiles = self.m_unsent.clone();
        self.startWorker(move |_| saveText(&text, &files));
    }

    pub fn syncStatsFileWithMap(&self, stats: &StatMap)
    {
        let mut n: i32 = 30;
        while self.m_shared.m_busy.load(Ordering::SeqCst)
        {
            n -= 1;
            if n <= 0
            {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        self.m_shared.m_busy.store(true, Ordering::SeqCst);
        let text: String = StatFileWriter::serialize(self.m_username.as_deref(), Some("local"), stats);
        if let Err(exception) = saveText(&text, &self.m_unsent)
        {
            exception.printStackTrace();
        }
        self.m_shared.m_busy.store(false, Ordering::SeqCst);
    }

    pub fn canSave(&self) -> bool
    {
        self.m_saveCooldown.get() <= 0
            && !self.m_shared.m_busy.load(Ordering::SeqCst)
            && self.m_unsentStats.borrow().is_none()
    }

    pub fn tick(&self, statHandler: &StatFileWriter)
    {
        if self.m_saveCooldown.get() > 0
        {
            self.m_saveCooldown.set(self.m_saveCooldown.get() - 1);
        }
        if self.m_saveUnsentCooldown.get() > 0
        {
            self.m_saveUnsentCooldown.set(self.m_saveUnsentCooldown.get() - 1);
        }
        if let Some(unsentStats) = self.m_unsentStats.borrow_mut().take()
        {
            statHandler.tickUnsent(Some(unsentStats));
        }
        let text: Option<String> = self.m_shared.m_stats.lock().unwrap_or_else(PoisonError::into_inner).take();
        if let Some(text) = text
            && let Some(stats) = deserializeCaught(&text)
        {
            statHandler.tickStats(Some(stats));
        }
    }
}
