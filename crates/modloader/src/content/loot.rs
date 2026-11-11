use crate::registry::Registries;
use game::loot::{LootEntry, LootTable};
use mod_protocol::Identifier;
use mod_protocol::abi::RegistryKind;
use mod_protocol::spec::{LootEntrySpec, LootSpec};
use std::cell::{RefCell, RefMut};
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
                let table: LootTable = LootTable
                {
                    m_keepBeta: merged.m_keepBeta,
                    m_entries: merged
                        .m_entries
                        .iter()
                        .filter_map(|entry| resolve(registries, entry, &mut problems))
                        .collect(),
                };

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

    if !report
    {
        return;
    }

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
