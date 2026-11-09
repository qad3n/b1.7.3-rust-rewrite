use crate::io::{FileOutputStream, IoResult, OutputStream};
use crate::string::JString;
use crate::throwable::Throwable;
use std::io::IsTerminal;
use std::panic::Location;
use std::path::Path;
use std::sync::{Mutex, MutexGuard, OnceLock, PoisonError};
use std::time::{Duration, Instant};

pub mod colors
{
    pub const RED: &str = "\x1b[31m";
    pub const YELLOW: &str = "\x1b[33m";
    pub const GREY: &str = "\x1b[90m";
    pub const RESET: &str = "\x1b[0m";
    // info keeps the terminal's own foreground so light themes stay readable
    pub const PLAIN: &str = "";
}

// B173_LOG picks the lowest level the console shows, the log file keeps every level
const CONSOLE_LEVEL_VAR: &str = "B173_LOG";

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Level
{
    pub m_name: &'static str,
    pub m_value: i32,
    pub m_color: &'static str,
}

impl Level
{
    pub const SEVERE: Level = Level { m_name: "SEVERE", m_value: 1000, m_color: colors::RED };
    pub const WARNING: Level = Level { m_name: "WARNING", m_value: 900, m_color: colors::YELLOW };
    pub const INFO: Level = Level { m_name: "INFO", m_value: 800, m_color: colors::PLAIN };
    pub const CONFIG: Level = Level { m_name: "CONFIG", m_value: 700, m_color: colors::PLAIN };
    pub const DEBUG: Level = Level { m_name: "DEBUG", m_value: 500, m_color: colors::GREY };
    pub const TRACE: Level = Level { m_name: "TRACE", m_value: 400, m_color: colors::GREY };

    pub fn getLocalizedName(&self) -> &'static str { self.m_name }

    pub fn intValue(&self) -> i32 { self.m_value }
}

// where a record was logged from, game code by its caller and library code by the module the log crate names
#[derive(Clone, Copy)]
enum Site
{
    Code(&'static Location<'static>),
    Module(&'static str, u32),
}

pub struct LogRecord
{
    m_level: Level,
    m_message: JString,
    m_millis: i64,
    m_thrown: Option<Throwable>,
    m_site: Site,
}

impl LogRecord
{
    pub fn getLevel(&self) -> Level { self.m_level }

    pub fn source(&self) -> String
    {
        match self.m_site
        {
            Site::Code(location) => format!("{}:{}", modulePath(location.file()), location.line()),
            Site::Module(module, line) => format!("{module}:{line}"),
        }
    }

    pub fn getMessage(&self) -> &JString { &self.m_message }

    pub fn getMillis(&self) -> i64 { self.m_millis }

    pub fn getThrown(&self) -> Option<&Throwable> { self.m_thrown.as_ref() }
}

pub trait Formatter: Send
{
    fn format(&self, record: &LogRecord) -> String;
}

pub trait Handler: Send
{
    fn publish(&mut self, record: &LogRecord);

    fn flush(&mut self);

    fn close(&mut self);
}

pub struct ConsoleHandler
{
    m_formatter: Box<dyn Formatter>,
    m_level: Level,
}

impl ConsoleHandler
{
    pub fn new(formatter: Box<dyn Formatter>) -> ConsoleHandler
    {
        ConsoleHandler { m_formatter: formatter, m_level: consoleLevel() }
    }
}

impl Handler for ConsoleHandler
{
    fn publish(&mut self, record: &LogRecord)
    {
        if record.m_level.m_value < self.m_level.m_value
        {
            return;
        }
        eprint!("{}", paint(record.m_level.m_color, &self.m_formatter.format(record)));
    }

    fn flush(&mut self) {}

    fn close(&mut self) {}
}

pub struct FileHandler
{
    m_formatter: Box<dyn Formatter>,
    m_output: FileOutputStream,
}

impl FileHandler
{
    pub fn new(pattern: &Path, append: bool, formatter: Box<dyn Formatter>) -> IoResult<FileHandler>
    {
        Ok(FileHandler { m_formatter: formatter, m_output: FileOutputStream::create(pattern, append)? })
    }
}

impl Handler for FileHandler
{
    fn publish(&mut self, record: &LogRecord)
    {
        let text: String = self.m_formatter.format(record);
        let bytes: Vec<u8> = JString::from(text.as_str()).getBytes();
        let written: IoResult<()> = self.m_output.writeFrom(&bytes, 0, bytes.len() as i32);
        reportError(written.and_then(|()| self.m_output.flush()));
    }

    fn flush(&mut self) { reportError(self.m_output.flush()); }

    fn close(&mut self) { reportError(self.m_output.close()); }
}

fn reportError(result: IoResult<()>)
{
    if let Err(error) = result
    {
        crate::system::errPrintln("java.util.logging.ErrorManager: 2");
        error.printStackTrace();
    }
}

struct LoggerState
{
    m_handlers: Vec<Box<dyn Handler>>,
    m_useParentHandlers: bool,
}

pub struct Logger
{
    m_name: &'static str,
    m_state: Mutex<LoggerState>,
}

static MINECRAFT: Logger = Logger {
    m_name: "Minecraft",
    m_state: Mutex::new(LoggerState { m_handlers: Vec::new(), m_useParentHandlers: true }),
};

impl Logger
{
    pub fn getLogger(name: &str) -> &'static Logger
    {
        match name
        {
            "Minecraft" => &MINECRAFT,
            _ => panic!("no logger named {name}"),
        }
    }

    fn state(&self) -> MutexGuard<'_, LoggerState> { self.m_state.lock().unwrap_or_else(PoisonError::into_inner) }

    pub fn getName(&self) -> &'static str { self.m_name }

    pub fn setUseParentHandlers(&self, useParentHandlers: bool)
    {
        self.state().m_useParentHandlers = useParentHandlers;
    }

    pub fn addHandler(&self, handler: Box<dyn Handler>) { self.state().m_handlers.push(handler); }

    #[track_caller]
    pub fn log(&self, level: Level, message: &str, thrown: Option<Throwable>)
    {
        self.publish(level, message, thrown, Site::Code(Location::caller()));
    }

    fn publish(&self, level: Level, message: &str, thrown: Option<Throwable>, site: Site)
    {
        let record: LogRecord = LogRecord {
            m_level: level,
            m_message: JString::from(message),
            m_millis: crate::system::currentTimeMillis(),
            m_thrown: thrown,
            m_site: site,
        };
        let mut state: MutexGuard<'_, LoggerState> = self.state();
        for handler in state.m_handlers.iter_mut()
        {
            handler.publish(&record);
        }
        if state.m_useParentHandlers && record.m_level.m_value >= Level::INFO.m_value
        {
            eprintln!("{}: {}", record.m_level.m_name, message);
            if let Some(thrown) = &record.m_thrown
            {
                thrown.printStackTrace();
            }
        }
    }

    #[track_caller]
    pub fn info(&self, message: &str) { self.log(Level::INFO, message, None); }

    #[track_caller]
    pub fn warning(&self, message: &str) { self.log(Level::WARNING, message, None); }

    #[track_caller]
    pub fn severe(&self, message: &str) { self.log(Level::SEVERE, message, None); }

    #[track_caller]
    pub fn debug(&self, message: &str) { self.log(Level::DEBUG, message, None); }

    #[track_caller]
    pub fn trace(&self, message: &str) { self.log(Level::TRACE, message, None); }
}

// libraries that log through the log crate land in a named logger, only targets listed here show below warnings
struct Bridge
{
    m_logger: &'static Logger,
    m_targets: &'static [&'static str],
}

impl log::Log for Bridge
{
    fn enabled(&self, metadata: &log::Metadata<'_>) -> bool
    {
        metadata.level() <= log::Level::Warn
            || self.m_targets.iter().any(|target| metadata.target().starts_with(target))
    }

    fn log(&self, record: &log::Record<'_>)
    {
        if !self.enabled(record.metadata())
        {
            return;
        }

        let level: Level = match record.level()
        {
            log::Level::Error => Level::SEVERE,
            log::Level::Warn => Level::WARNING,
            log::Level::Info => Level::INFO,
            log::Level::Debug | log::Level::Trace => Level::DEBUG,
        };
        let module: &'static str = record.module_path_static().unwrap_or("unknown");
        let site: Site = Site::Module(module, record.line().unwrap_or_default());
        self.m_logger.publish(level, &record.args().to_string(), None, site);
    }

    fn flush(&self) {}
}

// the one game logger, every crate logs through it
pub fn logger() -> &'static Logger { &MINECRAFT }

pub fn bridge(logger: &'static Logger, targets: &'static [&'static str])
{
    static BRIDGE: OnceLock<Bridge> = OnceLock::new();
    let bridge: &'static Bridge = BRIDGE.get_or_init(|| Bridge { m_logger: logger, m_targets: targets });
    match log::set_logger(bridge)
    {
        // trace stays off so cranelift never builds its per instruction records
        Ok(()) => log::set_max_level(log::LevelFilter::Debug),
        Err(error) => logger.warning(&format!("library logging stays off: {error}")),
    }
}

fn consoleLevel() -> Level
{
    let Some(name) = std::env::var_os(CONSOLE_LEVEL_VAR)
    else
    {
        return Level::DEBUG;
    };

    match name.to_string_lossy().to_ascii_lowercase().as_str()
    {
        "error" | "severe" => Level::SEVERE,
        "warn" | "warning" => Level::WARNING,
        "info" => Level::INFO,
        "debug" | "fine" => Level::DEBUG,
        "trace" | "finest" => Level::TRACE,
        other =>
        {
            eprintln!("unknown {CONSOLE_LEVEL_VAR} level {other}, showing debug and up");
            Level::DEBUG
        }
    }
}

// a call site file like crates/client/src/client/minecraft/run.rs reads as client::client::minecraft::run
fn modulePath(file: &str) -> String
{
    let file: String = file.replace('\\', "/");
    let Some((root, module)) = file.rsplit_once("/src/")
    else
    {
        return file;
    };

    let crateName: &str = root.rsplit('/').next().unwrap_or(root);
    let module: &str = module.trim_end_matches(".rs");
    let module: &str = module.strip_suffix("/mod").unwrap_or(module);
    let mut path: String = crateName.replace('-', "_");
    if module != "lib" && module != "main"
    {
        path += "::";
        path += &module.replace('/', "::");
    }
    path
}

// ansi colour only for a real terminal, piped output and NO_COLOR stay plain
pub fn paint(color: &str, text: &str) -> String
{
    static COLORED: OnceLock<bool> = OnceLock::new();
    let colored: bool =
        *COLORED.get_or_init(|| std::io::stderr().is_terminal() && std::env::var_os("NO_COLOR").is_none());
    if !colored || color.is_empty()
    {
        return text.to_owned();
    }

    let body: &str = text.trim_end_matches('\n');
    let newlines: &str = &text[body.len()..];
    format!("{color}{body}{}{newlines}", colors::RESET)
}

// load timing for log lines, milliseconds under a second and seconds above
pub fn took(started: Instant) -> String
{
    let elapsed: Duration = started.elapsed();
    match elapsed.as_millis()
    {
        millis @ 0..1000 => format!("{millis}ms"),
        _ => format!("{:.2}s", elapsed.as_secs_f64()),
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn module_path_reads_crate_and_modules()
    {
        assert_eq!(modulePath("crates/client/src/client/minecraft/run.rs"), "client::client::minecraft::run");
        assert_eq!(modulePath("crates/game/src/item/item/mod.rs"), "game::item::item");
        assert_eq!(modulePath("vendor/b1.7.3-modding-sdk/b173_api/src/lib.rs"), "b173_api");
        assert_eq!(modulePath("crates\\client\\src\\bin\\beta173.rs"), "client::bin::beta173");
    }
}
