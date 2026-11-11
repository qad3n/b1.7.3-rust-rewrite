mod report;
mod tables;
mod vanilla;

use mod_protocol::Identifier;
pub use report::report;
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, HashMap};
pub use tables::
{
    BiomeEntry, BlockEntry, DimensionEntry, EntityEntry, FuelEntry, ItemEntry, PacketEntry, RecipeEntry, Registries,
    SYNCED_ORDER, TileEntityEntry,
};
pub(crate) use vanilla::{BIOMES, BiomeRow, OVERWORLD_BIOMES};

#[derive(Debug, thiserror::Error, Clone, PartialEq)]
pub enum RegistryError
{
    #[error("{0} registry is frozen, {1} was registered too late")]
    Frozen(&'static str, Identifier),
    #[error("{0} is already registered in the {1} registry by {2}")]
    Duplicate(Identifier, &'static str, String),
    #[error("{0} is not in the {1} registry")]
    Missing(Identifier, &'static str),
    #[error("{0} in the {1} registry was already replaced by {2}, {3} cannot replace it again")]
    DoubleReplace(Identifier, &'static str, String, String),
    #[error("the {0} registry has no free numeric ids left for {1}")]
    Exhausted(&'static str, Identifier),
}

pub const UNASSIGNED: i32 = -1;

pub struct Row<T>
{
    pub m_id: Identifier,
    pub m_numeric: i32,
    pub m_owner: String,
    pub m_replacedBy: Option<String>,
    pub m_value: T,
}

pub struct Registry<T>
{
    m_name: &'static str,
    m_rows: Vec<Row<T>>,
    m_byId: HashMap<Identifier, usize>,
    m_byNumeric: HashMap<i32, usize>,
    m_frozen: bool,
}

impl<T> Registry<T>
{
    pub fn new(name: &'static str) -> Self
    {
        Self { m_name: name, m_rows: Vec::new(), m_byId: HashMap::new(), m_byNumeric: HashMap::new(), m_frozen: false }
    }

    pub fn name(&self) -> &'static str { self.m_name }

    pub fn seed(&mut self, id: Identifier, numeric: i32, value: T)
    {
        let owner: String = id.namespace().to_owned();
        self.insert(Row { m_id: id, m_numeric: numeric, m_owner: owner, m_replacedBy: None, m_value: value });
    }

    fn insert(&mut self, row: Row<T>)
    {
        let index: usize = self.m_rows.len();
        self.m_byId.insert(row.m_id.clone(), index);

        if row.m_numeric != UNASSIGNED
        {
            self.m_byNumeric.insert(row.m_numeric, index);
        }

        self.m_rows.push(row);
    }

    pub fn register(&mut self, id: Identifier, owner: &str, value: T) -> Result<(), RegistryError>
    {
        if self.m_frozen
        {
            return Err(RegistryError::Frozen(self.m_name, id));
        }
        if let Some(existing) = self.get(&id)
        {
            return Err(RegistryError::Duplicate(id, self.m_name, existing.m_owner.clone()));
        }

        self.insert(Row
        {
            m_id: id,
            m_numeric: UNASSIGNED,
            m_owner: owner.to_owned(),
            m_replacedBy: None,
            m_value: value,
        });

        Ok(())
    }

    // one mod may replace an entry it does not own, a second replacement is a load error naming both
    pub fn replace(&mut self, id: &Identifier, owner: &str, value: T) -> Result<(), RegistryError>
    {
        if self.m_frozen
        {
            return Err(RegistryError::Frozen(self.m_name, id.clone()));
        }
        let name: &'static str = self.m_name;
        let row: &mut Row<T> = self.getMut(id).ok_or_else(|| RegistryError::Missing(id.clone(), name))?;
        if let Some(previous) = &row.m_replacedBy
        {
            return Err(RegistryError::DoubleReplace(id.clone(), name, previous.clone(), owner.to_owned()));
        }

        row.m_replacedBy = Some(owner.to_owned());
        row.m_value = value;
        Ok(())
    }

    pub fn get(&self, id: &Identifier) -> Option<&Row<T>> { self.m_byId.get(id).map(|index| &self.m_rows[*index]) }

    pub fn getMut(&mut self, id: &Identifier) -> Option<&mut Row<T>>
    {
        self.m_byId.get(id).map(|index| &mut self.m_rows[*index])
    }

    pub fn byNumeric(&self, numeric: i32) -> Option<&Row<T>>
    {
        self.m_byNumeric.get(&numeric).map(|index| &self.m_rows[*index])
    }

    pub fn numeric(&self, id: &Identifier) -> Option<i32>
    {
        self.get(id).map(|row| row.m_numeric).filter(|numeric| *numeric != UNASSIGNED)
    }

    pub fn rows(&self) -> impl Iterator<Item = &Row<T>> { self.m_rows.iter() }

    pub fn rowCount(&self) -> usize { self.m_rows.len() }

    pub fn isFrozen(&self) -> bool { self.m_frozen }

    pub fn modded(&self) -> impl Iterator<Item = &Row<T>> { self.m_rows.iter().filter(|row| !row.m_id.isVanilla()) }

    // fresh numeric ids go to unassigned rows in sorted identifier order so load order never changes them
    pub fn freeze(
        &mut self,
        mut allocate: impl FnMut(&Identifier, &dyn Fn(i32) -> bool) -> Option<i32>,
    ) -> Result<(), RegistryError>
    {
        let mut pending: Vec<usize> =
            (0..self.m_rows.len()).filter(|index| self.m_rows[*index].m_numeric == UNASSIGNED).collect();
        pending.sort_by(|left, right| self.m_rows[*left].m_id.cmp(&self.m_rows[*right].m_id));

        for index in pending
        {
            let taken: &HashMap<i32, usize> = &self.m_byNumeric;
            let numeric: i32 = allocate(&self.m_rows[index].m_id, &|candidate| taken.contains_key(&candidate))
                .ok_or_else(|| RegistryError::Exhausted(self.m_name, self.m_rows[index].m_id.clone()))?;
            self.m_rows[index].m_numeric = numeric;
            self.m_byNumeric.insert(numeric, index);
        }

        self.m_frozen = true;
        Ok(())
    }

    pub fn snapshot(&self) -> BTreeMap<Identifier, i32>
    {
        self.m_rows
            .iter()
            .filter(|row| row.m_numeric != UNASSIGNED)
            .map(|row| (row.m_id.clone(), row.m_numeric))
            .collect()
    }

    // applies a saved or received id map to modded rows, rows the map lacks keep their ids when free
    pub fn remap(&mut self, map: &BTreeMap<Identifier, i32>) -> Vec<(Identifier, i32, i32)>
    {
        let mut changes: Vec<(Identifier, i32, i32)> = Vec::new();

        for row in self.m_rows.iter_mut().filter(|row| !row.m_id.isVanilla())
        {
            if let Some(numeric) = map.get(&row.m_id)
                && *numeric != row.m_numeric
            {
                changes.push((row.m_id.clone(), row.m_numeric, *numeric));
                row.m_numeric = *numeric;
            }
        }

        self.m_byNumeric = self
            .m_rows
            .iter()
            .enumerate()
            .filter(|(_, row)| row.m_numeric != UNASSIGNED)
            .map(|(index, row)| (row.m_numeric, index))
            .collect();
        changes
    }
}

// type-erased access so bookkeeping can walk every synced registry from one list
pub trait SyncedRegistry
{
    fn name(&self) -> &'static str;
    fn snapshot(&self) -> BTreeMap<Identifier, i32>;
    fn contains(&self, id: &Identifier) -> bool;
    fn ownsAny(&self, owner: &str) -> bool;
    fn reassign(&mut self, saved: &BTreeMap<Identifier, i32>, reserved: &BTreeSet<i32>);
}

impl<T> SyncedRegistry for Registry<T>
{
    fn name(&self) -> &'static str { self.m_name }

    fn snapshot(&self) -> BTreeMap<Identifier, i32> { Self::snapshot(self) }

    fn contains(&self, id: &Identifier) -> bool { self.get(id).is_some() }

    fn ownsAny(&self, owner: &str) -> bool { self.modded().any(|row| row.m_owner == owner) }

    // saved ids win, modded rows whose id a saved row reserves move to the lowest free ids
    fn reassign(&mut self, saved: &BTreeMap<Identifier, i32>, reserved: &BTreeSet<i32>)
    {
        let free: Vec<(Identifier, i32)> = self
            .modded()
            .filter(|row| !saved.contains_key(&row.m_id))
            .filter(|row| reserved.contains(&row.m_numeric))
            .map(|row| (row.m_id.clone(), row.m_numeric))
            .collect();
        let mut map: BTreeMap<Identifier, i32> = saved.clone();
        let used: BTreeSet<i32> = self
            .rows()
            .filter(|row| !free.iter().any(|(id, _)| *id == row.m_id))
            .map(|row| row.m_numeric)
            .chain(reserved.iter().copied())
            .collect();
        let mut next: i32 = free.iter().map(|(_, numeric)| *numeric).min().unwrap_or(0);

        for (id, _) in free
        {
            while used.contains(&next)
            {
                next += 1;
            }

            map.insert(id, next);
            next += 1;
        }

        self.remap(&map);
    }
}

thread_local!
{
    static REGISTRIES: RefCell<Option<Registries>> = const { RefCell::new(None) };
}

// vanilla rows are seeded on first use so a game without mods touches nothing early
pub fn with<R>(body: impl FnOnce(&mut Registries) -> R) -> R
{
    REGISTRIES.with(|cell|
    {
        let mut slot: std::cell::RefMut<'_, Option<Registries>> = cell.borrow_mut();
        let registries: &mut Registries = slot.get_or_insert_with(Registries::seeded);
        body(registries)
    })
}
