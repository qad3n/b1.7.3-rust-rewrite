use crate::content::tags;
use crate::{log, registry};
use mod_protocol::Identifier;
use mod_protocol::abi::RegistryKind;
use std::collections::{BTreeMap, BTreeSet};

// the loader groups materials itself from item and block names, so mods that never tag still merge; what a name
// says goes into the same c: tags mods fill by hand
const COMMON: &str = "c";
const UNIFY: &str = "unify";

#[derive(Clone, Copy, PartialEq, Eq)]
enum Affix
{
    Prefix(&'static str),
    Suffix(&'static str),
}

// a name shape and the tag folder it means; gated kinds only count for materials a plain kind already found, so
// a note block or a fishing rod never passes for a metal
struct Rule
{
    m_kind: &'static str,
    m_affix: Affix,
    m_gated: bool,
}

const RULES: [Rule; 16] = [
    Rule { m_kind: "ingots", m_affix: Affix::Suffix("_ingot"), m_gated: false },
    Rule { m_kind: "ingots", m_affix: Affix::Prefix("ingot_"), m_gated: false },
    Rule { m_kind: "nuggets", m_affix: Affix::Suffix("_nugget"), m_gated: false },
    Rule { m_kind: "nuggets", m_affix: Affix::Prefix("nugget_"), m_gated: false },
    Rule { m_kind: "dusts", m_affix: Affix::Suffix("_dust"), m_gated: false },
    Rule { m_kind: "dusts", m_affix: Affix::Prefix("dust_"), m_gated: false },
    Rule { m_kind: "gems", m_affix: Affix::Suffix("_gem"), m_gated: false },
    Rule { m_kind: "gems", m_affix: Affix::Prefix("gem_"), m_gated: false },
    Rule { m_kind: "ores", m_affix: Affix::Suffix("_ore"), m_gated: false },
    Rule { m_kind: "ores", m_affix: Affix::Prefix("ore_"), m_gated: false },
    Rule { m_kind: "gears", m_affix: Affix::Suffix("_gear"), m_gated: false },
    Rule { m_kind: "plates", m_affix: Affix::Suffix("_plate"), m_gated: true },
    Rule { m_kind: "rods", m_affix: Affix::Suffix("_rod"), m_gated: true },
    Rule { m_kind: "storage_blocks", m_affix: Affix::Suffix("_block"), m_gated: true },
    Rule { m_kind: "storage_blocks", m_affix: Affix::Prefix("block_"), m_gated: true },
    Rule { m_kind: "raw_materials", m_affix: Affix::Prefix("raw_"), m_gated: true },
];

// vanilla materials whose names carry no shape, as beta names them
const VANILLA: [(&str, &str); 2] = [("diamond", "gems/diamond"), ("redstone", "dusts/redstone")];

// the same material under the names mods give it
const ALIASES: [(&str, &str); 3] = [("golden", "gold"), ("wooden", "wood"), ("aluminium", "aluminum")];

// lit and glowing variants are the same block in another state, never a member of their own
const SKIPPED: [&str; 2] = ["lit_", "glowing_"];

fn material(path: &str, affix: Affix) -> Option<&str>
{
    let found: &str = match affix
    {
        Affix::Prefix(prefix) => path.strip_prefix(prefix)?,
        Affix::Suffix(suffix) => path.strip_suffix(suffix)?,
    };

    let plain: bool = !found.is_empty() && found.bytes().all(|byte| byte.is_ascii_lowercase() || byte == b'_');
    if !plain
    {
        return None;
    }

    Some(ALIASES.iter().find(|(name, _)| *name == found).map_or(found, |(_, alias)| alias))
}

// the tags one name reads as, a name two different kinds claim is left out and logged
fn tagOf(id: &Identifier, gated: bool, materials: &BTreeSet<String>) -> Option<Identifier>
{
    let path: &str = id.path();
    if SKIPPED.iter().any(|prefix| path.starts_with(prefix))
    {
        return None;
    }

    if let Some((_, tag)) = VANILLA.iter().find(|(name, _)| id.isVanilla() && *name == path)
    {
        return Identifier::parse(&format!("{COMMON}:{tag}")).ok().filter(|_| !gated);
    }

    let claims: BTreeSet<(&str, &str)> = RULES
        .iter()
        .filter(|rule| rule.m_gated == gated)
        .filter_map(|rule| material(path, rule.m_affix).map(|found| (rule.m_kind, found)))
        .filter(|(_, found)| !gated || materials.contains(*found))
        .collect();

    match claims.len()
    {
        0 => None,
        1 => claims
            .into_iter()
            .next()
            .and_then(|(kind, found)| Identifier::parse(&format!("{COMMON}:{kind}/{found}")).ok()),

        _ =>
        {
            let read: Vec<String> = claims.iter().map(|(kind, found)| format!("{kind}/{found}")).collect();
            log::warn(UNIFY, &format!("{id} reads as {}, it joins none of them", read.join(" and ")));
            None
        }
    }
}

fn names(registry: RegistryKind) -> Vec<Identifier>
{
    registry::with(|registries| match registry
    {
        RegistryKind::Block => registries.m_blocks.rows().map(|row| row.m_id.clone()).collect(),
        _ => registries.m_items.rows().map(|row| row.m_id.clone()).collect(),
    })
}

fn materialOf(tag: &Identifier) -> Option<String> { tag.path().split_once('/').map(|(_, found)| found.to_owned()) }

// the tags every name of a registry reads as, plain kinds first so gated kinds know their materials
fn inferRegistry(registry: RegistryKind) -> BTreeMap<Identifier, Vec<Identifier>>
{
    let all: Vec<Identifier> = names(registry);
    let mut tagged: BTreeMap<Identifier, Vec<Identifier>> = BTreeMap::new();
    let mut materials: BTreeSet<String> = BTreeSet::new();

    for gated in [false, true]
    {
        for id in &all
        {
            if let Some(tag) = tagOf(id, gated, &materials)
            {
                tagged.entry(tag).or_default().push(id.clone());
            }
        }

        materials.extend(tagged.keys().filter_map(materialOf));
    }

    tagged
}

// a name the loader read differently from the tag its own mod gave it is pointed out, the mod's tag still counts
fn checkAgainstMods(registry: RegistryKind, inferred: &BTreeMap<Identifier, Vec<Identifier>>)
{
    for (tag, members) in inferred
    {
        let kind: &str = tag.path().split('/').next().unwrap_or_default();

        for member in members
        {
            let explicit: Vec<Identifier> = tags::of(registry, member)
                .into_iter()
                .filter(|other|
                {
                    other.namespace() == COMMON && other != tag && other.path().starts_with(&format!("{kind}/"))
                })
                .collect();
            if !explicit.is_empty()
            {
                let named: Vec<String> = explicit.iter().map(Identifier::to_string).collect();
                log::warn(
                    UNIFY,
                    &format!(
                        "{registry:?} {member} is tagged {} by its mod but its name reads as {tag}",
                        named.join(", ")
                    ),
                );
            }
        }
    }
}

// runs at startup before tags reach the game, every registered block and item read by name
pub fn run()
{
    for registry in [RegistryKind::Item, RegistryKind::Block]
    {
        let inferred: BTreeMap<Identifier, Vec<Identifier>> = inferRegistry(registry);
        checkAgainstMods(registry, &inferred);

        let count: usize = inferred.values().map(Vec::len).sum();
        log::debug(
            UNIFY,
            &format!("{registry:?} names read into {count} memberships of {} shared tags", inferred.len()),
        );

        for (tag, members) in inferred
        {
            tags::add(registry, tag, members, false);
        }
    }
}
