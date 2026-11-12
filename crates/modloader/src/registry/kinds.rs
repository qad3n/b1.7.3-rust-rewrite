use super::{Registries, Registry};
use mod_protocol::Identifier;
use mod_protocol::abi::RegistryKind;
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};

// one registry kind, the data and tag folder name and the singular form that paths and logs also accept
pub struct KindRow
{
    pub m_kind: RegistryKind,
    pub m_plural: &'static str,
    pub m_singular: &'static str,
}

pub const KINDS: [KindRow; 22] = [
    KindRow { m_kind: RegistryKind::Block, m_plural: "blocks", m_singular: "block" },
    KindRow { m_kind: RegistryKind::Item, m_plural: "items", m_singular: "item" },
    KindRow { m_kind: RegistryKind::Entity, m_plural: "entities", m_singular: "entity" },
    KindRow { m_kind: RegistryKind::TileEntity, m_plural: "tile_entities", m_singular: "tile_entity" },
    KindRow { m_kind: RegistryKind::Recipe, m_plural: "recipes", m_singular: "recipe" },
    KindRow { m_kind: RegistryKind::Biome, m_plural: "biomes", m_singular: "biome" },
    KindRow { m_kind: RegistryKind::Dimension, m_plural: "dimensions", m_singular: "dimension" },
    KindRow { m_kind: RegistryKind::Fluid, m_plural: "fluids", m_singular: "fluid" },
    KindRow { m_kind: RegistryKind::Effect, m_plural: "effects", m_singular: "effect" },
    KindRow { m_kind: RegistryKind::Attribute, m_plural: "attributes", m_singular: "attribute" },
    KindRow { m_kind: RegistryKind::Particle, m_plural: "particles", m_singular: "particle" },
    KindRow { m_kind: RegistryKind::Sound, m_plural: "sounds", m_singular: "sound" },
    KindRow { m_kind: RegistryKind::GameRule, m_plural: "gamerules", m_singular: "gamerule" },
    KindRow { m_kind: RegistryKind::Energy, m_plural: "energy", m_singular: "energy" },
    KindRow { m_kind: RegistryKind::Machine, m_plural: "machines", m_singular: "machine" },
    KindRow { m_kind: RegistryKind::AccessorySlot, m_plural: "accessory_slots", m_singular: "accessory_slot" },
    KindRow { m_kind: RegistryKind::Feature, m_plural: "features", m_singular: "feature" },
    KindRow { m_kind: RegistryKind::Structure, m_plural: "structures", m_singular: "structure" },
    KindRow { m_kind: RegistryKind::LootCondition, m_plural: "loot_conditions", m_singular: "loot_condition" },
    KindRow { m_kind: RegistryKind::DamageType, m_plural: "damage_types", m_singular: "damage_type" },
    KindRow { m_kind: RegistryKind::Achievement, m_plural: "achievements", m_singular: "achievement" },
    KindRow { m_kind: RegistryKind::Component, m_plural: "components", m_singular: "component" },
];

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

pub fn byFolder(folder: &str) -> Option<RegistryKind>
{
    KINDS.iter().find(|row| row.m_plural == folder || row.m_singular == folder).map(|row| row.m_kind)
}

pub fn singular(kind: RegistryKind) -> &'static str
{
    KINDS.iter().find(|row| row.m_kind == kind).map_or("unknown", |row| row.m_singular)
}

// kinds whose rows have no numeric ids, every name a mod or the game adds is still claimed here once
pub fn named() -> BTreeMap<RegistryKind, Registry<()>>
{
    KINDS.iter().filter(|row| !numbered(row.m_kind)).map(|row| (row.m_kind, Registry::new(row.m_singular))).collect()
}

fn numbered(kind: RegistryKind) -> bool
{
    matches!(
        kind,
        RegistryKind::Block
            | RegistryKind::Item
            | RegistryKind::Entity
            | RegistryKind::TileEntity
            | RegistryKind::Recipe
            | RegistryKind::Biome
            | RegistryKind::Dimension
    )
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
