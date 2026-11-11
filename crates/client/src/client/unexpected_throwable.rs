use jvm::throwable::Throwable;

pub struct UnexpectedThrowable
{
    pub m_description: String,
    pub m_exception: Throwable,
    // game and mod loader state when it crashed
    pub m_details: String,
}

impl UnexpectedThrowable
{
    pub fn new(description: String, exception: Throwable) -> Self
    {
        Self { m_description: description, m_exception: exception, m_details: String::new() }
    }

    pub fn withDetails(self, details: String) -> Self { Self { m_details: details, ..self } }
}
