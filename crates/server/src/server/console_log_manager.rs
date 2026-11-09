use jvm::calendar::{DateFields, dateFields};
use jvm::log_files;
use jvm::logging::{Formatter, Level, LogRecord, logger};
use std::path::Path;

pub struct ConsoleLogManager;

impl ConsoleLogManager
{
    pub fn init()
    {
        // rewrite: logs/server.log with gzip backups of earlier runs replaces the ever growing server.log
        log_files::install(logger(), Path::new("logs"), "server", || Box::new(ConsoleLogFormatter::new()));
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
