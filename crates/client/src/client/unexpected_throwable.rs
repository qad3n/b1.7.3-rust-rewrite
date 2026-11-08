use jvm::throwable::Throwable;

pub struct UnexpectedThrowable
{
    pub m_description: String,
    pub m_exception: Throwable,
}

impl UnexpectedThrowable
{
    pub fn new(description: String, exception: Throwable) -> UnexpectedThrowable
    {
        UnexpectedThrowable { m_description: description, m_exception: exception }
    }
}
