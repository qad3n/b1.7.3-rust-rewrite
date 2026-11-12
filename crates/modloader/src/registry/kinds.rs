use super::{Registries, Registry};
use mod_protocol::Identifier;
use mod_protocol::abi::RegistryKind;
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};

thread_local!
{
    // sounds and vanilla particles exist only where the client loaded them, kept apart so a game without mods
    // never builds the registries early
    static CLIENT: RefCell<BTreeMap<RegistryKind, BTreeSet<Identifier>>> = const { RefCell::new(BTreeMap::new()) };
}

pub fn addClientName(kind: RegistryKind, id: Identifier)
{
    CLIENT.with(|names| names.borrow_mut().entry(kind).or_default().insert(id));
}

fn clientOnly(kind: RegistryKind) -> bool { matches!(kind, RegistryKind::Particle | RegistryKind::Sound) }

pub fn byFolder(folder: &str) -> Option<RegistryKind> { RegistryKind::byFolder(folder) }

pub fn singular(kind: RegistryKind) -> &'static str { kind.singular() }

// kinds whose rows have no numeric ids, every name a mod or the game adds is still claimed here once
pub fn named() -> BTreeMap<RegistryKind, Registry<()>>
{
    RegistryKind::ROWS
        .iter()
        .filter(|row| !row.kind.numbered())
        .map(|row| (row.kind, Registry::new(row.singular)))
        .collect()
}

impl Registries
{
    fn namedMut(&mut self, kind: RegistryKind) -> Option<&mut Registry<()>> { self.m_named.get_mut(&kind) }

    // a game or loader row of a named registry, a repeat keeps the first
    pub fn seedName(&mut self, kind: RegistryKind, id: Identifier)
    {
        if let Some(registry) = self.namedMut(kind)
            && registry.get(&id).is_none()
        {
            registry.seed(id, super::UNASSIGNED, ());
        }
    }

    // a mod's entry of a named registry, refused once frozen or when the name is taken
    pub fn claim(&mut self, kind: RegistryKind, id: &Identifier) -> Result<(), String>
    {
        let owner: String = id.namespace().to_owned();
        let registry: &mut Registry<()> =
            self.namedMut(kind).ok_or_else(|| format!("{} is a numbered registry", singular(kind)))?;
        registry.register(id.clone(), &owner, ()).map_err(|error| error.to_string())
    }

    // a server never loads sounds or vanilla particles, so it takes any name of those kinds
    pub fn contains(&self, kind: RegistryKind, id: &Identifier) -> bool
    {
        clientOnly(kind) && !crate::side::isClient() || self.list(kind).contains(id)
    }

    // vanilla rows first in their game order, then mod rows in name order
    pub fn list(&self, kind: RegistryKind) -> Vec<Identifier>
    {
        let ids: Vec<Identifier> = match kind
        {
            RegistryKind::Block => ids(&self.m_blocks),
            RegistryKind::Item => ids(&self.m_items),
            RegistryKind::Entity => ids(&self.m_entities),
            RegistryKind::TileEntity => ids(&self.m_tileEntities),
            RegistryKind::Recipe => ids(&self.m_recipes),
            RegistryKind::Biome => ids(&self.m_biomes),
            RegistryKind::Dimension => ids(&self.m_dimensions),
            _ => self.m_named.get(&kind).map(ids).unwrap_or_default(),
        };
        let client: BTreeSet<Identifier> = CLIENT.with(|names| names.borrow().get(&kind).cloned().unwrap_or_default());
        let ids: Vec<Identifier> =
            client.into_iter().filter(|id| !ids.contains(id)).chain(ids.iter().cloned()).collect();
        let (mut vanilla, mut modded): (Vec<Identifier>, Vec<Identifier>) =
            ids.into_iter().partition(Identifier::isVanilla);

        modded.sort();
        vanilla.append(&mut modded);
        vanilla
    }

    pub fn freezeNamed(&mut self)
    {
        for registry in self.m_named.values_mut()
        {
            registry.close();
        }
    }
}

fn ids<T>(registry: &Registry<T>) -> Vec<Identifier> { registry.rows().map(|row| row.m_id.clone()).collect() }
