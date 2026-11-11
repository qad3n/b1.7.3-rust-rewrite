use crate::resources::{Location, Pack};
use std::path::PathBuf;
use std::rc::Rc;

// lowest pack first so a higher pack overrides it
pub(crate) struct Stack
{
    m_packs: Vec<Rc<Pack>>,
}

impl Stack
{
    pub(crate) fn new(packs: Vec<Rc<Pack>>) -> Self { Self { m_packs: packs } }

    // the index of the top pack holding a location unless a filter above it hides the file
    pub(crate) fn owner(&self, location: &Location) -> Option<usize>
    {
        for (index, pack) in self.m_packs.iter().enumerate().rev()
        {
            if pack.has(location)
            {
                return Some(index);
            }

            if pack.blocks(location)
            {
                return None;
            }
        }

        None
    }

    pub(crate) fn pack(&self, index: usize) -> &Rc<Pack> { &self.m_packs[index] }

    pub(crate) fn read(&self, location: &Location) -> Option<Vec<u8>>
    {
        self.m_packs[self.owner(location)?].read(location)
    }

    // every copy of a merged file lowest first and a filter drops the copies below it
    pub(crate) fn readAll(&self, location: &Location) -> Vec<Vec<u8>>
    {
        let mut copies: Vec<Vec<u8>> = Vec::new();

        for pack in self.m_packs.iter().rev()
        {
            if let Some(bytes) = pack.read(location)
            {
                copies.push(bytes);
            }

            if pack.blocks(location)
            {
                break;
            }
        }

        copies.reverse();
        copies
    }

    pub(crate) fn disk(&self, location: &Location) -> Option<(String, PathBuf)>
    {
        let pack: &Rc<Pack> = &self.m_packs[self.owner(location)?];
        Some((pack.name().to_owned(), pack.disk(location)?))
    }
}
