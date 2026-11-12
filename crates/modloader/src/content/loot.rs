use crate::host::context::stackData;
use crate::registry::Registries;
use game::loot::{LootContext, LootEntry, LootPool, LootRules, LootTable, PoolEntry};
use jvm::random::Random;
use mod_protocol::Identifier;
use mod_protocol::abi::{BlockPos, GuestCall, GuestReply, Handle, LootTarget, RegistryKind, StackData};
use mod_protocol::spec::{ConditionalLootSpec, ConditionalPoolSpec, LootEntrySpec, LootSpec};
use std::cell::{Ref, RefCell, RefMut};
use std::collections::BTreeMap;

// the rule number an entry gets for a name nothing registered, a condition that never passes and a count of none
const UNKNOWN: u32 = u32::MAX;

#[derive(Default)]
struct Merged
{
    m_keepBeta: bool,
    m_entries: Vec<LootEntrySpec>,
    m_pools: Vec<ConditionalPoolSpec>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RuleKind
{
    Condition,
    Count,
}

#[derive(Clone, Copy)]
enum Answer
{
    Guest(usize, Handle),
    KilledByPlayer,
}

// a loot condition or count rule, its number is its place in the list
struct Rule
{
    m_id: Identifier,
    m_label: String,
    m_kind: RuleKind,
    m_answer: Answer,
}

fn builtins() -> Vec<Rule>
{
    vec![Rule
    {
        m_id: Identifier::minecraft("killed_by_player"),
        m_label: "Killed by a player".to_owned(),
        m_kind: RuleKind::Condition,
        m_answer: Answer::KilledByPlayer,
    }]
}

pub fn vanillaNames() -> Vec<String> { builtins().into_iter().map(|rule| rule.m_id.to_string()).collect() }

thread_local!
{
    // every mod's loot for one block or entity merged in load order, a replace drops what came before
    static LOOT: RefCell<BTreeMap<(RegistryKind, Identifier), Merged>> = const { RefCell::new(BTreeMap::new()) };
    // chest tables only mods roll, so they stay as specs and resolve on each roll
    static CHESTS: RefCell<BTreeMap<Identifier, Merged>> = const { RefCell::new(BTreeMap::new()) };
    static RULES: RefCell<Vec<Rule>> = RefCell::new(builtins());
    static OWN: crate::stash::Stash<(LootMap, ChestMap)> = const { crate::stash::Stash::new() };
}

type LootMap = BTreeMap<(RegistryKind, Identifier), Merged>;
type ChestMap = BTreeMap<Identifier, Merged>;
type WireTable = (bool, Vec<LootEntrySpec>, Vec<ConditionalPoolSpec>);

// every merged table, what a server sends a client so recipe viewers show the server's drops
pub type LootWire = (Vec<((RegistryKind, Identifier), WireTable)>, Vec<(Identifier, WireTable)>);

fn toWire(merged: &Merged) -> WireTable { (merged.m_keepBeta, merged.m_entries.clone(), merged.m_pools.clone()) }

fn fromWire((keepBeta, entries, pools): WireTable) -> Merged
{
    Merged { m_keepBeta: keepBeta, m_entries: entries, m_pools: pools }
}

pub fn snapshot() -> LootWire
{
    let tables = LOOT.with(|loot| loot.borrow().iter().map(|(key, merged)| (key.clone(), toWire(merged))).collect());
    let chests =
        CHESTS.with(|chests| chests.borrow().iter().map(|(key, merged)| (key.clone(), toWire(merged))).collect());
    (tables, chests)
}

pub fn replace((tables, chests): LootWire)
{
    let tables: LootMap = tables.into_iter().map(|(key, table)| (key, fromWire(table))).collect();
    let chests: ChestMap = chests.into_iter().map(|(key, table)| (key, fromWire(table))).collect();
    let own: (LootMap, ChestMap) = (LOOT.with(|loot| loot.replace(tables)), CHESTS.with(|slot| slot.replace(chests)));
    OWN.with(|stash| stash.keep(own));
    push(false);
}

pub fn restore()
{
    if let Some((tables, chests)) = OWN.with(crate::stash::Stash::take)
    {
        LOOT.with(|loot| loot.replace(tables));
        CHESTS.with(|slot| slot.replace(chests));
        push(false);
    }
}

pub fn registerRule(
    modIndex: usize,
    id: Identifier,
    label: String,
    kind: RuleKind,
    handler: Handle,
) -> Result<(), String>
{
    crate::registry::claim(RegistryKind::LootCondition, &id)?;
    RULES.with(|rules|
    {
        rules.borrow_mut().push(Rule
        {
            m_id: id,
            m_label: label,
            m_kind: kind,
            m_answer: Answer::Guest(modIndex, handler),
        });
    });
    Ok(())
}

fn ruleNumber(id: &Identifier, kind: RuleKind, problems: &mut Vec<String>) -> u32
{
    let number: Option<usize> =
        RULES.with(|rules| rules.borrow().iter().position(|rule| rule.m_id == *id && rule.m_kind == kind));
    match number
    {
        Some(number) => number as u32,
        None =>
        {
            let noun: &str = match kind
            {
                RuleKind::Condition => "condition",
                RuleKind::Count => "count rule",
            };
            problems.push(format!("loot names {noun} {id}, which is not registered"));
            UNKNOWN
        }
    }
}

// how viewers name a rule
pub fn label(rule: u32) -> String
{
    RULES.with(|rules| rules.borrow().get(rule as usize).map(|rule| rule.m_label.clone())).unwrap_or_default()
}

fn answer(rule: u32) -> Option<Answer>
{
    RULES.with(|rules| rules.borrow().get(rule as usize).map(|rule| rule.m_answer))
}

// a guest sees the dying entity and its killer by id, the broken block by name and the killer's held stack
fn guestContext(context: &LootContext, random: &Random) -> mod_protocol::abi::LootContext
{
    mod_protocol::abi::LootContext
    {
        world: context.m_world.map_or(0, |world| crate::host::context::dimension(world)),
        pos: context.m_pos.map(|[x, y, z]| BlockPos { x, y, z }),
        state: context.m_pos.and_then(|_| crate::states::describe(context.m_block, context.m_blockMeta)),
        entity: context.m_entity.map(|entity| entity.m_entityId.get()),
        killer: context.m_killer.map(|killer| killer.m_entityId.get()),
        tool: context.m_killer.and_then(|killer| killer.getHeldItem()).map(|stack| stackData(&stack)),
        seed: random.nextLong(),
    }
}

fn ask(modIndex: usize, context: &LootContext, call: GuestCall) -> Option<GuestReply>
{
    let runtime: std::rc::Rc<crate::state::ModRuntime> = crate::state::modAt(modIndex)?;
    match context.m_world
    {
        Some(world) => crate::host::context::withWorld(world, || runtime.call(&call)),
        None => runtime.call(&call),
    }
}

pub fn check(rule: u32, context: &LootContext, random: &Random) -> bool
{
    match answer(rule)
    {
        Some(Answer::KilledByPlayer) => context.m_byPlayer,
        Some(Answer::Guest(modIndex, handler)) =>
        {
            let call: GuestCall = GuestCall::LootCondition { handler, context: guestContext(context, random) };
            matches!(ask(modIndex, context, call), Some(GuestReply::Bool(true)))
        }
        None => false,
    }
}

pub fn count(rule: u32, context: &LootContext, random: &Random) -> i32
{
    let Some(Answer::Guest(modIndex, handler)) = answer(rule)
    else
    {
        return 0;
    };

    let call: GuestCall = GuestCall::LootCount { handler, context: guestContext(context, random) };
    match ask(modIndex, context, call)
    {
        Some(GuestReply::Int(count)) => count,
        _ => 0,
    }
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
pub fn addPool(target: LootTarget, id: Identifier, pool: ConditionalPoolSpec) -> Result<(), String>
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
    Ok(game::loot::roll(&resolved, &random, &LootContext::default()).iter().map(|stack| stackData(stack)).collect())
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
fn resolvePool(registries: &Registries, pool: &ConditionalPoolSpec, problems: &mut Vec<String>) -> LootPool
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
                m_rules: rules(entry, problems),
            }
        })
        .collect();
    LootPool { m_min: pool.rolls, m_max: pool.max_rolls.unwrap_or(pool.rolls), m_entries: entries }
}

// an unless naming no rule keeps the entry out, as a when naming none does
fn rules(entry: &ConditionalLootSpec, problems: &mut Vec<String>) -> LootRules
{
    let mut when: Vec<u32> = entry.when.iter().map(|id| ruleNumber(id, RuleKind::Condition, problems)).collect();
    let mut unless: Vec<u32> = Vec::new();

    for id in &entry.unless
    {
        match ruleNumber(id, RuleKind::Condition, problems)
        {
            UNKNOWN => when.push(UNKNOWN),
            number => unless.push(number),
        }
    }

    let count: Option<u32> = entry.count_by.as_ref().map(|id| ruleNumber(id, RuleKind::Count, problems));
    LootRules { m_when: when, m_unless: unless, m_count: count }
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
