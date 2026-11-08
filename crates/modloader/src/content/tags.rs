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
