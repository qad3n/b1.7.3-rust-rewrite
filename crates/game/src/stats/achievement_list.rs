use crate::block::Block;
use crate::item::{Item, ItemStack};
use crate::stats::{Achievement, StatBase};
use jvm::array_list::JArrayList;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

thread_local! {
    static INIT: Cell<u8> = const { Cell::new(0) };
    static MIN_DISPLAY_COLUMN: Cell<i32> = const { Cell::new(0) };
    static MIN_DISPLAY_ROW: Cell<i32> = const { Cell::new(0) };
    static MAX_DISPLAY_COLUMN: Cell<i32> = const { Cell::new(0) };
    static MAX_DISPLAY_ROW: Cell<i32> = const { Cell::new(0) };
    static ACHIEVEMENT_LIST: RefCell<Option<Rc<JArrayList<Rc<StatBase>>>>> = const { RefCell::new(None) };
    static NAMED: RefCell<Vec<Option<Rc<StatBase>>>> = RefCell::new(vec![None; 16]);
}

pub struct AchievementList;

macro_rules! named_achievements {
    ($($name:ident = $id:expr,)*) => {
        impl AchievementList
        {
            $(pub fn $name() -> Option<Rc<StatBase>> { AchievementList::named($id) })*
        }
    };
}

named_achievements! {
    openInventory = 0,
    mineWood = 1,
    buildWorkBench = 2,
    buildPickaxe = 3,
    buildFurnace = 4,
    acquireIron = 5,
    buildHoe = 6,
    makeBread = 7,
    bakeCake = 8,
    buildBetterPickaxe = 9,
    cookFish = 10,
    onARail = 11,
    buildSword = 12,
    killEnemy = 13,
    killCow = 14,
    flyPig = 15,
}

fn ensureInit()
{
    if INIT.with(Cell::get) != 0
    {
        return;
    }
    INIT.with(|state| state.set(1));
    initAchievements();
    INIT.with(|state| state.set(2));
}

fn store(id: i32, achievement: Rc<StatBase>)
{
    NAMED.with(|named| named.borrow_mut()[id as usize] = Some(achievement));
}

fn initAchievements()
{
    ACHIEVEMENT_LIST.with(|list| *list.borrow_mut() = Some(Rc::new(JArrayList::new())));
    store(
        0,
        Achievement::create(0, "openInventory", 0, 0, ItemStack::withItem(&Item::book()), None)
            .setLocal()
            .registerStat(),
    );
    store(
        1,
        Achievement::create(
            1,
            "mineWood",
            2,
            1,
            ItemStack::withBlock(&Block::wood()),
            AchievementList::openInventory(),
        )
        .registerStat(),
    );
    store(
        2,
        Achievement::create(
            2,
            "buildWorkBench",
            4,
            -1,
            ItemStack::withBlock(&Block::workbench()),
            AchievementList::mineWood(),
        )
        .registerStat(),
    );
    store(
        3,
        Achievement::create(
            3,
            "buildPickaxe",
            4,
            2,
            ItemStack::withItem(&Item::pickaxeWood()),
            AchievementList::buildWorkBench(),
        )
        .registerStat(),
    );
    store(
        4,
        Achievement::create(
            4,
            "buildFurnace",
            3,
            4,
            ItemStack::withBlock(&Block::stoneOvenActive()),
            AchievementList::buildPickaxe(),
        )
        .registerStat(),
    );
    store(
        5,
        Achievement::create(
            5,
            "acquireIron",
            1,
            4,
            ItemStack::withItem(&Item::ingotIron()),
            AchievementList::buildFurnace(),
        )
        .registerStat(),
    );
    store(
        6,
        Achievement::create(
            6,
            "buildHoe",
            2,
            -3,
            ItemStack::withItem(&Item::hoeWood()),
            AchievementList::buildWorkBench(),
        )
        .registerStat(),
    );
    store(
        7,
        Achievement::create(7, "makeBread", -1, -3, ItemStack::withItem(&Item::bread()), AchievementList::buildHoe())
            .registerStat(),
    );
    store(
        8,
        Achievement::create(8, "bakeCake", 0, -5, ItemStack::withItem(&Item::cake()), AchievementList::buildHoe())
            .registerStat(),
    );
    store(
        9,
        Achievement::create(
            9,
            "buildBetterPickaxe",
            6,
            2,
            ItemStack::withItem(&Item::pickaxeStone()),
            AchievementList::buildPickaxe(),
        )
        .registerStat(),
    );
    store(
        10,
        Achievement::create(
            10,
            "cookFish",
            2,
            6,
            ItemStack::withItem(&Item::fishCooked()),
            AchievementList::buildFurnace(),
        )
        .registerStat(),
    );
    store(
        11,
        Achievement::setSpecial(&Achievement::create(
            11,
            "onARail",
            2,
            3,
            ItemStack::withBlock(&Block::rail()),
            AchievementList::acquireIron(),
        ))
        .registerStat(),
    );
    store(
        12,
        Achievement::create(
            12,
            "buildSword",
            6,
            -1,
            ItemStack::withItem(&Item::swordWood()),
            AchievementList::buildWorkBench(),
        )
        .registerStat(),
    );
    store(
        13,
        Achievement::create(13, "killEnemy", 8, -1, ItemStack::withItem(&Item::bone()), AchievementList::buildSword())
            .registerStat(),
    );
    store(
        14,
        Achievement::create(14, "killCow", 7, -3, ItemStack::withItem(&Item::leather()), AchievementList::buildSword())
            .registerStat(),
    );
    store(
        15,
        Achievement::setSpecial(&Achievement::create(
            15,
            "flyPig",
            8,
            -4,
            ItemStack::withItem(&Item::saddle()),
            AchievementList::killCow(),
        ))
        .registerStat(),
    );
    jvm::system::println(&format!("{} achievements", AchievementList::achievementList().size()));
}

impl AchievementList
{
    pub fn init() { ensureInit(); }

    fn named(id: i32) -> Option<Rc<StatBase>>
    {
        ensureInit();
        NAMED.with(|named| named.borrow()[id as usize].clone())
    }

    pub fn achievementList() -> Rc<JArrayList<Rc<StatBase>>>
    {
        ensureInit();
        ACHIEVEMENT_LIST.with(|list| list.borrow().clone()).unwrap_or_else(|| jvm::throwable::npe())
    }

    pub fn minDisplayColumn() -> i32
    {
        ensureInit();
        MIN_DISPLAY_COLUMN.with(Cell::get)
    }

    pub fn minDisplayRow() -> i32
    {
        ensureInit();
        MIN_DISPLAY_ROW.with(Cell::get)
    }

    pub fn maxDisplayColumn() -> i32
    {
        ensureInit();
        MAX_DISPLAY_COLUMN.with(Cell::get)
    }

    pub fn maxDisplayRow() -> i32
    {
        ensureInit();
        MAX_DISPLAY_ROW.with(Cell::get)
    }

    pub(super) fn updateDisplayBounds(column: i32, row: i32)
    {
        ensureInit();
        if column < MIN_DISPLAY_COLUMN.with(Cell::get)
        {
            MIN_DISPLAY_COLUMN.with(|value| value.set(column));
        }
        if row < MIN_DISPLAY_ROW.with(Cell::get)
        {
            MIN_DISPLAY_ROW.with(|value| value.set(row));
        }
        if column > MAX_DISPLAY_COLUMN.with(Cell::get)
        {
            MAX_DISPLAY_COLUMN.with(|value| value.set(column));
        }
        if row > MAX_DISPLAY_ROW.with(Cell::get)
        {
            MAX_DISPLAY_ROW.with(|value| value.set(row));
        }
    }
}
