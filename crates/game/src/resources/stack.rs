use crate::resources::{Location, Pack};
use std::path::PathBuf;
use std::rc::Rc;

// lowest pack first so a higher pack overrides it
// the jar then one pack per mod then the texture packs the player picked
pub(crate) struct Stack
{
    m_packs: Vec<Rc<Pack>>,
    // index of the first texture pack the player picked
    m_user: usize,
}

impl Stack
{
    pub(crate) fn new(packs: Vec<Rc<Pack>>, user: usize) -> Self { Self { m_packs: packs, m_user: user } }

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
    pub(crate) fn readEach(&self, locations: &[Location], user: bool) -> Vec<(usize, Vec<u8>)>
    {
        let first: usize = if user { self.m_user } else { 0 };
        let lowest: Vec<usize> = locations
            .iter()
            .map(|location| self.m_packs.iter().rposition(|pack| pack.blocks(location)).unwrap_or(0))
            .collect();
        let mut copies: Vec<(usize, Vec<u8>)> = Vec::new();

        for (index, pack) in self.m_packs.iter().enumerate().skip(first)
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

    // the file on disk for a location when a pack above the floor pack holds it
    pub(crate) fn disk(&self, location: &Location, floor: Option<&str>) -> Option<(String, PathBuf)>
    {
        let lowest: usize = floor
            .and_then(|name| self.m_packs[..self.m_user].iter().position(|pack| pack.name() == name))
            .map_or(0, |index| index + 1);
        let pack: &Rc<Pack> = &self.m_packs[self.owner(location).filter(|owner| *owner >= lowest)?];
        Some((pack.name().to_owned(), pack.disk(location)?))
    }

    // minecraft first then the namespace of every mod pack
    pub(crate) fn namespaces(&self) -> Vec<String>
    {
        let mods: &[Rc<Pack>] = self.m_packs.get(1..self.m_user).unwrap_or(&[]);
        std::iter::once(crate::resources::MINECRAFT.to_owned())
            .chain(mods.iter().map(|pack| pack.name().to_owned()))
            .collect()
    }
}
