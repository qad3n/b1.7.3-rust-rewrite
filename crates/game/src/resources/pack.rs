use crate::resources::{Location, Source, legacy};
use std::cell::RefCell;

// a beta layout pack whose files are found through the legacy table
pub(crate) struct Pack
{
    m_source: RefCell<Source>,
}

impl Pack
{
    pub(crate) fn new(source: Source) -> Self { Self { m_source: RefCell::new(source) } }

    pub(crate) fn read(&self, location: &Location) -> Option<Vec<u8>>
    {
        let path: &str = legacy::path(location)?;
        self.m_source.borrow_mut().read(path)
    }
}
