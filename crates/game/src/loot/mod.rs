use crate::entity::Entity;
use crate::item::ItemStack;
use crate::world::World;
use jvm::random::Random;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

// one possible drop, rolled on its own
#[derive(Clone, PartialEq, Debug)]
pub struct LootEntry
{
    pub m_item: i32,
    pub m_meta: i32,
    pub m_min: i32,
    pub m_max: i32,
    pub m_chance: f32,
    // drops only when a player killed the entity, blocks ignore it
    pub m_player: bool,
    // drops only from a block with this metadata, entities ignore it
    pub m_blockMeta: Option<i32>,
}

// one pick of a pool, no item drops nothing; an entry whose condition fails sits out the roll
#[derive(Clone, PartialEq, Debug)]
pub struct PoolEntry
{
    pub m_item: Option<i32>,
    pub m_meta: i32,
    pub m_min: i32,
    pub m_max: i32,
    pub m_weight: i32,
    pub m_player: bool,
    pub m_blockMeta: Option<i32>,
    pub m_rules: LootRules,
}

// conditions a mod registered that must all pass or all fail, and its rule for the count, by the loader's numbers
#[derive(Clone, PartialEq, Debug, Default)]
pub struct LootRules
{
    pub m_when: Vec<u32>,
    pub m_unless: Vec<u32>,
    pub m_count: Option<u32>,
}

// what one roll knows about the drop, for the conditions and count rules of mods
thread_local!
{
    // the player whose dig is dropping a block, so block loot can ask about them and their tool
    static HARVESTER: RefCell<Option<Rc<Entity>>> = const { RefCell::new(None) };
}

pub fn harvesting<R>(player: &Rc<Entity>, body: impl FnOnce() -> R) -> R
{
    let outer: Option<Rc<Entity>> = HARVESTER.with(|slot| slot.replace(Some(Rc::clone(player))));
    let result: R = body();

    HARVESTER.with(|slot| slot.replace(outer));
    result
}

pub fn harvester() -> Option<Rc<Entity>> { HARVESTER.with(|slot| slot.borrow().clone()) }

#[derive(Clone, Copy, Default)]
pub struct LootContext<'a>
{
    pub m_world: Option<&'a Rc<World>>,
    pub m_pos: Option<[i32; 3]>,
    pub m_block: i32,
    pub m_blockMeta: i32,
    pub m_entity: Option<&'a Rc<Entity>>,
    pub m_killer: Option<&'a Rc<Entity>>,
    pub m_byPlayer: bool,
}

impl LootContext<'_>
{
    fn allows(&self, player: bool, blockMeta: Option<i32>) -> bool
    {
        (!player || self.m_byPlayer) && blockMeta.is_none_or(|meta| meta == self.m_blockMeta)
    }
}

impl LootRules
{
    fn pass(&self, context: &LootContext, random: &Random) -> bool
    {
        self.m_when.iter().all(|rule| crate::hooks::lootCondition(*rule, context, random))
            && !self.m_unless.iter().any(|rule| crate::hooks::lootCondition(*rule, context, random))
    }
}

// rolled between min and max times, each roll picking one entry by weight
#[derive(Clone, PartialEq, Debug)]
pub struct LootPool
{
    pub m_min: i32,
    pub m_max: i32,
    pub m_entries: Vec<PoolEntry>,
}

// a block or entity without a table drops exactly as beta, a table adds its entries and pools or replaces the beta
// drops
#[derive(Clone, PartialEq, Debug, Default)]
pub struct LootTable
{
    pub m_keepBeta: bool,
    pub m_entries: Vec<LootEntry>,
    pub m_pools: Vec<LootPool>,
}

thread_local!
{
    static BLOCKS: RefCell<HashMap<i32, Rc<LootTable>>> = RefCell::new(HashMap::new());
    static ENTITIES: RefCell<HashMap<String, Rc<LootTable>>> = RefCell::new(HashMap::new());
}

pub fn block(id: i32) -> Option<Rc<LootTable>> { BLOCKS.with(|tables| tables.borrow().get(&id).cloned()) }

pub fn entity(key: &str) -> Option<Rc<LootTable>> { ENTITIES.with(|tables| tables.borrow().get(key).cloned()) }

pub fn setBlock(id: i32, table: LootTable) { BLOCKS.with(|tables| tables.borrow_mut().insert(id, Rc::new(table))); }

pub fn setEntity(key: &str, table: LootTable)
{
    ENTITIES.with(|tables| tables.borrow_mut().insert(key.to_owned(), Rc::new(table)));
}

pub fn clear()
{
    BLOCKS.with(|tables| tables.borrow_mut().clear());
    ENTITIES.with(|tables| tables.borrow_mut().clear());
}

// the stacks a table rolls, the random is only touched for entries that pass their conditions
pub fn roll(table: &LootTable, random: &Random, context: &LootContext) -> Vec<Rc<ItemStack>>
{
    let mut stacks: Vec<Rc<ItemStack>> = Vec::new();

    for entry in &table.m_entries
    {
        if !context.allows(entry.m_player, entry.m_blockMeta) || random.nextFloat() >= entry.m_chance
        {
            continue;
        }

        let count: i32 = between(random, entry.m_min, entry.m_max);
        if count > 0
        {
            stacks.push(ItemStack::withDamage(entry.m_item, count, entry.m_meta));
        }
    }

    for pool in &table.m_pools
    {
        rollPool(pool, random, context, &mut stacks);
    }

    stacks
}

fn between(random: &Random, min: i32, max: i32) -> i32
{
    if max > min
    {
        return min + random.nextIntBounded(max - min + 1);
    }

    min
}

fn rollPool(pool: &LootPool, random: &Random, context: &LootContext, stacks: &mut Vec<Rc<ItemStack>>)
{
    // a mod condition may call into a mod, so each one runs once per pool
    let allowed: Vec<&PoolEntry> = pool
        .m_entries
        .iter()
        .filter(|entry| context.allows(entry.m_player, entry.m_blockMeta) && entry.m_rules.pass(context, random))
        .collect();
    let total: i32 = allowed.iter().map(|entry| entry.m_weight.max(0)).sum();
    if total <= 0
    {
        return;
    }

    for _ in 0..between(random, pool.m_min, pool.m_max)
    {
        let mut pick: i32 = random.nextIntBounded(total);
        let Some(entry) = allowed.iter().find(|entry|
        {
            pick -= entry.m_weight.max(0);
            pick < 0
        })
        else
        {
            continue;
        };

        let count: i32 = match entry.m_rules.m_count
        {
            Some(rule) => crate::hooks::lootCount(rule, context, random),
            None => between(random, entry.m_min, entry.m_max),
        };
        if let Some(item) = entry.m_item
            && count > 0
        {
            stacks.push(ItemStack::withDamage(item, count, entry.m_meta));
        }
    }
}
