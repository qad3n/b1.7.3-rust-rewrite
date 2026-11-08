use jvm::calendar::{DateFields, dateFields};
use jvm::logging::{ConsoleHandler, FileHandler, Formatter, Level, LogRecord, Logger};
use std::path::Path;

fn logger() -> &'static Logger { Logger::getLogger("Minecraft") }

pub struct ConsoleLogManager;

impl ConsoleLogManager
{
    pub fn init()
    {
        logger().setUseParentHandlers(false);
        logger().addHandler(Box::new(ConsoleHandler::new(Box::new(ConsoleLogFormatter::new()))));
        match FileHandler::new(Path::new("server.log"), true, Box::new(ConsoleLogFormatter::new()))
        {
            Ok(fileHandler) => logger().addHandler(Box::new(fileHandler)),
            Err(exception) => logger().log(Level::WARNING, "Failed to log to server.log", Some(exception)),
        }
    }
}

pub struct ConsoleLogFormatter;

impl ConsoleLogFormatter
{
    fn new() -> ConsoleLogFormatter { ConsoleLogFormatter }

    fn formatDate(millis: i64) -> String
    {
        let fields: DateFields = dateFields(millis);
        format!(
            "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
            fields.m_year, fields.m_month, fields.m_day, fields.m_hour, fields.m_minute, fields.m_second
        )
    }
}

impl Formatter for ConsoleLogFormatter
{
    fn format(&self, logRecord: &LogRecord) -> String
    {
        let mut stringBuilder: String = String::new();
        stringBuilder.push_str(&ConsoleLogFormatter::formatDate(logRecord.getMillis()));
        let level: Level = logRecord.getLevel();
        if level == Level::FINEST
        {
            stringBuilder.push_str(" [FINEST] ");
        }
        else if level == Level::FINER
        {
            stringBuilder.push_str(" [FINER] ");
        }
        else if level == Level::FINE
        {
            stringBuilder.push_str(" [FINE] ");
        }
        else if level == Level::INFO
        {
            stringBuilder.push_str(" [INFO] ");
        }
        else if level == Level::WARNING
        {
            stringBuilder.push_str(" [WARNING] ");
        }
        else if level == Level::SEVERE
        {
            stringBuilder.push_str(" [SEVERE] ");
        }
        else if level == Level::SEVERE
        {
            stringBuilder.push_str(&format!(" [{}] ", level.getLocalizedName()));
        }
        stringBuilder.push_str(&logRecord.getMessage().to_string());
        stringBuilder.push('\n');
        if let Some(throwable) = logRecord.getThrown()
        {
            stringBuilder.push_str(&format!("{throwable}\n"));
        }
        stringBuilder
    }
}
