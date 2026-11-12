use super::data::{self, Owner};
use game::resources::Source;
use mod_protocol::Identifier;
use mod_protocol::abi::RegistryKind;
use mod_protocol::data::{self as files, DATA, Parsed};
use mod_protocol::spec::LoadConditions;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::Path;

// data files whose conditions or targets need every mod registered first, in the order they were read
struct Waiting
{
    m_owner: Owner,
    m_name: String,
    m_parsed: Parsed,
}

thread_local!
{
    static WAITING: RefCell<Vec<Waiting>> = const { RefCell::new(Vec::new()) };
    // the override folder's data files, shown in the mods menu
    static OVERRIDES: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
}

pub fn overrides() -> Vec<String> { OVERRIDES.with(|files| files.borrow().clone()) }

pub fn push(owner: &Owner, name: String, parsed: Parsed)
{
    let owner: Owner = Owner { m_id: owner.m_id.clone(), m_index: owner.m_index };
    WAITING.with(|waiting| waiting.borrow_mut().push(Waiting { m_owner: owner, m_name: name, m_parsed: parsed }));
}

// config/data holds data/<namespace>/<kind>/<path>.toml of any namespace, read after every mod so it wins
pub fn loadOverrides(config: &Path) -> Vec<(String, String)>
{
    let root: std::path::PathBuf = config.join("data");
    if !root.is_dir()
    {
        return Vec::new();
    }

    let mut source: Source = match Source::open(config)
    {
        Ok(source) => source,
        Err(error) => return vec![(crate::log::LOADER.to_owned(), format!("{}: {error}", root.display()))],
    };

    let mut problems: Vec<(String, String)> = Vec::new();
    let names: Vec<String> = source.list(DATA).into_iter().filter(|name| files::isData(name)).collect();

    for name in &names
    {
        let parsed: Result<Parsed, String> =
            data::read(&mut source, name).and_then(|text| files::parse(None, name, &text));
        match parsed
        {
            Ok(parsed) =>
            {
                let namespace: String = name.trim_start_matches(DATA).split('/').next().unwrap_or_default().to_owned();
                push(&Owner { m_id: namespace, m_index: None }, format!("config/{name}"), parsed);
            }

            Err(reason) => problems.push((crate::log::LOADER.to_owned(), format!("config/{reason}"))),
        }
    }

    crate::log::info(crate::log::LOADER, &format!("{} data override files in {}", names.len(), root.display()));
    OVERRIDES.with(|files| *files.borrow_mut() = names);
    problems
}

// every mod registered, so conditions can be judged and waiting files applied in their read order
pub fn settle() -> Vec<(String, String)>
{
    let waiting: Vec<Waiting> = WAITING.with(|waiting| waiting.borrow_mut().drain(..).collect());
    let mut problems: Vec<(String, String)> = Vec::new();

    for Waiting { m_owner: owner, m_name: name, m_parsed: parsed } in waiting
    {
        if let Some(when) = &parsed.when
            && let Some(reason) = unmet(when)
        {
            crate::log::debug(&owner.m_id, &format!("{name} skipped, {reason}"));
            continue;
        }

        if let Err(reason) = data::apply(&owner, parsed.file)
        {
            problems.push((owner.m_id.clone(), format!("{name}: {reason}")));
        }
    }

    problems
}

// the first condition that does not hold, none when the file loads
fn unmet(when: &LoadConditions) -> Option<String>
{
    let active = |modid: &str| crate::state::byId(modid).is_some_and(|runtime| runtime.isActive());

    if let Some(modid) = when.loaded.iter().find(|modid| !active(modid))
    {
        return Some(format!("{modid} is not loaded"));
    }

    if let Some(modid) = when.missing.iter().find(|modid| active(modid))
    {
        return Some(format!("{modid} is loaded"));
    }

    let registered =
        |kind: RegistryKind, id: &Identifier| crate::registry::with(|registries| registries.contains(kind, id));
    let filled = |kind: RegistryKind, tag: &Identifier| !super::tags::values(kind, tag).is_empty();

    first(&when.registered, |kind, id| (!registered(kind, id)).then(|| format!("{id} is not registered")))
        .or_else(|| first(&when.tag_empty, |kind, tag| filled(kind, tag).then(|| format!("#{tag} has members"))))
        .or_else(|| first(&when.tag_filled, |kind, tag| (!filled(kind, tag)).then(|| format!("#{tag} is empty"))))
}

// conditions are keyed by registry folder, an unknown folder fails the condition
fn first(
    table: &BTreeMap<String, Vec<Identifier>>,
    check: impl Fn(RegistryKind, &Identifier) -> Option<String>,
) -> Option<String>
{
    table.iter().find_map(|(folder, ids)| match RegistryKind::byFolder(folder)
    {
        Some(kind) => ids.iter().find_map(|id| check(kind, id)),
        None => Some(format!("{folder} is not a registry")),
    })
}
