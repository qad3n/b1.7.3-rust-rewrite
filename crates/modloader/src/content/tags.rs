use crate::registry::Registries;
use game::tags::{Registry, TagRow};
use mod_protocol::Identifier;
use mod_protocol::abi::RegistryKind;
use mod_protocol::spec::{TagEntry, TagRef};
use std::cell::{RefCell, RefMut};
use std::collections::{BTreeMap, BTreeSet};

type Key = (RegistryKind, Identifier);
type Entries = BTreeMap<Key, BTreeSet<TagEntry>>;

// registry, its tag folder name and the singular form both accepted in data paths and used on the wire
const REGISTRIES: [(RegistryKind, &str, &str); 4] = [
    (RegistryKind::Block, "blocks", "block"),
    (RegistryKind::Item, "items", "item"),
    (RegistryKind::Entity, "entities", "entity"),
    (RegistryKind::Biome, "biomes", "biome"),
];

// shared lists such as c:ores/ruby that mods fill and read without depending on each other
thread_local!
{
    static TAGS: RefCell<Entries> = const { RefCell::new(BTreeMap::new()) };
    // the client's own tags while a modded server's tags stand in for them
    static LOCAL: RefCell<Option<Entries>> = const { RefCell::new(None) };
}

pub fn registryKind(folder: &str) -> Option<RegistryKind>
{
    REGISTRIES.iter().find(|(_, plural, singular)| *plural == folder || *singular == folder).map(|(kind, _, _)| *kind)
}

fn registryName(registry: RegistryKind) -> Option<&'static str>
{
    REGISTRIES.iter().find(|(kind, _, _)| *kind == registry).map(|(_, _, singular)| *singular)
}

pub fn add(registry: RegistryKind, tag: Identifier, values: Vec<Identifier>, replace: bool)
{
    let entries: Vec<TagEntry> =
        values.into_iter().map(|id| TagEntry { target: TagRef::Id(id), required: true }).collect();
    addEntries(registry, tag, entries, replace);
}

pub fn addEntries(registry: RegistryKind, tag: Identifier, entries: Vec<TagEntry>, replace: bool)
{
    TAGS.with(|tags|
    {
        let mut tags: RefMut<BTreeMap<(RegistryKind, Identifier), BTreeSet<TagEntry>>> = tags.borrow_mut();
        let members: &mut BTreeSet<TagEntry> = tags.entry((registry, tag)).or_default();

        if replace
        {
            members.clear();
        }

        members.extend(entries);
    });
}

// a mod block or item flag puts it in the engine tag of the same meaning
pub fn flag(registry: RegistryKind, tag: &str, id: &Identifier)
{
    add(registry, Identifier::minecraft(tag), vec![id.clone()], false);
}

pub fn values(registry: RegistryKind, tag: &Identifier) -> Vec<Identifier>
{
    resolved(registry, tag, &mut Vec::new()).into_keys().collect()
}

pub fn contains(registry: RegistryKind, tag: &Identifier, value: &Identifier) -> bool
{
    resolved(registry, tag, &mut Vec::new()).contains_key(value)
}

// every tag of the registry that resolves to hold the value
pub fn of(registry: RegistryKind, value: &Identifier) -> Vec<Identifier>
{
    let keys: Vec<Key> = TAGS.with(|tags| tags.borrow().keys().cloned().collect());
    keys.into_iter()
        .filter(|(kind, tag)| *kind == registry && contains(registry, tag, value))
        .map(|(_, tag)| tag)
        .collect()
}

// every tag a registry holds, in name order
pub fn names(registry: RegistryKind) -> Vec<Identifier>
{
    TAGS.with(|tags| tags.borrow().keys().filter(|(kind, _)| *kind == registry).map(|(_, tag)| tag.clone()).collect())
}

pub fn hash() -> String
{
    let text: String = table();
    crate::discovery::hex(&<sha2::Sha256 as sha2::Digest>::digest(text.as_bytes()))
}

// the engine's beta members become minecraft tags that mods can read, extend or replace
pub fn seed()
{
    for row in &game::tags::ROWS
    {
        let members: Vec<Identifier> =
            crate::registry::with(|registries| row.m_seeds.iter().filter_map(|n| name(registries, row, *n)).collect());
        add(kind(row.m_registry), Identifier::minecraft(row.m_name), members, false);
    }

    for row in &game::tags::BIOME_ROWS
    {
        let members: Vec<Identifier> = row.m_seeds.iter().filter_map(|name| vanillaBiome(name)).collect();
        add(RegistryKind::Biome, Identifier::minecraft(row.m_name), members, false);
    }

    for row in super::vanilla_tags::TAGS
    {
        let members: Vec<Identifier> = row.m_members.iter().map(|member| vanillaId(member)).collect();
        add(row.m_registry, vanillaId(row.m_name), members, false);
    }
}

fn vanillaId(name: &str) -> Identifier { Identifier::parse(name).expect("vanilla tag rows hold valid identifiers") }

// engine tags read numbers, so every change and every new id map resolves them again
pub fn push()
{
    for row in &game::tags::ROWS
    {
        let members: BTreeMap<Identifier, bool> =
            resolved(kind(row.m_registry), &Identifier::minecraft(row.m_name), &mut Vec::new());

        let mut ids: Vec<i32> = crate::registry::with(|registries|
        {
            members.keys().filter_map(|member| numeric(registries, row.m_registry, member)).collect()
        });

        // a block with more than 16 states spans hidden page ids that are the same block
        if row.m_registry == Registry::Block
        {
            ids.extend(members.keys().flat_map(crate::states::pageIds));
        }

        game::tags::set(row.m_tag, &ids);
    }

    for row in &game::tags::BIOME_ROWS
    {
        let members: BTreeMap<Identifier, bool> =
            resolved(RegistryKind::Biome, &Identifier::minecraft(row.m_name), &mut Vec::new());
        let names: BTreeSet<String> = members.keys().filter_map(biomeName).collect();
        game::tags::setBiomes(row.m_tag, names);
    }
}

// beta biomes go by their display name, a vanilla identifier maps through the shared biome table
fn vanillaBiome(name: &str) -> Option<Identifier>
{
    crate::registry::BIOMES
        .iter()
        .find(|(_, biome)| biome().m_biomeName.get() == name)
        .map(|(path, _)| Identifier::minecraft(path))
}

fn biomeName(id: &Identifier) -> Option<String>
{
    if id.isVanilla()
    {
        return crate::registry::BIOMES
            .iter()
            .find(|(path, _)| *path == id.path())
            .map(|(_, biome)| biome().m_biomeName.get().to_owned());
    }

    crate::dimensions::biomes().into_iter().find(|biome| biome.m_id == *id).map(|biome| biome.m_spec.name)
}

// after a content build every registry is complete, so missing required members are real mistakes
pub fn report()
{
    let mut problems: Vec<String> = Vec::new();
    let keys: Vec<Key> = TAGS.with(|tags| tags.borrow().keys().cloned().collect());
    for (registry, tag) in keys
    {
        let Some(engine) = engineRegistry(registry)
        else
        {
            continue;
        };

        let members: BTreeMap<Identifier, bool> = resolved(registry, &tag, &mut problems);

        let missing: Vec<Identifier> = crate::registry::with(|registries|
        {
            members
                .iter()
                .filter(|(member, required)| **required && numeric(registries, engine, member).is_none())
                .map(|(member, _)| member.clone())
                .collect()
        });
        problems.extend(missing.iter().map(|member| format!("tag #{tag} lists {member}, which is not registered")));
    }

    let unique: BTreeSet<String> = problems.into_iter().collect();
    for problem in unique
    {
        crate::log::warn(crate::log::LOADER, &problem);
    }
}

// every tag resolved to plain ids, one line each, for the data hash and for sending to clients
pub fn table() -> String
{
    let keys: Vec<Key> = TAGS.with(|tags| tags.borrow().keys().cloned().collect());
    let mut text: String = String::new();

    for (registry, tag) in keys
    {
        let Some(registryName) = registryName(registry)
        else
        {
            continue;
        };

        let members: Vec<String> = values(registry, &tag).iter().map(Identifier::to_string).collect();
        text.push_str(&format!("{registryName} {tag} {}\n", members.join(" ")));
    }

    text
}

// a modded server's tags replace the client's until it disconnects
pub fn applySynced(text: &str) -> Result<(), String>
{
    let mut synced: Entries = BTreeMap::new();

    for line in text.lines()
    {
        let mut words = line.split(' ').filter(|word| !word.is_empty());
        let (Some(registryName), Some(tag)) = (words.next(), words.next())
        else
        {
            return Err(format!("bad tag line '{line}'"));
        };

        let registry: RegistryKind =
            registryKind(registryName).ok_or_else(|| format!("unknown tag registry {registryName}"))?;
        let tag: Identifier = Identifier::parse(tag).map_err(|error| error.to_string())?;
        let mut members: BTreeSet<TagEntry> = BTreeSet::new();

        for word in words
        {
            let id: Identifier = Identifier::parse(word).map_err(|error| error.to_string())?;
            members.insert(TagEntry { target: TagRef::Id(id), required: false });
        }

        synced.insert((registry, tag), members);
    }

    let own: Entries = TAGS.with(|tags| std::mem::replace(&mut *tags.borrow_mut(), synced));

    LOCAL.with(|local|
    {
        local.borrow_mut().get_or_insert(own);
    });

    push();
    Ok(())
}

pub fn restoreLocal()
{
    if let Some(own) = LOCAL.with(|local| local.borrow_mut().take())
    {
        TAGS.with(|tags| *tags.borrow_mut() = own);
        push();
    }
}

// members of one tag with nested tags expanded, each marked required when any path to it is
fn resolved(registry: RegistryKind, tag: &Identifier, problems: &mut Vec<String>) -> BTreeMap<Identifier, bool>
{
    TAGS.with(|tags|
    {
        let mut members: BTreeMap<Identifier, bool> = BTreeMap::new();
        expand(&tags.borrow(), registry, tag, &mut Vec::new(), &mut members, problems);
        members
    })
}

fn expand(
    tags: &Entries,
    registry: RegistryKind,
    tag: &Identifier,
    trail: &mut Vec<Identifier>,
    members: &mut BTreeMap<Identifier, bool>,
    problems: &mut Vec<String>,
)
{
    let Some(entries) = tags.get(&(registry, tag.clone()))
    else
    {
        return;
    };

    trail.push(tag.clone());

    for entry in entries
    {
        match &entry.target
        {
            TagRef::Id(id) => *members.entry(id.clone()).or_insert(false) |= entry.required,

            TagRef::Tag(nested) if trail.contains(nested) =>
            {
                problems.push(format!("tag #{tag} includes #{nested}, which already includes it"));
            }

            TagRef::Tag(nested) if !tags.contains_key(&(registry, nested.clone())) =>
            {
                if entry.required
                {
                    problems.push(format!("tag #{tag} includes #{nested}, which does not exist"));
                }
            }

            TagRef::Tag(nested) => expand(tags, registry, nested, trail, members, problems),
        }
    }

    trail.pop();
}

fn kind(registry: Registry) -> RegistryKind
{
    match registry
    {
        Registry::Block => RegistryKind::Block,
        Registry::Item => RegistryKind::Item,
    }
}

fn engineRegistry(registry: RegistryKind) -> Option<Registry>
{
    match registry
    {
        RegistryKind::Block => Some(Registry::Block),
        RegistryKind::Item => Some(Registry::Item),
        _ => None,
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

fn numeric(registries: &Registries, registry: Registry, id: &Identifier) -> Option<i32>
{
    match registry
    {
        Registry::Block => registries.m_blocks.numeric(id),
        Registry::Item => registries.m_items.numeric(id),
    }
}
