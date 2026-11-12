use super::tags::{self, Scope};
use mod_protocol::Identifier;
use mod_protocol::abi::{MapTarget, RegistryKind};
use mod_protocol::data::VALUE_KEY;
use mod_protocol::spec::TagRef;
use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;

// a registry a mod created, each entry a TOML document only the owner's types read
struct Registry
{
    m_synced: bool,
    m_entries: BTreeMap<Identifier, String>,
}

// one call or data file setting values of a data map, merged in load order
struct Values
{
    m_values: Vec<(TagRef, String)>,
    m_remove: Vec<TagRef>,
    m_replace: bool,
}

struct DataMap
{
    m_target: MapTarget,
    m_synced: bool,
    m_sets: Vec<Values>,
}

thread_local!
{
    static REGISTRIES: RefCell<BTreeMap<Identifier, Registry>> = const { RefCell::new(BTreeMap::new()) };
    // entries of registries whose owner has not created them yet, other mods may load first
    static PENDING: RefCell<Vec<(Identifier, Identifier, String)>> = const { RefCell::new(Vec::new()) };
    static MAPS: RefCell<BTreeMap<Identifier, DataMap>> = RefCell::new(builtins());
    // values set before their map was created, resolved with the rest once mods load
    static EARLY: RefCell<Vec<(Identifier, Option<MapTarget>, Values)>> = const { RefCell::new(Vec::new()) };
    // each map's values resolved through tags, rebuilt after any tag change
    static RESOLVED: RefCell<BTreeMap<Identifier, BTreeMap<Identifier, String>>> = const { RefCell::new(BTreeMap::new()) };
    static STALE: Cell<bool> = const { Cell::new(true) };
    // a modded server's synced registries and maps, read before the client's own until it disconnects
    static SERVED_ENTRIES: RefCell<BTreeMap<Identifier, BTreeMap<Identifier, String>>> = const { RefCell::new(BTreeMap::new()) };
    static SERVED_MAPS: RefCell<BTreeMap<Identifier, BTreeMap<Identifier, String>>> = const { RefCell::new(BTreeMap::new()) };
}

const REGISTRY_ROW: &str = "registry";
const MAP_ROW: &str = "map";

// furnace burn ticks by item, any mod or data file may set or remove them, as later versions' data map
const FURNACE_FUELS: &str = "furnace_fuels";

fn builtins() -> BTreeMap<Identifier, DataMap>
{
    let fuels: DataMap = DataMap { m_target: MapTarget::Kind(RegistryKind::Item), m_synced: true, m_sets: Vec::new() };
    BTreeMap::from([(Identifier::minecraft(FURNACE_FUELS), fuels)])
}

// built in maps write into the tables the game reads, after recipes so a map has the last word
pub fn applyBuiltins()
{
    for (item, text) in mapEntries(&Identifier::minecraft(FURNACE_FUELS))
    {
        let numeric: Option<i32> = crate::registry::with(|registries| registries.itemId(&item));
        match (numeric, integer(&text))
        {
            (Some(numeric), Some(ticks)) => super::putFuel(numeric, ticks),
            _ =>
            {
                crate::log::warn(crate::log::LOADER, &format!("furnace fuel {item} = {text} is not an item and ticks"))
            }
        }
    }
}

// the whole number a data map value holds
fn integer(text: &str) -> Option<i32>
{
    let document: toml::Table = toml::from_str(text).ok()?;
    i32::try_from(document.get(VALUE_KEY)?.as_integer()?).ok()
}

pub fn create(id: Identifier, synced: bool) -> Result<(), String>
{
    REGISTRIES.with(|registries|
    {
        let mut registries = registries.borrow_mut();
        if registries.contains_key(&id)
        {
            return Err(format!("registry {id} already exists"));
        }

        registries.insert(id, Registry { m_synced: synced, m_entries: BTreeMap::new() });
        Ok(())
    })
}

pub fn add(registry: Identifier, id: Identifier, value: String) -> Result<(), String>
{
    toml::from_str::<toml::Table>(&value).map_err(|error| format!("entry {id} of {registry}: {error}"))?;
    PENDING.with(|pending| pending.borrow_mut().push((registry, id, value)));
    Ok(())
}

pub fn createMap(id: Identifier, target: MapTarget, synced: bool) -> Result<(), String>
{
    MAPS.with(|maps|
    {
        let mut maps = maps.borrow_mut();
        if maps.contains_key(&id)
        {
            return Err(format!("data map {id} already exists"));
        }

        maps.insert(id, DataMap { m_target: target, m_synced: synced, m_sets: Vec::new() });
        Ok(())
    })
}

pub fn setValues(
    map: Identifier,
    target: Option<MapTarget>,
    values: Vec<(TagRef, String)>,
    remove: Vec<TagRef>,
    replace: bool,
)
{
    let set: Values = Values { m_values: values, m_remove: remove, m_replace: replace };
    EARLY.with(|early| early.borrow_mut().push((map, target, set)));
    STALE.with(|stale| stale.set(true));
}

// once every mod registered, pending entries join their registries and early values their maps
pub fn settle() -> Vec<(String, String)>
{
    let mut problems: Vec<(String, String)> = Vec::new();

    for (registry, id, value) in PENDING.with(|pending| pending.borrow_mut().drain(..).collect::<Vec<_>>())
    {
        let id: Identifier = crate::remap::rename(&id);
        let placed: Result<(), String> = REGISTRIES.with(|registries|
        {
            let mut registries = registries.borrow_mut();
            let entries: &mut BTreeMap<Identifier, String> = &mut registries
                .get_mut(&registry)
                .ok_or_else(|| format!("registry {registry} does not exist"))?
                .m_entries;
            if entries.contains_key(&id)
            {
                return Err(format!("{id} is already in registry {registry}"));
            }

            entries.insert(id.clone(), value);
            Ok(())
        });
        problems.extend(placed.err().map(|reason| (id.namespace().to_owned(), reason)));
    }

    for (map, target, set) in EARLY.with(|early| early.borrow_mut().drain(..).collect::<Vec<_>>())
    {
        let placed: Result<(), String> = MAPS.with(|maps|
        {
            let mut maps = maps.borrow_mut();
            let entry: &mut DataMap = maps.get_mut(&map).ok_or_else(|| format!("data map {map} does not exist"))?;
            if let Some(target) = &target
                && *target != entry.m_target
            {
                return Err(format!("data map {map} attaches to {:?}, a file named {target:?}", entry.m_target));
            }

            entry.m_sets.push(set);
            Ok(())
        });
        problems.extend(placed.err().map(|reason| (map.namespace().to_owned(), reason)));
    }

    STALE.with(|stale| stale.set(true));
    problems
}

pub fn invalidate() { STALE.with(|stale| stale.set(true)); }

pub fn contains(registry: &Identifier, id: &Identifier) -> bool
{
    if let Some(served) =
        SERVED_ENTRIES.with(|served| served.borrow().get(registry).map(|entries| entries.contains_key(id)))
    {
        return served;
    }

    REGISTRIES.with(|registries| registries.borrow().get(registry).is_some_and(|row| row.m_entries.contains_key(id)))
}

pub fn exists(registry: &Identifier) -> bool
{
    REGISTRIES.with(|registries| registries.borrow().contains_key(registry))
}

pub fn entries(registry: &Identifier) -> Vec<(Identifier, String)>
{
    if let Some(served) = SERVED_ENTRIES.with(|served| served.borrow().get(registry).cloned())
    {
        return served.into_iter().collect();
    }

    REGISTRIES.with(|registries|
    {
        registries
            .borrow()
            .get(registry)
            .map(|row| row.m_entries.iter().map(|(id, value)| (id.clone(), value.clone())).collect())
            .unwrap_or_default()
    })
}

pub fn value(map: &Identifier, id: &Identifier) -> Option<String>
{
    if let Some(served) = SERVED_MAPS.with(|served| served.borrow().get(map).map(|values| values.get(id).cloned()))
    {
        return served;
    }

    resolve();
    RESOLVED.with(|resolved| resolved.borrow().get(map).and_then(|values| values.get(id).cloned()))
}

pub fn mapEntries(map: &Identifier) -> Vec<(Identifier, String)>
{
    if let Some(served) = SERVED_MAPS.with(|served| served.borrow().get(map).cloned())
    {
        return served.into_iter().collect();
    }

    resolve();
    RESOLVED.with(|resolved|
    {
        resolved
            .borrow()
            .get(map)
            .map(|values| values.iter().map(|(id, value)| (id.clone(), value.clone())).collect())
            .unwrap_or_default()
    })
}

fn members(target: &MapTarget, key: &TagRef) -> Vec<Identifier>
{
    let scope: Scope = match target
    {
        MapTarget::Kind(kind) => Scope::Kind(*kind),
        MapTarget::Registry(registry) => Scope::Custom(registry.clone()),
    };

    match key
    {
        TagRef::Id(id) => vec![id.clone()],
        TagRef::Tag(tag) => tags::values(scope, tag),
    }
}

// later sets win per entry, replace starts over, removals apply after the set's own values
fn resolve()
{
    if !STALE.with(|stale| stale.replace(false))
    {
        return;
    }

    let resolved: BTreeMap<Identifier, BTreeMap<Identifier, String>> = MAPS.with(|maps|
    {
        maps.borrow()
            .iter()
            .map(|(id, map)|
            {
                let mut values: BTreeMap<Identifier, String> = BTreeMap::new();

                for set in &map.m_sets
                {
                    if set.m_replace
                    {
                        values.clear();
                    }

                    for (key, value) in &set.m_values
                    {
                        for member in members(&map.m_target, key)
                        {
                            values.insert(member, value.clone());
                        }
                    }

                    for key in &set.m_remove
                    {
                        for member in members(&map.m_target, key)
                        {
                            values.remove(&member);
                        }
                    }
                }

                (id.clone(), values)
            })
            .collect()
    });
    RESOLVED.with(|slot| *slot.borrow_mut() = resolved);
}

// synced registries and maps for the server's data table, one line per entry
pub fn synced() -> Vec<(String, Identifier, String)>
{
    let mut rows: Vec<(String, Identifier, String)> = Vec::new();
    let registries: Vec<Identifier> = REGISTRIES.with(|registries|
    {
        registries.borrow().iter().filter(|(_, row)| row.m_synced).map(|(id, _)| id.clone()).collect()
    });
    let maps: Vec<Identifier> =
        MAPS.with(|maps| maps.borrow().iter().filter(|(_, map)| map.m_synced).map(|(id, _)| id.clone()).collect());

    // a row with an empty value only names the registry or map, so one with no entries still replaces the client's
    for registry in registries
    {
        let holder: String = format!("{REGISTRY_ROW} {registry}");
        rows.push((holder.clone(), registry.clone(), String::new()));
        rows.extend(entries(&registry).into_iter().map(|(id, value)| (holder.clone(), id, value)));
    }

    for map in maps
    {
        let holder: String = format!("{MAP_ROW} {map}");
        rows.push((holder.clone(), map.clone(), String::new()));
        rows.extend(mapEntries(&map).into_iter().map(|(id, value)| (holder.clone(), id, value)));
    }

    rows
}

// the entry names of every registry, for the join check against a server
pub fn names() -> BTreeMap<Identifier, Vec<Identifier>>
{
    REGISTRIES.with(|registries|
    {
        registries.borrow().iter().map(|(id, row)| (id.clone(), row.m_entries.keys().cloned().collect())).collect()
    })
}

// the server's synced rows stand in for the client's own
pub fn replaceSynced(rows: Vec<(String, Identifier, String)>)
{
    let mut entries: BTreeMap<Identifier, BTreeMap<Identifier, String>> = BTreeMap::new();
    let mut maps: BTreeMap<Identifier, BTreeMap<Identifier, String>> = BTreeMap::new();

    for (holder, id, value) in rows
    {
        let target = match holder.split_once(' ')
        {
            Some((REGISTRY_ROW, name)) => Identifier::parse(name).ok().map(|name| entries.entry(name).or_default()),
            Some((MAP_ROW, name)) => Identifier::parse(name).ok().map(|name| maps.entry(name).or_default()),
            _ => None,
        };
        match target
        {
            Some(values) =>
            {
                if !value.is_empty()
                {
                    values.insert(id, value);
                }
            }
            None => crate::log::warn(crate::log::LOADER, &format!("server registry row {holder} is not understood")),
        }
    }

    SERVED_ENTRIES.with(|served| *served.borrow_mut() = entries);
    SERVED_MAPS.with(|served| *served.borrow_mut() = maps);
}

pub fn restoreSynced()
{
    SERVED_ENTRIES.with(|served| served.borrow_mut().clear());
    SERVED_MAPS.with(|served| served.borrow_mut().clear());
}
