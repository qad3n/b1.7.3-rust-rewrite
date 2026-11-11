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

    // copies of merged files lowest pack first and in the given order within a pack
    // a filter keeps the copy of its own pack and drops the copies below it
    pub(crate) fn readEach(&self, locations: &[Location]) -> Vec<(usize, Vec<u8>)>
    {
        let lowest: Vec<usize> = locations
            .iter()
            .map(|location| self.m_packs.iter().rposition(|pack| pack.blocks(location)).unwrap_or(0))
            .collect();
        let mut copies: Vec<(usize, Vec<u8>)> = Vec::new();

        for (index, pack) in self.m_packs.iter().enumerate()
        {
            for (which, location) in locations.iter().enumerate().filter(|(which, _)| index >= lowest[*which])
            {
                if let Some(bytes) = pack.read(location)
                {
                    copies.push((which, bytes));
                }
            }
        }

        copies
    }

    pub(crate) fn disk(&self, location: &Location) -> Option<(String, PathBuf)>
    {
        let pack: &Rc<Pack> = &self.m_packs[self.owner(location)?];
        Some((pack.name().to_owned(), pack.disk(location)?))
    }
}
