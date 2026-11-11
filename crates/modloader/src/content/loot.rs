use crate::host::context::stackData;
use crate::registry::Registries;
use game::loot::{LootEntry, LootTable};
use jvm::random::Random;
use mod_protocol::Identifier;
use mod_protocol::abi::{RegistryKind, StackData};
use mod_protocol::spec::{LootEntrySpec, LootSpec};
use std::cell::{Ref, RefCell, RefMut};
use std::collections::BTreeMap;

struct Merged
{
    m_keepBeta: bool,
    m_entries: Vec<LootEntrySpec>,
}

thread_local!
{
    // every mod's loot for one block or entity merged in load order, a replace drops what came before
    static LOOT: RefCell<BTreeMap<(RegistryKind, Identifier), Merged>> = const { RefCell::new(BTreeMap::new()) };
    // chest tables only mods roll, so they stay as specs and resolve on each roll
    static CHESTS: RefCell<BTreeMap<Identifier, Vec<LootEntrySpec>>> = const { RefCell::new(BTreeMap::new()) };
}

pub fn add(registry: RegistryKind, target: Identifier, spec: LootSpec) -> Result<(), String>
{
    if !matches!(registry, RegistryKind::Block | RegistryKind::Entity)
    {
        return Err("loot is for blocks and entities".to_owned());
    }

    LOOT.with(|loot|
    {
        let mut loot: RefMut<BTreeMap<(RegistryKind, Identifier), Merged>> = loot.borrow_mut();
        let merged: &mut Merged =
            loot.entry((registry, target)).or_insert_with(|| Merged { m_keepBeta: true, m_entries: Vec::new() });

        if spec.replace
        {
            merged.m_keepBeta = false;
            merged.m_entries.clear();
        }

        merged.m_entries.extend(spec.entries);
    });

    Ok(())
}

pub fn addChest(table: Identifier, spec: LootSpec) -> Result<(), String>
{
    if table.isVanilla()
    {
        return Err(format!("chest loot {table} is beta's own and fixed"));
    }

    CHESTS.with(|chests|
    {
        let mut chests: RefMut<BTreeMap<Identifier, Vec<LootEntrySpec>>> = chests.borrow_mut();
        let entries: &mut Vec<LootEntrySpec> = chests.entry(table).or_default();

        if spec.replace
        {
            entries.clear();
        }

        entries.extend(spec.entries);
    });

    Ok(())
}

pub fn rollChest(table: &Identifier, seed: i64) -> Result<Vec<StackData>, String>
{
    let mut problems: Vec<String> = Vec::new();
    let resolved: Option<LootTable> = CHESTS.with(|chests|
    {
        let chests: Ref<BTreeMap<Identifier, Vec<LootEntrySpec>>> = chests.borrow();
        let entries: &Vec<LootEntrySpec> = chests.get(table)?;
        Some(crate::registry::with(|registries| build(registries, true, entries, &mut problems)))
    });
    warnAll(problems);

    let resolved: LootTable = resolved.ok_or_else(|| format!("no chest loot table {table}"))?;
    let random: Random = Random::withSeed(seed);
    Ok(game::loot::roll(&resolved, &random, false, 0).iter().map(|stack| stackData(stack)).collect())
}

// every chest table with its items resolved, an entry naming an unregistered item is left out
pub fn chests() -> Vec<(Identifier, LootTable)>
{
    let mut problems: Vec<String> = Vec::new();
    let tables: Vec<(Identifier, LootTable)> = CHESTS.with(|chests|
    {
        crate::registry::with(|registries|
        {
            chests
                .borrow()
                .iter()
                .map(|(id, entries)| (id.clone(), build(registries, true, entries, &mut problems)))
                .collect()
        })
    });

    warnAll(problems);
    tables
}

// the merged loot of every entity a mod gave loot, beta drops kept unless a mod replaced them
pub fn entities() -> Vec<(Identifier, LootTable)>
{
    // push already warned about every entry left out here
    let mut problems: Vec<String> = Vec::new();

    LOOT.with(|loot|
    {
        crate::registry::with(|registries|
        {
            loot.borrow()
                .iter()
                .filter(|((registry, _), _)| *registry == RegistryKind::Entity)
                .map(|((_, id), merged)|
                {
                    (id.clone(), build(registries, merged.m_keepBeta, &merged.m_entries, &mut problems))
                })
                .collect()
        })
    })
}

fn build(registries: &Registries, keepBeta: bool, entries: &[LootEntrySpec], problems: &mut Vec<String>) -> LootTable
{
    LootTable
    {
        m_keepBeta: keepBeta,
        m_entries: entries.iter().filter_map(|entry| resolve(registries, entry, problems)).collect(),
    }
}

// the game keys loot by numbers and save keys, so every content build and id map change resolves it again
pub fn push(report: bool)
{
    game::loot::clear();

    let mut problems: Vec<String> = Vec::new();

    LOOT.with(|loot|
    {
        crate::registry::with(|registries|
        {
            for ((registry, target), merged) in loot.borrow().iter()
            {
                let table: LootTable = build(registries, merged.m_keepBeta, &merged.m_entries, &mut problems);

                match registry
                {
                    RegistryKind::Block => match registries.m_blocks.numeric(target)
                    {
                        Some(block) => game::loot::setBlock(block, table),
                        None => problems.push(format!("loot for block {target}, which is not registered")),
                    },

                    _ => match registries.m_entities.get(target)
                    {
                        Some(row) => game::loot::setEntity(&row.m_value.m_saveKey, table),
                        None => problems.push(format!("loot for entity {target}, which is not registered")),
                    },
                }
            }
        })
    });

    if report
    {
        warnAll(problems);
    }
}

fn warnAll(problems: Vec<String>)
{
    for problem in problems
    {
        crate::log::warn(crate::log::LOADER, &problem);
    }
}

fn resolve(registries: &Registries, entry: &LootEntrySpec, problems: &mut Vec<String>) -> Option<LootEntry>
{
    let Some(item) = registries.itemId(&entry.item)
    else
    {
        problems.push(format!("loot drops {}, which is not a registered item", entry.item));
        return None;
    };

    Some(LootEntry
    {
        m_item: item,
        m_meta: entry.meta,
        m_min: entry.count,
        m_max: entry.max.unwrap_or(entry.count),
        m_chance: entry.chance,
        m_player: entry.player,
        m_blockMeta: entry.block_meta,
    })
}
