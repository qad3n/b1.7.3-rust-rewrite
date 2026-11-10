use crate::block::{BLOCK_IDS, Block};
use std::sync::{LazyLock, PoisonError, RwLock, RwLockReadGuard};

// item ids sit past the 256 block ids
const PORK_RAW: i32 = 256 + 63;
const PORK_COOKED: i32 = 256 + 64;
const ITEM_IDS: usize = 32000;
const WORD_BITS: usize = 64;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Registry
{
    Block,
    Item,
}

// every set of ids the engine asks about, a row in ROWS holds its name and beta members
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tag
{
    Climbable,
    Leaves,
    Logs,
    Infiniburn,
    FlowerSoil,
    DeadBushSoil,
    CropSoil,
    ReedSoil,
    AnimalsSpawnableOn,
    Fences,
    MineablePickaxe,
    MineableAxe,
    MineableShovel,
    NeedsStoneTool,
    NeedsIronTool,
    NeedsDiamondTool,
    WolfFood,
}

pub struct TagRow
{
    pub m_tag: Tag,
    pub m_name: &'static str,
    pub m_registry: Registry,
    pub m_seeds: &'static [i32],
}

// rows follow Tag order so a tag indexes its row
pub const ROWS: [TagRow; 17] = [
    TagRow { m_tag: Tag::Climbable, m_name: "climbable", m_registry: Registry::Block, m_seeds: &[Block::LADDER] },
    TagRow { m_tag: Tag::Leaves, m_name: "leaves", m_registry: Registry::Block, m_seeds: &[Block::LEAVES] },
    TagRow { m_tag: Tag::Logs, m_name: "logs", m_registry: Registry::Block, m_seeds: &[Block::WOOD] },
    TagRow { m_tag: Tag::Infiniburn, m_name: "infiniburn", m_registry: Registry::Block, m_seeds: &[Block::NETHERRACK] },
    TagRow {
        m_tag: Tag::FlowerSoil,
        m_name: "flower_soil",
        m_registry: Registry::Block,
        m_seeds: &[Block::GRASS, Block::DIRT, Block::TILLED_FIELD],
    },
    TagRow { m_tag: Tag::DeadBushSoil, m_name: "dead_bush_soil", m_registry: Registry::Block, m_seeds: &[Block::SAND] },
    TagRow { m_tag: Tag::CropSoil, m_name: "crop_soil", m_registry: Registry::Block, m_seeds: &[Block::TILLED_FIELD] },
    TagRow {
        m_tag: Tag::ReedSoil,
        m_name: "reed_soil",
        m_registry: Registry::Block,
        m_seeds: &[Block::GRASS, Block::DIRT],
    },
    TagRow {
        m_tag: Tag::AnimalsSpawnableOn,
        m_name: "animals_spawnable_on",
        m_registry: Registry::Block,
        m_seeds: &[Block::GRASS],
    },
    TagRow { m_tag: Tag::Fences, m_name: "fences", m_registry: Registry::Block, m_seeds: &[Block::FENCE] },
    TagRow {
        m_tag: Tag::MineablePickaxe,
        m_name: "mineable/pickaxe",
        m_registry: Registry::Block,
        m_seeds: &[
            Block::COBBLESTONE,
            Block::STAIR_DOUBLE,
            Block::STAIR_SINGLE,
            Block::STONE,
            Block::SAND_STONE,
            Block::COBBLESTONE_MOSSY,
            Block::ORE_IRON,
            Block::BLOCK_STEEL,
            Block::ORE_COAL,
            Block::BLOCK_GOLD,
            Block::ORE_GOLD,
            Block::ORE_DIAMOND,
            Block::BLOCK_DIAMOND,
            Block::ICE,
            Block::NETHERRACK,
            Block::ORE_LAPIS,
            Block::BLOCK_LAPIS,
        ],
    },
    TagRow {
        m_tag: Tag::MineableAxe,
        m_name: "mineable/axe",
        m_registry: Registry::Block,
        m_seeds: &[Block::PLANKS, Block::BOOK_SHELF, Block::WOOD, Block::CHEST],
    },
    TagRow {
        m_tag: Tag::MineableShovel,
        m_name: "mineable/shovel",
        m_registry: Registry::Block,
        m_seeds: &[
            Block::GRASS,
            Block::DIRT,
            Block::SAND,
            Block::GRAVEL,
            Block::SNOW,
            Block::BLOCK_SNOW,
            Block::BLOCK_CLAY,
            Block::TILLED_FIELD,
        ],
    },
    TagRow {
        m_tag: Tag::NeedsStoneTool,
        m_name: "needs_stone_tool",
        m_registry: Registry::Block,
        m_seeds: &[Block::BLOCK_STEEL, Block::ORE_IRON, Block::BLOCK_LAPIS, Block::ORE_LAPIS],
    },
    TagRow {
        m_tag: Tag::NeedsIronTool,
        m_name: "needs_iron_tool",
        m_registry: Registry::Block,
        m_seeds: &[
            Block::BLOCK_DIAMOND,
            Block::ORE_DIAMOND,
            Block::BLOCK_GOLD,
            Block::ORE_GOLD,
            Block::ORE_REDSTONE,
            Block::ORE_REDSTONE_GLOWING,
        ],
    },
    TagRow {
        m_tag: Tag::NeedsDiamondTool,
        m_name: "needs_diamond_tool",
        m_registry: Registry::Block,
        m_seeds: &[Block::OBSIDIAN],
    },
    TagRow { m_tag: Tag::WolfFood, m_name: "wolf_food", m_registry: Registry::Item, m_seeds: &[PORK_RAW, PORK_COOKED] },
];

// shared with chunk generation threads, so one lock instead of a thread local
static SETS: LazyLock<RwLock<Vec<Vec<u64>>>> = LazyLock::new(|| RwLock::new(ROWS.iter().map(seeded).collect()));

pub fn row(tag: Tag) -> &'static TagRow { &ROWS[tag as usize] }

pub fn byName(name: &str) -> Option<Tag> { ROWS.iter().find(|row| row.m_name == name).map(|row| row.m_tag) }

pub fn has(tag: Tag, id: i32) -> bool
{
    let Some(index) = slot(row(tag).m_registry, id)
    else
    {
        return false;
    };

    let sets: RwLockReadGuard<'_, Vec<Vec<u64>>> = SETS.read().unwrap_or_else(PoisonError::into_inner);
    sets[tag as usize][index / WORD_BITS] & (1 << (index % WORD_BITS)) != 0
}

// the loader's resolved members replace the beta seeds of one tag
pub fn set(tag: Tag, ids: &[i32])
{
    let bits: Vec<u64> = build(row(tag).m_registry, ids);
    SETS.write().unwrap_or_else(PoisonError::into_inner)[tag as usize] = bits;
}

pub fn reset()
{
    let seeds: Vec<Vec<u64>> = ROWS.iter().map(seeded).collect();
    *SETS.write().unwrap_or_else(PoisonError::into_inner) = seeds;
}

fn seeded(row: &TagRow) -> Vec<u64> { build(row.m_registry, row.m_seeds) }

fn build(registry: Registry, ids: &[i32]) -> Vec<u64>
{
    let mut bits: Vec<u64> = vec![0; limit(registry).div_ceil(WORD_BITS)];
    for index in ids.iter().filter_map(|id| slot(registry, *id))
    {
        bits[index / WORD_BITS] |= 1 << (index % WORD_BITS);
    }
    bits
}

fn slot(registry: Registry, id: i32) -> Option<usize>
{
    let index: usize = usize::try_from(id).ok()?;
    (index < limit(registry)).then_some(index)
}

fn limit(registry: Registry) -> usize
{
    match registry
    {
        Registry::Block => BLOCK_IDS,
        Registry::Item => ITEM_IDS,
    }
}
