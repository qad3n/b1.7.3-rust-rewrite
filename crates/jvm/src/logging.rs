use crate::io::{FileOutputStream, IoResult, OutputStream};
use crate::string::JString;
use crate::throwable::Throwable;
use std::io::IsTerminal;
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
    pub const FINE: Level = Level { m_name: "FINE", m_value: 500, m_color: colors::GREY };
    pub const FINER: Level = Level { m_name: "FINER", m_value: 400, m_color: colors::GREY };
    pub const FINEST: Level = Level { m_name: "FINEST", m_value: 300, m_color: colors::GREY };

    pub fn getLocalizedName(&self) -> &'static str { self.m_name }

    pub fn intValue(&self) -> i32 { self.m_value }
}

pub struct LogRecord
{
    m_level: Level,
    m_message: JString,
    m_millis: i64,
    m_thrown: Option<Throwable>,
}

impl LogRecord
{
    pub fn getLevel(&self) -> Level { self.m_level }

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

    pub fn log(&self, level: Level, message: &str, thrown: Option<Throwable>)
    {
        let record: LogRecord = LogRecord {
            m_level: level,
            m_message: JString::from(message),
            m_millis: crate::system::currentTimeMillis(),
            m_thrown: thrown,
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

    pub fn info(&self, message: &str) { self.log(Level::INFO, message, None); }

    pub fn warning(&self, message: &str) { self.log(Level::WARNING, message, None); }

    pub fn severe(&self, message: &str) { self.log(Level::SEVERE, message, None); }

    pub fn fine(&self, message: &str) { self.log(Level::FINE, message, None); }

    pub fn debug(&self, message: &str) { self.fine(message); }
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
            log::Level::Debug | log::Level::Trace => Level::FINE,
        };
        let source: &str = record.target().split("::").next().unwrap_or_default();
        self.m_logger.log(level, &format!("[{source}] {}", record.args()), None);
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
        return Level::FINE;
    };

    match name.to_string_lossy().to_ascii_lowercase().as_str()
    {
        "error" | "severe" => Level::SEVERE,
        "warn" | "warning" => Level::WARNING,
        "info" => Level::INFO,
        "debug" | "fine" => Level::FINE,
        "trace" | "finest" => Level::FINEST,
        other =>
        {
            eprintln!("unknown {CONSOLE_LEVEL_VAR} level {other}, showing debug and up");
            Level::FINE
        }
    }
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
