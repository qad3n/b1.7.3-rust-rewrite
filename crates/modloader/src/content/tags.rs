use crate::registry::Registries;
use game::tags::{Registry, TagRow};
use mod_protocol::Identifier;
use mod_protocol::abi::RegistryKind;
use mod_protocol::spec::{TagEntry, TagRef};
use std::cell::{RefCell, RefMut};
use std::collections::{BTreeMap, BTreeSet};

// a built in registry kind or a registry a mod created
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Scope
{
    Kind(RegistryKind),
    Custom(Identifier),
}

impl From<RegistryKind> for Scope
{
    fn from(kind: RegistryKind) -> Scope { Scope::Kind(kind) }
}

impl Scope
{
    // the word naming the registry in synced tag lines, a mod registry is @<owner>:<name>
    fn word(&self) -> String
    {
        match self
        {
            Scope::Kind(kind) => kind.singular().to_owned(),
            Scope::Custom(id) => format!("@{id}"),
        }
    }

    fn parse(word: &str) -> Option<Scope>
    {
        match word.strip_prefix('@')
        {
            Some(id) => Identifier::parse(id).ok().map(Scope::Custom),
            None => registryKind(word).map(Scope::Kind),
        }
    }

    fn known(&self, member: &Identifier) -> bool
    {
        match self
        {
            Scope::Kind(kind) => crate::registry::with(|registries| registries.contains(*kind, member)),
            Scope::Custom(registry) => super::custom::contains(registry, member),
        }
    }
}

type Key = (Scope, Identifier);
type Entries = BTreeMap<Key, BTreeSet<TagEntry>>;

// shared lists such as c:ores/ruby that mods fill and read without depending on each other
thread_local!
{
    static TAGS: RefCell<Entries> = const { RefCell::new(BTreeMap::new()) };
    // the client's own tags while a modded server's tags stand in for them
    static LOCAL: RefCell<Option<Entries>> = const { RefCell::new(None) };
}

pub fn registryKind(folder: &str) -> Option<RegistryKind> { crate::registry::byFolder(folder) }

pub fn add(registry: impl Into<Scope>, tag: Identifier, values: Vec<Identifier>, replace: bool)
{
    let entries: Vec<TagEntry> =
        values.into_iter().map(|id| TagEntry { target: TagRef::Id(id), required: true }).collect();
    addEntries(registry, tag, entries, replace);
}

pub fn addEntries(registry: impl Into<Scope>, tag: Identifier, entries: Vec<TagEntry>, replace: bool)
{
    let registry: Scope = registry.into();

    TAGS.with(|tags|
    {
        let mut tags: RefMut<Entries> = tags.borrow_mut();
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

pub fn values(registry: impl Into<Scope>, tag: &Identifier) -> Vec<Identifier>
{
    resolved(&registry.into(), tag, &mut Vec::new()).into_keys().collect()
}

pub fn contains(registry: impl Into<Scope>, tag: &Identifier, value: &Identifier) -> bool
{
    resolved(&registry.into(), tag, &mut Vec::new()).contains_key(value)
}

// every tag of the registry that resolves to hold the value
pub fn of(registry: impl Into<Scope>, value: &Identifier) -> Vec<Identifier>
{
    let registry: Scope = registry.into();
    let keys: Vec<Key> = TAGS.with(|tags| tags.borrow().keys().cloned().collect());
    keys.into_iter()
        .filter(|(scope, tag)| *scope == registry && resolved(scope, tag, &mut Vec::new()).contains_key(value))
        .map(|(_, tag)| tag)
        .collect()
}

// every tag a registry holds, in name order
pub fn names(registry: impl Into<Scope>) -> Vec<Identifier>
{
    let registry: Scope = registry.into();
    TAGS.with(|tags| tags.borrow().keys().filter(|(scope, _)| *scope == registry).map(|(_, tag)| tag.clone()).collect())
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
    super::custom::invalidate();
    crate::biomes::invalidateOverworld();

    for row in &game::tags::ROWS
    {
        let members: BTreeMap<Identifier, bool> =
            resolved(&Scope::Kind(kind(row.m_registry)), &Identifier::minecraft(row.m_name), &mut Vec::new());

        let ids: Vec<i32> = match row.m_registry
        {
            Registry::Block => numericBlocks(&members).into_iter().collect(),
            Registry::Item => crate::registry::with(|registries|
            {
                members.keys().filter_map(|member| numeric(registries, row.m_registry, member)).collect()
            }),
        };

        game::tags::set(row.m_tag, &ids);
    }

    for row in &game::tags::BIOME_ROWS
    {
        let members: BTreeMap<Identifier, bool> =
            resolved(&Scope::Kind(RegistryKind::Biome), &Identifier::minecraft(row.m_name), &mut Vec::new());
        let names: BTreeSet<String> = members.keys().filter_map(biomeName).collect();
        game::tags::setBiomes(row.m_tag, names);
    }
}

// a block tag's numeric ids for code that checks many blocks
pub fn blockIds(tag: &Identifier) -> BTreeSet<i32>
{
    numericBlocks(&resolved(&Scope::Kind(RegistryKind::Block), tag, &mut Vec::new()))
}

fn numericBlocks(members: &BTreeMap<Identifier, bool>) -> BTreeSet<i32>
{
    let mut ids: BTreeSet<i32> =
        crate::registry::with(|registries| members.keys().filter_map(|member| registries.blockId(member)).collect());

    // a block with more than 16 states spans hidden page ids that are the same block
    ids.extend(members.keys().flat_map(crate::states::pageIds));
    ids
}

// beta biomes go by their display name, a vanilla identifier maps through the shared biome table
fn vanillaBiome(name: &str) -> Option<Identifier>
{
    game::worldgen::BiomeGenBase::NAMED
        .iter()
        .find(|(_, biome)| biome().m_biomeName.get() == name)
        .map(|(path, _)| Identifier::minecraft(path))
}

fn biomeName(id: &Identifier) -> Option<String>
{
    if id.isVanilla()
    {
        return game::worldgen::BiomeGenBase::NAMED
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
        let members: BTreeMap<Identifier, bool> = resolved(&registry, &tag, &mut problems);
        let missing: Vec<Identifier> = members
            .iter()
            .filter(|(member, required)| **required && !registry.known(member))
            .map(|(member, _)| member.clone())
            .collect();
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
        let members: Vec<String> =
            resolved(&registry, &tag, &mut Vec::new()).keys().map(Identifier::to_string).collect();
        text.push_str(&format!("{} {tag} {}\n", registry.word(), members.join(" ")));
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

        let registry: Scope =
            Scope::parse(registryName).ok_or_else(|| format!("unknown tag registry {registryName}"))?;
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
fn resolved(registry: &Scope, tag: &Identifier, problems: &mut Vec<String>) -> BTreeMap<Identifier, bool>
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
    registry: &Scope,
    tag: &Identifier,
    trail: &mut Vec<Identifier>,
    members: &mut BTreeMap<Identifier, bool>,
    problems: &mut Vec<String>,
)
{
    let Some(entries) = tags.get(&(registry.clone(), tag.clone()))
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

            TagRef::Tag(nested) if !tags.contains_key(&(registry.clone(), nested.clone())) =>
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
