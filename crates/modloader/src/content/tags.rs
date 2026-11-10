use crate::registry::Registries;
use game::tags::{Registry, TagRow};
use mod_protocol::Identifier;
use mod_protocol::abi::RegistryKind;
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};

// shared lists such as c:ores/ruby that mods fill and read without depending on each other
thread_local! {
    static TAGS: RefCell<BTreeMap<(RegistryKind, Identifier), BTreeSet<Identifier>>> = const { RefCell::new(BTreeMap::new()) };
}

pub fn registryKind(folder: &str) -> Option<RegistryKind>
{
    match folder
    {
        "blocks" | "block" => Some(RegistryKind::Block),
        "items" | "item" => Some(RegistryKind::Item),
        "entities" | "entity" => Some(RegistryKind::Entity),
        "biomes" | "biome" => Some(RegistryKind::Biome),
        _ => None,
    }
}

pub fn add(registry: RegistryKind, tag: Identifier, values: Vec<Identifier>, replace: bool)
{
    TAGS.with(|tags| {
        let mut tags = tags.borrow_mut();
        let entry: &mut BTreeSet<Identifier> = tags.entry((registry, tag)).or_default();
        if replace
        {
            entry.clear();
        }
        entry.extend(values);
    });
}

// a mod block or item flag puts it in the engine tag of the same meaning
pub fn flag(registry: RegistryKind, tag: &str, id: &Identifier)
{
    add(registry, Identifier::minecraft(tag), vec![id.clone()], false);
}

pub fn values(registry: RegistryKind, tag: &Identifier) -> Vec<Identifier>
{
    TAGS.with(|tags| {
        tags.borrow().get(&(registry, tag.clone())).map(|set| set.iter().cloned().collect()).unwrap_or_default()
    })
}

pub fn contains(registry: RegistryKind, tag: &Identifier, value: &Identifier) -> bool
{
    TAGS.with(|tags| tags.borrow().get(&(registry, tag.clone())).is_some_and(|set| set.contains(value)))
}

pub fn hash() -> String
{
    TAGS.with(|tags| {
        let text: String = format!("{:?}", tags.borrow());
        crate::discovery::hex(&<sha2::Sha256 as sha2::Digest>::digest(text.as_bytes()))
    })
}

// the engine's beta members become minecraft tags that mods can read, extend or replace
pub fn seed()
{
    for row in &game::tags::ROWS
    {
        let members: Vec<Identifier> =
            crate::registry::with(|registries| row.m_seeds.iter().filter_map(|n| name(registries, row, *n)).collect());
        TAGS.with(|tags| {
            tags.borrow_mut()
                .entry((kind(row.m_registry), Identifier::minecraft(row.m_name)))
                .or_default()
                .extend(members)
        });
    }
}

// engine tags read numbers, so every change and every new id map resolves them again
pub fn push()
{
    for row in &game::tags::ROWS
    {
        let members: Vec<Identifier> = values(kind(row.m_registry), &Identifier::minecraft(row.m_name));
        let ids: Vec<i32> = crate::registry::with(|registries| {
            members.iter().filter_map(|member| numeric(registries, row, member)).collect()
        });
        game::tags::set(row.m_tag, &ids);
    }
}

fn kind(registry: Registry) -> RegistryKind
{
    match registry
    {
        Registry::Block => RegistryKind::Block,
        Registry::Item => RegistryKind::Item,
    }
}

fn name(registries: &Registries, row: &TagRow, numeric: i32) -> Option<Identifier>
{
    match row.m_registry
    {
        Registry::Block => registries.m_blocks.byNumeric(numeric).map(|entry| entry.m_id.clone()),
        Registry::Item => registries.m_items.byNumeric(numeric).map(|entry| entry.m_id.clone()),
    }
}

fn numeric(registries: &Registries, row: &TagRow, id: &Identifier) -> Option<i32>
{
    match row.m_registry
    {
        Registry::Block => registries.m_blocks.numeric(id),
        Registry::Item => registries.m_items.numeric(id),
    }
}
