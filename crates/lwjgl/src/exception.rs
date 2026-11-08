use std::fmt;

// org.lwjgl.LWJGLException, a checked exception the caller handles as a Result
#[derive(Debug, Clone)]
pub struct LWJGLException
{
    m_message: Option<String>,
}

impl LWJGLException
{
    pub fn new(message: impl Into<String>) -> LWJGLException { LWJGLException { m_message: Some(message.into()) } }

    pub fn getMessage(&self) -> Option<&str> { self.m_message.as_deref() }

    pub fn printStackTrace(&self) { jvm::system::errPrintln(&self.to_string()); }
}

impl fmt::Display for LWJGLException
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        match &self.m_message
        {
            Some(message) => write!(formatter, "org.lwjgl.LWJGLException: {message}"),
            None => write!(formatter, "org.lwjgl.LWJGLException"),
        }
    }
}

impl std::error::Error for LWJGLException {}
