use crate::resources::{Location, Pack};

// lowest pack first so a higher pack overrides it
pub(crate) struct Stack
{
    m_packs: Vec<Pack>,
}

impl Stack
{
    pub(crate) fn new(packs: Vec<Pack>) -> Self { Self { m_packs: packs } }

    pub(crate) fn read(&self, location: &Location) -> Option<Vec<u8>>
    {
        self.m_packs.iter().rev().find_map(|pack| pack.read(location))
    }
}
