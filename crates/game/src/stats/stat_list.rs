use crate::block::Block;
use crate::crafting::{CraftingManager, FurnaceRecipes};
use crate::item::Item;
use crate::locale::StatCollector;
use crate::stats::{AchievementList, IStatType, StatBase};
use jvm::array_list::JArrayList;
use jvm::hash_map::{JHashMap, JHashSet};
use jvm::throwable::OrNpe;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

type StatArray = Option<Vec<Option<Rc<StatBase>>>>;

thread_local! {
    static INIT: Cell<u8> = const { Cell::new(0) };
    static BY_ID: RefCell<JHashMap<i32, Rc<StatBase>>> = RefCell::new(JHashMap::new());
    static ALL: Rc<JArrayList<Rc<StatBase>>> = Rc::new(JArrayList::new());
    static GENERAL: Rc<JArrayList<Rc<StatBase>>> = Rc::new(JArrayList::new());
    static USED: Rc<JArrayList<Rc<StatBase>>> = Rc::new(JArrayList::new());
    static MINED: Rc<JArrayList<Rc<StatBase>>> = Rc::new(JArrayList::new());
    static NAMED: RefCell<Vec<Option<Rc<StatBase>>>> = RefCell::new(vec![None; NAMED_STATS.len()]);
    static MINE_BLOCK_STAT_ARRAY: RefCell<StatArray> = const { RefCell::new(None) };
    static ITEMS_CRAFTED: RefCell<StatArray> = const { RefCell::new(None) };
    static ITEMS_USED: RefCell<StatArray> = const { RefCell::new(None) };
    static ITEMS_BROKEN: RefCell<StatArray> = const { RefCell::new(None) };
    static HAS_BLOCK_STATS: Cell<bool> = const { Cell::new(false) };
    static HAS_ITEM_STATS: Cell<bool> = const { Cell::new(false) };
}

struct NamedStat
{
    m_id: i32,
    m_key: &'static str,
    m_formatter: &'static dyn IStatType,
    m_local: bool,
}

macro_rules! named_stats {
    ($($name:ident = $index:literal ($id:expr, $key:expr, $formatter:expr, $local:expr),)*) => {
        const NAMED_STATS: &[NamedStat] = &[$(NamedStat { m_id: $id, m_key: $key, m_formatter: $formatter, m_local: $local },)*];

        impl StatList
        {
            $(pub fn $name() -> Option<Rc<StatBase>> { StatList::named($index) })*
        }
    };
}

pub struct StatList;

named_stats! {
    startGameStat = 0 (1000, "stat.startGame", StatBase::NUMBER_FORMATTER, true),
    createWorldStat = 1 (1001, "stat.createWorld", StatBase::NUMBER_FORMATTER, true),
    loadWorldStat = 2 (1002, "stat.loadWorld", StatBase::NUMBER_FORMATTER, true),
    joinMultiplayerStat = 3 (1003, "stat.joinMultiplayer", StatBase::NUMBER_FORMATTER, true),
    leaveGameStat = 4 (1004, "stat.leaveGame", StatBase::NUMBER_FORMATTER, true),
    minutesPlayedStat = 5 (1100, "stat.playOneMinute", StatBase::TIME_FORMATTER, true),
    distanceWalkedStat = 6 (2000, "stat.walkOneCm", StatBase::DISTANCE_FORMATTER, true),
    distanceSwumStat = 7 (2001, "stat.swimOneCm", StatBase::DISTANCE_FORMATTER, true),
    distanceFallenStat = 8 (2002, "stat.fallOneCm", StatBase::DISTANCE_FORMATTER, true),
    distanceClimbedStat = 9 (2003, "stat.climbOneCm", StatBase::DISTANCE_FORMATTER, true),
    distanceFlownStat = 10 (2004, "stat.flyOneCm", StatBase::DISTANCE_FORMATTER, true),
    distanceDoveStat = 11 (2005, "stat.diveOneCm", StatBase::DISTANCE_FORMATTER, true),
    distanceByMinecartStat = 12 (2006, "stat.minecartOneCm", StatBase::DISTANCE_FORMATTER, true),
    distanceByBoatStat = 13 (2007, "stat.boatOneCm", StatBase::DISTANCE_FORMATTER, true),
    distanceByPigStat = 14 (2008, "stat.pigOneCm", StatBase::DISTANCE_FORMATTER, true),
    jumpStat = 15 (2010, "stat.jump", StatBase::NUMBER_FORMATTER, true),
    dropStat = 16 (2011, "stat.drop", StatBase::NUMBER_FORMATTER, true),
    damageDealtStat = 17 (2020, "stat.damageDealt", StatBase::NUMBER_FORMATTER, false),
    damageTakenStat = 18 (2021, "stat.damageTaken", StatBase::NUMBER_FORMATTER, false),
    deathsStat = 19 (2022, "stat.deaths", StatBase::NUMBER_FORMATTER, false),
    mobKillsStat = 20 (2023, "stat.mobKills", StatBase::NUMBER_FORMATTER, false),
    playerKillsStat = 21 (2024, "stat.playerKills", StatBase::NUMBER_FORMATTER, false),
    fishCaughtStat = 22 (2025, "stat.fishCaught", StatBase::NUMBER_FORMATTER, false),
}

fn ensureInit()
{
    if INIT.with(Cell::get) != 0
    {
        return;
    }
    INIT.with(|state| state.set(1));
    initStatList();
    INIT.with(|state| state.set(2));
}

fn initStatList()
{
    for (index, row) in NAMED_STATS.iter().enumerate()
    {
        let mut stat: Rc<StatBase> =
            StatBase::basic(row.m_id, StatCollector::translateToLocal(row.m_key), row.m_formatter);
        if row.m_local
        {
            stat = stat.setLocal();
        }
        let stat: Rc<StatBase> = stat.registerStat();
        NAMED.with(|named| named.borrow_mut()[index] = Some(stat));
    }
    let mineBlockStatArray: Vec<Option<Rc<StatBase>>> = initBlocksMinedStats("stat.mineBlock", 16777216);
    MINE_BLOCK_STAT_ARRAY.with(|array| *array.borrow_mut() = Some(mineBlockStatArray));
    AchievementList::init();
    HAS_BLOCK_STATS.with(|value| value.set(false));
    HAS_ITEM_STATS.with(|value| value.set(false));
}

fn element(array: &'static std::thread::LocalKey<RefCell<StatArray>>, index: i32) -> Option<Rc<StatBase>>
{
    ensureInit();
    array.with(|array| {
        let array: std::cell::Ref<StatArray> = array.borrow();
        let stats: &Vec<Option<Rc<StatBase>>> = array.as_ref().orNpe();
        if !(0..stats.len() as i32).contains(&index)
        {
            jvm::throwable::throw(&jvm::throwable::ARRAY_INDEX, &index.to_string());
        }
        stats[index as usize].clone()
    })
}

fn initBlocksMinedStats(name: &str, id: i32) -> Vec<Option<Rc<StatBase>>>
{
    let mut stats: Vec<Option<Rc<StatBase>>> = vec![None; 256];
    for i in 0..256
    {
        let Some(block) = Block::byId(i)
        else
        {
            continue;
        };
        if block.getEnableStats()
        {
            let s: String = StatCollector::translateToLocalFormatted(name, &[block.translateBlockName()]);
            let stat: Rc<StatBase> = StatBase::crafting(id + i, s, i).registerStat();
            MINED.with(|mined| mined.add(Rc::clone(&stat)));
            stats[i as usize] = Some(stat);
        }
    }

    replaceAllSimilarBlocks(&mut stats);
    stats
}

fn initItemsUsedStats(stats: StatArray, name: &str, id: i32, minId: i32, maxId: i32) -> StatArray
{
    let mut stats: Vec<Option<Rc<StatBase>>> = stats.unwrap_or_else(|| vec![None; 32000]);
    for i in minId..maxId
    {
        let Some(item) = Item::byId(i)
        else
        {
            continue;
        };
        let s: String = StatCollector::translateToLocalFormatted(name, &[item.getStatName()]);
        let stat: Rc<StatBase> = StatBase::crafting(id + i, s, i).registerStat();
        if i >= 256
        {
            USED.with(|used| used.add(Rc::clone(&stat)));
        }
        stats[i as usize] = Some(stat);
    }

    replaceAllSimilarBlocks(&mut stats);
    Some(stats)
}

fn initItemsBrokenStats(stats: StatArray, name: &str, id: i32, minId: i32, maxId: i32) -> StatArray
{
    let mut stats: Vec<Option<Rc<StatBase>>> = stats.unwrap_or_else(|| vec![None; 32000]);
    for i in minId..maxId
    {
        let Some(item) = Item::byId(i)
        else
        {
            continue;
        };
        if item.isDamagable()
        {
            let s: String = StatCollector::translateToLocalFormatted(name, &[item.getStatName()]);
            stats[i as usize] = Some(StatBase::crafting(id + i, s, i).registerStat());
        }
    }

    replaceAllSimilarBlocks(&mut stats);
    Some(stats)
}

fn initItemsCraftedStats()
{
    if !(HAS_BLOCK_STATS.with(Cell::get) && HAS_ITEM_STATS.with(Cell::get))
    {
        return;
    }

    let mut hashset: JHashSet<i32> = JHashSet::new();
    for irecipe in CraftingManager::getInstance().getRecipeList().toVec()
    {
        hashset.add(irecipe.getRecipeOutput().m_itemID.get());
    }
    for itemstack in FurnaceRecipes::smelting().getSmeltingList().borrow().values()
    {
        hashset.add(itemstack.m_itemID.get());
    }

    let mut stats: Vec<Option<Rc<StatBase>>> = vec![None; 32000];
    for integer in hashset.items()
    {
        let Some(item) = Item::byId(integer)
        else
        {
            continue;
        };
        let s: String = StatCollector::translateToLocalFormatted("stat.craftItem", &[item.getStatName()]);
        stats[integer as usize] = Some(StatBase::crafting(16842752 + integer, s, integer).registerStat());
    }

    replaceAllSimilarBlocks(&mut stats);
    ITEMS_CRAFTED.with(|array| *array.borrow_mut() = Some(stats));
}

fn replaceAllSimilarBlocks(stats: &mut [Option<Rc<StatBase>>])
{
    replaceSimilarBlocks(stats, Block::waterStill().m_blockID, Block::waterMoving().m_blockID);
    replaceSimilarBlocks(stats, Block::lavaStill().m_blockID, Block::lavaStill().m_blockID);
    replaceSimilarBlocks(stats, Block::pumpkinLantern().m_blockID, Block::pumpkin().m_blockID);
    replaceSimilarBlocks(stats, Block::stoneOvenActive().m_blockID, Block::stoneOvenIdle().m_blockID);
    replaceSimilarBlocks(stats, Block::oreRedstoneGlowing().m_blockID, Block::oreRedstone().m_blockID);
    replaceSimilarBlocks(stats, Block::redstoneRepeaterActive().m_blockID, Block::redstoneRepeaterIdle().m_blockID);
    replaceSimilarBlocks(stats, Block::torchRedstoneActive().m_blockID, Block::torchRedstoneIdle().m_blockID);
    replaceSimilarBlocks(stats, Block::mushroomRed().m_blockID, Block::mushroomBrown().m_blockID);
    replaceSimilarBlocks(stats, Block::stairDouble().m_blockID, Block::stairSingle().m_blockID);
    replaceSimilarBlocks(stats, Block::grass().m_blockID, Block::dirt().m_blockID);
    replaceSimilarBlocks(stats, Block::tilledField().m_blockID, Block::dirt().m_blockID);
}

fn replaceSimilarBlocks(stats: &mut [Option<Rc<StatBase>>], block1: i32, block2: i32)
{
    let block1: usize = block1 as usize;
    let block2: usize = block2 as usize;
    if stats[block1].is_some() && stats[block2].is_none()
    {
        stats[block2] = stats[block1].clone();
        return;
    }

    if let Some(stat) = &stats[block1]
    {
        ALL.with(|all| all.removeIdentity(stat));
        MINED.with(|mined| mined.removeIdentity(stat));
        GENERAL.with(|general| general.removeIdentity(stat));
    }
    stats[block1] = stats[block2].clone();
}

impl StatList
{
    fn named(index: usize) -> Option<Rc<StatBase>>
    {
        ensureInit();
        NAMED.with(|named| named.borrow()[index].clone())
    }

    pub fn all() -> Rc<JArrayList<Rc<StatBase>>>
    {
        ensureInit();
        ALL.with(Rc::clone)
    }

    pub fn general() -> Rc<JArrayList<Rc<StatBase>>>
    {
        ensureInit();
        GENERAL.with(Rc::clone)
    }

    pub fn used() -> Rc<JArrayList<Rc<StatBase>>>
    {
        ensureInit();
        USED.with(Rc::clone)
    }

    pub fn mined() -> Rc<JArrayList<Rc<StatBase>>>
    {
        ensureInit();
        MINED.with(Rc::clone)
    }

    pub(super) fn putById(stat: &Rc<StatBase>)
    {
        ensureInit();
        BY_ID.with(|byId| byId.borrow_mut().put(stat.m_statId, Rc::clone(stat)));
    }

    pub fn init() { ensureInit(); }

    pub fn initBlockStats()
    {
        ensureInit();
        let used: StatArray = ITEMS_USED.with(RefCell::take);
        let used: StatArray = initItemsUsedStats(used, "stat.useItem", 16908288, 0, 256);
        ITEMS_USED.with(|array| *array.borrow_mut() = used);
        let broken: StatArray = ITEMS_BROKEN.with(RefCell::take);
        let broken: StatArray = initItemsBrokenStats(broken, "stat.breakItem", 16973824, 0, 256);
        ITEMS_BROKEN.with(|array| *array.borrow_mut() = broken);
        HAS_BLOCK_STATS.with(|value| value.set(true));
        initItemsCraftedStats();
    }

    pub fn initItemStats()
    {
        ensureInit();
        let used: StatArray = ITEMS_USED.with(RefCell::take);
        let used: StatArray = initItemsUsedStats(used, "stat.useItem", 16908288, 256, 32000);
        ITEMS_USED.with(|array| *array.borrow_mut() = used);
        let broken: StatArray = ITEMS_BROKEN.with(RefCell::take);
        let broken: StatArray = initItemsBrokenStats(broken, "stat.breakItem", 16973824, 256, 32000);
        ITEMS_BROKEN.with(|array| *array.borrow_mut() = broken);
        HAS_ITEM_STATS.with(|value| value.set(true));
        initItemsCraftedStats();
    }

    pub fn mineBlockStat(block: i32) -> Option<Rc<StatBase>> { element(&MINE_BLOCK_STAT_ARRAY, block) }

    pub fn itemsCrafted(item: i32) -> Option<Rc<StatBase>> { element(&ITEMS_CRAFTED, item) }

    pub fn itemsUsed(item: i32) -> Option<Rc<StatBase>> { element(&ITEMS_USED, item) }

    pub fn itemsBroken(item: i32) -> Option<Rc<StatBase>> { element(&ITEMS_BROKEN, item) }

    pub fn byKey(id: i32) -> Option<Rc<StatBase>>
    {
        ensureInit();
        BY_ID.with(|byId| byId.borrow().get(&id))
    }
}
