use crate::host::context::stackData;
use crate::registry::Registries;
use game::loot::{LootEntry, LootPool, LootTable, PoolEntry};
use jvm::random::Random;
use mod_protocol::Identifier;
use mod_protocol::abi::{LootTarget, RegistryKind, StackData};
use mod_protocol::spec::{LootEntrySpec, LootPoolSpec, LootSpec};
use std::cell::{Ref, RefCell, RefMut};
use std::collections::BTreeMap;

#[derive(Default)]
struct Merged
{
    m_keepBeta: bool,
    m_entries: Vec<LootEntrySpec>,
    m_pools: Vec<LootPoolSpec>,
}

thread_local!
{
    // every mod's loot for one block or entity merged in load order, a replace drops what came before
    static LOOT: RefCell<BTreeMap<(RegistryKind, Identifier), Merged>> = const { RefCell::new(BTreeMap::new()) };
    // chest tables only mods roll, so they stay as specs and resolve on each roll
    static CHESTS: RefCell<BTreeMap<Identifier, Merged>> = const { RefCell::new(BTreeMap::new()) };
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
            loot.entry((registry, target)).or_insert_with(|| Merged { m_keepBeta: true, ..Merged::default() });

        if spec.replace
        {
            merged.m_keepBeta = false;
            merged.m_entries.clear();
            merged.m_pools.clear();
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
        let mut chests: RefMut<BTreeMap<Identifier, Merged>> = chests.borrow_mut();
        let merged: &mut Merged = chests.entry(table).or_default();

        if spec.replace
        {
            merged.m_entries.clear();
            merged.m_pools.clear();
        }

        merged.m_entries.extend(spec.entries);
    });

    Ok(())
}

// a pool joins a block's or entity's merged loot like its entries do, or a chest table
pub fn addPool(target: LootTarget, id: Identifier, pool: LootPoolSpec) -> Result<(), String>
{
    match target
    {
        LootTarget::Chest =>
        {
            if id.isVanilla()
            {
                return Err(format!("chest loot {id} is beta's own and fixed"));
            }

            CHESTS.with(|chests| chests.borrow_mut().entry(id).or_default().m_pools.push(pool));
        }

        LootTarget::Block | LootTarget::Entity =>
        {
            let registry: RegistryKind = if target == LootTarget::Block
            {
                RegistryKind::Block
            }
            else
            {
                RegistryKind::Entity
            };
            LOOT.with(|loot|
            {
                loot.borrow_mut()
                    .entry((registry, id))
                    .or_insert_with(|| Merged { m_keepBeta: true, ..Merged::default() })
                    .m_pools
                    .push(pool)
            });
        }
    }

    Ok(())
}

pub fn rollChest(table: &Identifier, seed: i64) -> Result<Vec<StackData>, String>
{
    let mut problems: Vec<String> = Vec::new();
    let resolved: Option<LootTable> = CHESTS.with(|chests|
    {
        let chests: Ref<BTreeMap<Identifier, Merged>> = chests.borrow();
        let merged: &Merged = chests.get(table)?;
        Some(crate::registry::with(|registries| build(registries, true, merged, &mut problems)))
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
                .map(|(id, merged)| (id.clone(), build(registries, true, merged, &mut problems)))
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
                .map(|((_, id), merged)| (id.clone(), build(registries, merged.m_keepBeta, merged, &mut problems)))
                .collect()
        })
    })
}

fn build(registries: &Registries, keepBeta: bool, merged: &Merged, problems: &mut Vec<String>) -> LootTable
{
    LootTable
    {
        m_keepBeta: keepBeta,
        m_entries: merged.m_entries.iter().filter_map(|entry| resolve(registries, entry, problems)).collect(),
        m_pools: merged.m_pools.iter().map(|pool| resolvePool(registries, pool, problems)).collect(),
    }
}

// an entry naming an unregistered item keeps its weight and drops nothing, so the other odds stay as written
fn resolvePool(registries: &Registries, pool: &LootPoolSpec, problems: &mut Vec<String>) -> LootPool
{
    let entries: Vec<PoolEntry> = pool
        .entries
        .iter()
        .map(|entry|
        {
            let item: Option<i32> = entry.item.as_ref().and_then(|name|
            {
                let found: Option<i32> = registries.itemId(name);
                if found.is_none()
                {
                    problems.push(format!("loot pool picks {name}, which is not a registered item"));
                }

                found
            });
            PoolEntry
            {
                m_item: item,
                m_meta: entry.meta,
                m_min: entry.count,
                m_max: entry.max.unwrap_or(entry.count),
                m_weight: entry.weight,
                m_player: entry.player,
                m_blockMeta: entry.block_meta,
            }
        })
        .collect();
    LootPool { m_min: pool.rolls, m_max: pool.max_rolls.unwrap_or(pool.rolls), m_entries: entries }
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
                let table: LootTable = build(registries, merged.m_keepBeta, merged, &mut problems);

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
