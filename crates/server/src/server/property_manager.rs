use jvm::io::{FileInputStream, FileOutputStream, IoResult};
use jvm::logging::Level;
use jvm::properties::Properties;
use jvm::throwable::{self, Throwable};
use jvm::{info, logThrown, warning};
use std::cell::RefCell;
use std::path::{Path, PathBuf};

pub struct PropertyManager
{
    m_serverProperties: RefCell<Properties>,
    m_serverPropertiesFile: PathBuf,
}

impl PropertyManager
{
    pub fn new(file: &Path) -> PropertyManager
    {
        let manager: PropertyManager = PropertyManager {
            m_serverProperties: RefCell::new(Properties::new()),
            m_serverPropertiesFile: file.to_path_buf(),
        };
        if game::port::file::exists(file)
        {
            let result: Result<IoResult<()>, Throwable> = throwable::catchThrowable(&throwable::EXCEPTION, || {
                manager.m_serverProperties.borrow_mut().load(&mut FileInputStream::open(file)?)
            });
            if let Err(exception) = result.and_then(|inner| inner)
            {
                logThrown!(Level::WARNING, Some(exception), "Failed to load {}", file.display());
                manager.generateNewProperties();
            }
        }
        else
        {
            warning!("{} does not exist", file.display());
            manager.generateNewProperties();
        }
        manager
    }

    pub fn generateNewProperties(&self)
    {
        info!("Generating new properties file");
        self.saveProperties();
    }

    pub fn saveProperties(&self)
    {
        let result: Result<IoResult<()>, Throwable> = throwable::catchThrowable(&throwable::EXCEPTION, || {
            self.m_serverProperties.borrow().store(
                &mut FileOutputStream::create(&self.m_serverPropertiesFile, false)?,
                "Minecraft server properties",
            )
        });
        if let Err(exception) = result.and_then(|inner| inner)
        {
            logThrown!(Level::WARNING, Some(exception), "Failed to save {}", self.m_serverPropertiesFile.display());
            self.generateNewProperties();
        }
    }

    pub fn getStringProperty(&self, key: &str, defaultValue: &str) -> String
    {
        if !self.m_serverProperties.borrow().containsKey(key)
        {
            self.m_serverProperties.borrow_mut().setProperty(key, defaultValue);
            self.saveProperties();
        }
        self.m_serverProperties.borrow().getProperty(key, defaultValue)
    }

    pub fn getIntProperty(&self, key: &str, defaultValue: i32) -> i32
    {
        let value: String = self.getStringProperty(key, &defaultValue.to_string());
        match game::port::parseInt(&value)
        {
            Ok(value) => value,
            Err(_) =>
            {
                self.m_serverProperties.borrow_mut().setProperty(key, &defaultValue.to_string());
                defaultValue
            }
        }
    }

    pub fn getBooleanProperty(&self, key: &str, defaultValue: bool) -> bool
    {
        let value: String = self.getStringProperty(key, &defaultValue.to_string());
        value.eq_ignore_ascii_case("true")
    }

    pub fn setProperty(&self, key: &str, value: bool)
    {
        self.m_serverProperties.borrow_mut().setProperty(key, &value.to_string());
        self.saveProperties();
    }
}
