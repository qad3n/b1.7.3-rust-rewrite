use crate::item::ItemStack;
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

// a block or entity without a table drops exactly as beta, a table adds its entries or replaces the beta drops
#[derive(Clone, PartialEq, Debug, Default)]
pub struct LootTable
{
    pub m_keepBeta: bool,
    pub m_entries: Vec<LootEntry>,
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
pub fn roll(table: &LootTable, random: &Random, byPlayer: bool, blockMeta: i32) -> Vec<Rc<ItemStack>>
{
    let mut stacks: Vec<Rc<ItemStack>> = Vec::new();

    for entry in &table.m_entries
    {
        let allowed: bool = (!entry.m_player || byPlayer) && entry.m_blockMeta.is_none_or(|meta| meta == blockMeta);
        if !allowed || random.nextFloat() >= entry.m_chance
        {
            continue;
        }

        let count: i32 = if entry.m_max > entry.m_min
        {
            entry.m_min + random.nextIntBounded(entry.m_max - entry.m_min + 1)
        }
        else
        {
            entry.m_min
        };

        if count > 0
        {
            stacks.push(ItemStack::withDamage(entry.m_item, count, entry.m_meta));
        }
    }

    stacks
}
