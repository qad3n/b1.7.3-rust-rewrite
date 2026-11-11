use super::ANY_DAMAGE;
use crate::content;
use crate::registry::{BlockEntry, EntityEntry, Registries, Row, UNASSIGNED};
use game::block::Block;
use game::loot::{LootEntry, LootPool, LootRules, LootTable};
use jvm::random::Random;
use mod_protocol::Identifier;
use mod_protocol::abi::{LootDrop, LootSource, LootView, StackData};
use mod_protocol::spec::DropSpec;
use std::collections::{BTreeMap, HashMap};
use std::rc::Rc;

// rolls of each block metadata's drop code, enough to tell the 1 in 20 sapling of leaves from none
const SAMPLES: i32 = 1000;
const METAS: i32 = 16;
const DUNGEON: &str = "dungeon";
const CHEST: &str = "chest";
// WorldGenDungeons.generate makes 8 picks for each chest
const DUNGEON_PICKS: i32 = 8;
// pickCheckLootItem rolls one of 11 outcomes per pick
const PICK: f32 = 1.0 / 11.0;

mod note
{
    pub const BURNING: &str = "When it dies burning";
    pub const SKELETON: &str = "When a skeleton's arrow kills it";
    pub const SMALLEST: &str = "Only the smallest size";
    pub const UNSHEARED: &str = "Only unsheared, in its color";
}

// a drop beta rolls in code, written out as the viewer shows it
struct Drop
{
    m_item: &'static str,
    m_damage: i32,
    m_min: i32,
    m_max: i32,
    m_chance: f32,
    m_note: Option<&'static str>,
}

impl Drop
{
    // EntityLiving.dropFewItems drops nextInt(3) of getDropItemId
    const fn few(item: &'static str) -> Self { Self::new(item, 0, 2, 1.0) }

    const fn new(item: &'static str, min: i32, max: i32, chance: f32) -> Self
    {
        Self { m_item: item, m_damage: 0, m_min: min, m_max: max, m_chance: chance, m_note: None }
    }

    const fn damage(self, damage: i32) -> Self { Self { m_damage: damage, ..self } }

    const fn note(self, note: &'static str) -> Self { Self { m_note: Some(note), ..self } }
}

struct Mob
{
    m_key: &'static str,
    m_drops: &'static [Drop],
}

// getDropItemId and the dropFewItems overrides of every vanilla mob that drops anything
const MOBS: [Mob; 12] = [
    Mob
    {
        m_key: "Creeper",
        m_drops: &[
            Drop::few("gunpowder"),
            Drop::new("record_13", 1, 1, 0.5).note(note::SKELETON),
            Drop::new("record_cat", 1, 1, 0.5).note(note::SKELETON),
        ],
    },
    Mob { m_key: "Skeleton", m_drops: &[Drop::few("arrow"), Drop::few("bone")] },
    Mob { m_key: "Spider", m_drops: &[Drop::few("string")] },
    Mob { m_key: "Zombie", m_drops: &[Drop::few("feather")] },
    Mob { m_key: "Slime", m_drops: &[Drop::few("slime_ball").note(note::SMALLEST)] },
    Mob { m_key: "Ghast", m_drops: &[Drop::few("gunpowder")] },
    Mob { m_key: "PigZombie", m_drops: &[Drop::few("cooked_porkchop")] },
    Mob { m_key: "Pig", m_drops: &[Drop::few("porkchop"), Drop::few("cooked_porkchop").note(note::BURNING)] },
    Mob { m_key: "Sheep", m_drops: &[Drop::new("wool", 1, 1, 1.0).damage(ANY_DAMAGE).note(note::UNSHEARED)] },
    Mob { m_key: "Cow", m_drops: &[Drop::few("leather")] },
    Mob { m_key: "Chicken", m_drops: &[Drop::few("feather")] },
    Mob { m_key: "Squid", m_drops: &[Drop::new("dye", 1, 3, 1.0)] },
];

// WorldGenDungeons.pickCheckLootItem, the chance of each outcome in one pick
const DUNGEON_LOOT: [Drop; 12] = [
    Drop::new("saddle", 1, 1, PICK),
    Drop::new("iron_ingot", 1, 4, PICK),
    Drop::new("bread", 1, 1, PICK),
    Drop::new("wheat", 1, 4, PICK),
    Drop::new("gunpowder", 1, 4, PICK),
    Drop::new("string", 1, 4, PICK),
    Drop::new("bucket", 1, 1, PICK),
    Drop::new("golden_apple", 1, 1, PICK / 100.0),
    Drop::new("redstone", 1, 4, PICK / 2.0),
    Drop::new("record_13", 1, 1, PICK / 20.0),
    Drop::new("record_cat", 1, 1, PICK / 20.0),
    Drop::new("dye", 1, 1, PICK).damage(3),
];

#[derive(Clone, Copy)]
struct Tally
{
    m_min: i32,
    m_max: i32,
    m_hits: i32,
}

// beta's drops as the loader describes them, then what mods add, in block, entity and chest order
pub fn list() -> Vec<LootView>
{
    let entityTables: HashMap<Identifier, LootTable> = content::loot::entities().into_iter().collect();

    let mut views: Vec<LootView> = crate::registry::with(|registries|
    {
        let mut views: Vec<LootView> = blocks(registries);
        views.extend(entities(registries, &entityTables));
        views.extend(dungeon(registries));
        views
    });

    views.extend(content::loot::chests().into_iter().map(|(table, loot)| LootView
    {
        source: LootSource::Chest { table },
        rolls: 1,
        sampled: false,
        drops: loot.m_entries.iter().map(entryDrop).chain(tableDrops(&loot)).collect(),
    }));

    views.retain(|view| !view.drops.is_empty());
    views
}

fn blocks(registries: &Registries) -> Vec<LootView>
{
    registries
        .m_blocks
        .rows()
        .filter(|row| row.m_numeric != UNASSIGNED && row.m_value.m_page.is_none())
        .flat_map(|row| blockViews(registries, row))
        .collect()
}

// one view per metadata that drops something of its own, metadata dropping the same as a lower one is left out
fn blockViews(registries: &Registries, row: &Row<BlockEntry>) -> Vec<LootView>
{
    let Some(block) = Block::byId(row.m_numeric)
    else
    {
        return Vec::new();
    };

    let table: Option<Rc<LootTable>> = game::loot::block(row.m_numeric);
    let keepBeta: bool = table.as_ref().is_none_or(|table| table.m_keepBeta);
    let specDrops: Option<&Vec<DropSpec>> = row.m_value.m_spec.as_ref().and_then(|spec| spec.drops.as_ref());
    // spec drops and beta drops ignore metadata except through the beta drop code
    let metas: i32 = if specDrops.is_some() && table.is_none()
    {
        1
    }
    else
    {
        METAS
    };

    let mut views: Vec<LootView> = Vec::new();
    let mut seen: Vec<Vec<LootDrop>> = Vec::new();

    for meta in 0..metas
    {
        let mut drops: Vec<LootDrop> = match (keepBeta, specDrops)
        {
            (false, _) => Vec::new(),
            (true, Some(specDrops)) => specDrops.iter().filter_map(|drop| specDrop(registries, drop)).collect(),
            (true, None) => sample(&block, meta),
        };

        if let Some(table) = &table
        {
            let matching = table.m_entries.iter().filter(|entry| entry.m_blockMeta.is_none_or(|only| only == meta));
            drops.extend(matching.map(entryDrop));
            drops.extend(tableDrops(table));
        }

        // a block dropping itself tells the viewer nothing new
        drops.retain(|drop| drop.stack.id != row.m_numeric);

        if drops.is_empty() || seen.contains(&drops)
        {
            continue;
        }

        seen.push(drops.clone());
        views.push(LootView
        {
            source: LootSource::Block { block: row.m_numeric, meta },
            rolls: 1,
            sampled: keepBeta && specDrops.is_none(),
            drops,
        });
    }

    views
}

// rolls dropBeta's loop many times with a fixed seed, idDropped then damageDropped for each of quantityDropped
fn sample(block: &Rc<Block>, meta: i32) -> Vec<LootDrop>
{
    let random: Random = Random::withSeed(0);
    let mut tallies: BTreeMap<(i32, i32), Tally> = BTreeMap::new();

    for _ in 0..SAMPLES
    {
        let mut rolled: BTreeMap<(i32, i32), i32> = BTreeMap::new();

        for _ in 0..block.quantityDropped(&random)
        {
            let id: i32 = block.idDropped(meta, &random);
            if id > 0
            {
                *rolled.entry((id, block.damageDropped(meta))).or_default() += 1;
            }
        }

        for (key, count) in rolled
        {
            let tally: &mut Tally = tallies.entry(key).or_insert(Tally { m_min: count, m_max: count, m_hits: 0 });
            tally.m_min = tally.m_min.min(count);
            tally.m_max = tally.m_max.max(count);
            tally.m_hits += 1;
        }
    }

    tallies
        .into_iter()
        .map(|((id, damage), tally)| LootDrop
        {
            stack: StackData { id, count: tally.m_min, damage },
            max: tally.m_max,
            chance: tally.m_hits as f32 / SAMPLES as f32,
            player: false,
            note: None,
        })
        .collect()
}

fn entities(registries: &Registries, tables: &HashMap<Identifier, LootTable>) -> Vec<LootView>
{
    registries
        .m_entities
        .rows()
        .filter(|row| row.m_numeric != UNASSIGNED && super::mob(row))
        .map(|row|
        {
            let table: Option<&LootTable> = tables.get(&row.m_id);
            let mut drops: Vec<LootDrop> = if table.is_none_or(|table| table.m_keepBeta)
            {
                betaMobDrops(registries, row)
            }
            else
            {
                Vec::new()
            };

            drops.extend(table.iter().flat_map(|table| table.m_entries.iter().map(entryDrop)));
            drops.extend(table.iter().flat_map(|table| tableDrops(table)));
            LootView { source: LootSource::Entity { entity: row.m_id.clone() }, rolls: 1, sampled: false, drops }
        })
        .collect()
}

// a mod entity's spec drops, or the vanilla mob's own
fn betaMobDrops(registries: &Registries, row: &Row<EntityEntry>) -> Vec<LootDrop>
{
    match &row.m_value.m_spec
    {
        Some(spec) => spec.drops.iter().flatten().filter_map(|drop| specDrop(registries, drop)).collect(),
        None => MOBS
            .iter()
            .find(|mob| mob.m_key == row.m_value.m_saveKey)
            .map(|mob| written(registries, mob.m_drops))
            .unwrap_or_default(),
    }
}

fn dungeon(registries: &Registries) -> Option<LootView>
{
    let table: Identifier = Identifier::minecraft(DUNGEON);
    registries.itemId(&Identifier::minecraft(CHEST))?;

    Some(LootView
    {
        source: LootSource::Chest { table },
        rolls: DUNGEON_PICKS,
        sampled: false,
        drops: written(registries, &DUNGEON_LOOT),
    })
}

fn written(registries: &Registries, drops: &[Drop]) -> Vec<LootDrop>
{
    drops
        .iter()
        .filter_map(|drop|
        {
            let Some(id) = registries.itemId(&Identifier::minecraft(drop.m_item))
            else
            {
                crate::log::warn(
                    crate::log::LOADER,
                    &format!("viewer loot names {}, which is not registered", drop.m_item),
                );
                return None;
            };

            Some(LootDrop
            {
                stack: StackData { id, count: drop.m_min, damage: drop.m_damage },
                max: drop.m_max,
                chance: drop.m_chance,
                player: false,
                note: drop.m_note.map(str::to_owned),
            })
        })
        .collect()
}

// a spec drop missing its item was already reported when its block or entity was built
fn specDrop(registries: &Registries, drop: &DropSpec) -> Option<LootDrop>
{
    let id: i32 = registries.itemId(&drop.item)?;
    let (min, max): (i32, i32) = match drop.count.as_slice()
    {
        [count] => (*count, *count),
        [min, max] => (*min, *max),
        _ => (1, 1),
    };

    Some(LootDrop
    {
        stack: StackData { id, count: min, damage: drop.meta },
        max,
        chance: drop.chance,
        player: false,
        note: None,
    })
}

// each pick of every weighted pool, its chance per roll and the rolls in its note
fn tableDrops(table: &LootTable) -> Vec<LootDrop> { table.m_pools.iter().flat_map(poolDrops).collect() }

fn poolDrops(pool: &LootPool) -> Vec<LootDrop>
{
    let total: i32 = pool.m_entries.iter().map(|entry| entry.m_weight.max(0)).sum();
    let rolls: String = if pool.m_max > pool.m_min
    {
        format!("{} to {}", pool.m_min, pool.m_max)
    }
    else
    {
        pool.m_min.to_string()
    };

    pool.m_entries
        .iter()
        .filter(|entry| total > 0 && entry.m_weight > 0)
        .filter_map(|entry|
        {
            Some(LootDrop
            {
                stack: StackData { id: entry.m_item?, count: entry.m_min, damage: entry.m_meta },
                max: entry.m_max,
                chance: entry.m_weight as f32 / total as f32,
                player: entry.m_player,
                note: Some(format!("One pick of a weighted pool, rolled {rolls} times{}", ruleNote(&entry.m_rules))),
            })
        })
        .collect()
}

// the labels of an entry's conditions and count rule, so a viewer shows what a drop needs
fn ruleNote(rules: &LootRules) -> String
{
    let names =
        |list: &[u32]| list.iter().map(|rule| crate::content::loot::label(*rule)).collect::<Vec<String>>().join(", ");
    let mut note: String = String::new();

    if !rules.m_when.is_empty()
    {
        note.push_str(&format!(", only when {}", names(&rules.m_when)));
    }
    if !rules.m_unless.is_empty()
    {
        note.push_str(&format!(", never when {}", names(&rules.m_unless)));
    }
    if let Some(rule) = rules.m_count
    {
        note.push_str(&format!(", count from {}", crate::content::loot::label(rule)));
    }

    note
}

fn entryDrop(entry: &LootEntry) -> LootDrop
{
    LootDrop
    {
        stack: StackData { id: entry.m_item, count: entry.m_min, damage: entry.m_meta },
        max: entry.m_max,
        chance: entry.m_chance,
        player: entry.m_player,
        note: None,
    }
}
