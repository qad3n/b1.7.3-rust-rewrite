use crate::resources::{Location, Pack};
use std::rc::Rc;

// lowest pack first so a higher pack overrides it
pub(crate) struct Stack
{
    m_packs: Vec<Rc<Pack>>,
}

impl Stack
{
    pub(crate) fn new(packs: Vec<Rc<Pack>>) -> Self { Self { m_packs: packs } }

    pub(crate) fn packs(&self) -> &[Rc<Pack>] { &self.m_packs }

    pub(crate) fn read(&self, location: &Location) -> Option<Vec<u8>>
    {
        self.m_packs.iter().rev().find_map(|pack| pack.read(location))
    }
}
