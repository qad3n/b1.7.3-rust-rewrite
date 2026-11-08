use crate::io::{FileOutputStream, IoResult, OutputStream};
use crate::string::JString;
use crate::throwable::Throwable;
use std::path::Path;
use std::sync::{Mutex, MutexGuard, PoisonError};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Level
{
    pub m_name: &'static str,
    pub m_value: i32,
}

impl Level
{
    pub const SEVERE: Level = Level { m_name: "SEVERE", m_value: 1000 };
    pub const WARNING: Level = Level { m_name: "WARNING", m_value: 900 };
    pub const INFO: Level = Level { m_name: "INFO", m_value: 800 };
    pub const CONFIG: Level = Level { m_name: "CONFIG", m_value: 700 };
    pub const FINE: Level = Level { m_name: "FINE", m_value: 500 };
    pub const FINER: Level = Level { m_name: "FINER", m_value: 400 };
    pub const FINEST: Level = Level { m_name: "FINEST", m_value: 300 };

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
        ConsoleHandler { m_formatter: formatter, m_level: Level::INFO }
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
        eprint!("{}", self.m_formatter.format(record));
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
}
