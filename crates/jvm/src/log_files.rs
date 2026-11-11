use crate::calendar::{DateFields, dateFields};
use crate::logging::{ConsoleHandler, Formatter, Handler, Level, LogRecord, Logger};
use flate2::Compression;
use flate2::write::GzEncoder;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard, PoisonError};
use std::time::{SystemTime, UNIX_EPOCH};

pub mod limits
{
    pub const ROLL_BYTES: u64 = 16 * 1024 * 1024;
    pub const KEEP_ARCHIVES: usize = 30;
}

const PLAIN_EXTENSION: &str = "log";
const ARCHIVE_EXTENSION: &str = "log.gz";

// the widest level name so messages line up in one column
pub const LEVEL_WIDTH: usize = 7;

// one program writes <dir>/<name>.log and keeps its old runs as <name>-<date>-<n>.log.gz
struct LogFile
{
    m_dir: PathBuf,
    m_name: &'static str,
    m_file: File,
    m_written: u64,
}

impl LogFile
{
    fn current(&self) -> PathBuf { currentPath(&self.m_dir, self.m_name) }
}

static FILE: Mutex<Option<LogFile>> = Mutex::new(None);

fn lock() -> MutexGuard<'static, Option<LogFile>> { FILE.lock().unwrap_or_else(PoisonError::into_inner) }

fn currentPath(dir: &Path, name: &str) -> PathBuf { dir.join(format!("{name}.{PLAIN_EXTENSION}")) }

// console plus file for a logger, the console shows info and up while the file keeps every level
pub fn install(logger: &Logger, dir: &Path, name: &'static str, formatter: fn() -> Box<dyn Formatter>)
{
    logger.setUseParentHandlers(false);
    logger.addHandler(Box::new(ConsoleHandler::new(formatter())));

    match open(dir, name)
    {
        Ok(()) => logger.addHandler(Box::new(FileLogHandler { m_formatter: formatter() })),
        Err(error) => logger.log(
            Level::WARNING,
            &format!("Failed to log to {}", currentPath(dir, name).display()),
            Some(crate::io::fromStd(&error)),
        ),
    }
}

fn open(dir: &Path, name: &'static str) -> io::Result<()>
{
    fs::create_dir_all(dir)?;

    let current: PathBuf = currentPath(dir, name);
    if current.exists()
    {
        compress(&stage(dir, name, &current)?)?;
    }

    compressStrays(dir, name);
    prune(dir, name);

    let file: File = File::create(&current)?;
    *lock() = Some(LogFile { m_dir: dir.to_path_buf(), m_name: name, m_file: file, m_written: 0 });

    let previous: Box<dyn Fn(&std::panic::PanicHookInfo<'_>) + Sync + Send + 'static> = std::panic::take_hook();

    std::panic::set_hook(Box::new(move |info|
    {
        // thrown java exceptions unwind as panics and are reported where they are caught
        if !crate::throwable::standsForJava(info.payload())
        {
            let thread: String = std::thread::current().name().unwrap_or("unnamed").to_owned();
            write(&format!("thread \"{thread}\" {info}\n"));
        }

        previous(info);
    }));

    Ok(())
}

// raw text into the open log file, a no op before install
pub fn write(text: &str)
{
    let mut guard: MutexGuard<'_, Option<LogFile>> = lock();
    let Some(log) = guard.as_mut()
    else
    {
        return;
    };

    if let Err(error) = log.m_file.write_all(text.as_bytes())
    {
        eprintln!("log write failed: {error}");
        return;
    }

    log.m_written += text.len() as u64;
    if log.m_written >= limits::ROLL_BYTES
    {
        roll(log);
    }
}

// a full log is renamed and then compressed off the calling thread
fn roll(log: &mut LogFile)
{
    let staged: PathBuf = match stage(&log.m_dir, log.m_name, &log.current())
    {
        Ok(staged) => staged,

        Err(error) =>
        {
            eprintln!("log roll failed: {error}");
            return;
        }
    };

    match File::create(log.current())
    {
        Ok(file) =>
        {
            log.m_file = file;
            log.m_written = 0;
        }

        Err(error) => eprintln!("log reopen failed: {error}"),
    }

    let dir: PathBuf = log.m_dir.clone();
    let name: &'static str = log.m_name;
    let spawned: io::Result<std::thread::JoinHandle<()>> =
        std::thread::Builder::new().name("Log compressor".to_owned()).spawn(move ||
        {
            if let Err(error) = compress(&staged)
            {
                eprintln!("log compress failed for {}: {error}", staged.display());
            }

            prune(&dir, name);
        });

    if let Err(error) = spawned
    {
        eprintln!("log compressor did not start: {error}");
    }
}

fn stage(dir: &Path, name: &str, current: &Path) -> io::Result<PathBuf>
{
    let modified: SystemTime = fs::metadata(current)?.modified()?;
    let millis: i64 = modified.duration_since(UNIX_EPOCH).map_or(0, |elapsed| elapsed.as_millis() as i64);
    let fields: DateFields = dateFields(millis);
    let date: String = format!("{:04}-{:02}-{:02}", fields.m_year, fields.m_month, fields.m_day);
    let mut index: u32 = 1;

    loop
    {
        let stem: String = format!("{name}-{date}-{index}");
        let staged: PathBuf = dir.join(format!("{stem}.{PLAIN_EXTENSION}"));
        if !staged.exists() && !dir.join(format!("{stem}.{ARCHIVE_EXTENSION}")).exists()
        {
            fs::rename(current, &staged)?;
            return Ok(staged);
        }

        index += 1;
    }
}

fn compress(plain: &Path) -> io::Result<()>
{
    let mut source: File = File::open(plain)?;
    let target: File = File::create(plain.with_extension(ARCHIVE_EXTENSION))?;
    let mut encoder: GzEncoder<File> = GzEncoder::new(target, Compression::default());
    io::copy(&mut source, &mut encoder)?;
    encoder.finish()?.sync_all()?;
    fs::remove_file(plain)
}

// a run that stopped mid compress leaves its staged plain log behind
fn compressStrays(dir: &Path, name: &str)
{
    let prefix: String = format!("{name}-");
    let suffix: String = format!(".{PLAIN_EXTENSION}");
    for path in listDir(dir).into_iter().filter(|path| hasNameParts(path, &prefix, &suffix))
    {
        if let Err(error) = compress(&path)
        {
            eprintln!("log compress failed for {}: {error}", path.display());
        }
    }
}

fn prune(dir: &Path, name: &str)
{
    let prefix: String = format!("{name}-");
    let suffix: String = format!(".{ARCHIVE_EXTENSION}");
    let mut archives: Vec<(SystemTime, PathBuf)> = listDir(dir)
        .into_iter()
        .filter(|path| hasNameParts(path, &prefix, &suffix))
        .filter_map(|path| Some((fs::metadata(&path).ok()?.modified().ok()?, path)))
        .collect();
    archives.sort_by_key(|archive| std::cmp::Reverse(archive.0));

    for (_, path) in archives.iter().skip(limits::KEEP_ARCHIVES)
    {
        if let Err(error) = fs::remove_file(path)
        {
            eprintln!("could not remove old log {}: {error}", path.display());
        }
    }
}

fn hasNameParts(path: &Path, prefix: &str, suffix: &str) -> bool
{
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.starts_with(prefix) && name.ends_with(suffix))
}

fn listDir(dir: &Path) -> Vec<PathBuf>
{
    match fs::read_dir(dir)
    {
        Ok(entries) => entries.filter_map(|entry| entry.ok().map(|entry| entry.path())).collect(),

        Err(error) =>
        {
            eprintln!("could not list {}: {error}", dir.display());
            Vec::new()
        }
    }
}

struct FileLogHandler
{
    m_formatter: Box<dyn Formatter>,
}

impl Handler for FileLogHandler
{
    fn publish(&mut self, record: &LogRecord) { write(&self.m_formatter.format(record)); }

    fn flush(&mut self) {}

    fn close(&mut self) {}
}

// date with milliseconds, level and message
pub struct LogFormatter;

impl LogFormatter
{
    pub fn boxed() -> Box<dyn Formatter> { Box::new(Self) }
}

impl Formatter for LogFormatter
{
    fn format(&self, record: &LogRecord) -> String
    {
        let fields: DateFields = dateFields(record.getMillis());
        let mut text: String = format!(
            "{:04}-{:02}-{:02} {:02}:{:02}:{:02}.{:03} {:<LEVEL_WIDTH$} [{}] {}\n",
            fields.m_year,
            fields.m_month,
            fields.m_day,
            fields.m_hour,
            fields.m_minute,
            fields.m_second,
            record.getMillis().rem_euclid(1000),
            record.getLevel().getLocalizedName(),
            record.source(),
            record.getMessage()
        );

        if let Some(thrown) = record.getThrown()
        {
            text.push_str(&format!("{thrown}\n"));
        }

        text
    }
}
