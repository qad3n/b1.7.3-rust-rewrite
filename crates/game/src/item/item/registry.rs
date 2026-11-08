use crate::block::{Block, Material};
use crate::item::item::EnumToolMaterial;
use crate::item::{ITEM, Item};
use jvm::random::Random;
use jvm::throwable::OrNpe;
use std::cell::{Cell, OnceCell, RefCell};
use std::rc::Rc;

thread_local! {
    static INIT: Cell<u8> = const { Cell::new(0) };
    static ITEM_RAND: OnceCell<Rc<Random>> = const { OnceCell::new() };
    static ITEMS_LIST: RefCell<Vec<Option<Rc<Item>>>> = RefCell::new(vec![None; 32000]);
}

pub fn ensureInit()
{
    if INIT.with(Cell::get) != 0
    {
        return;
    }
    INIT.with(|state| state.set(1));
    initItems();
    INIT.with(|state| state.set(2));
}

fn index(id: i32) -> usize
{
    if !(0..32000).contains(&id)
    {
        jvm::throwable::throw(&jvm::throwable::ARRAY_INDEX, &id.to_string());
    }
    id as usize
}

pub(super) fn register(id: i32, item: &Rc<Item>)
{
    ITEMS_LIST.with(|list| list.borrow_mut()[index(id)] = Some(Rc::clone(item)));
}

macro_rules! named_items {
    ($($name:ident = $id:expr,)*) => {
        impl Item
        {
            $(pub fn $name() -> Rc<Item> { Item::byId(256 + $id).orNpe() })*
        }
    };
}

named_items! {
    shovelSteel = 0,
    pickaxeSteel = 1,
    axeSteel = 2,
    flintAndSteel = 3,
    appleRed = 4,
    bow = 5,
    arrow = 6,
    coal = 7,
    diamond = 8,
    ingotIron = 9,
    ingotGold = 10,
    swordSteel = 11,
    swordWood = 12,
    shovelWood = 13,
    pickaxeWood = 14,
    axeWood = 15,
    swordStone = 16,
    shovelStone = 17,
    pickaxeStone = 18,
    axeStone = 19,
    swordDiamond = 20,
    shovelDiamond = 21,
    pickaxeDiamond = 22,
    axeDiamond = 23,
    stick = 24,
    bowlEmpty = 25,
    bowlSoup = 26,
    swordGold = 27,
    shovelGold = 28,
    pickaxeGold = 29,
    axeGold = 30,
    silk = 31,
    feather = 32,
    gunpowder = 33,
    hoeWood = 34,
    hoeStone = 35,
    hoeSteel = 36,
    hoeDiamond = 37,
    hoeGold = 38,
    seeds = 39,
    wheat = 40,
    bread = 41,
    helmetLeather = 42,
    plateLeather = 43,
    legsLeather = 44,
    bootsLeather = 45,
    helmetChain = 46,
    plateChain = 47,
    legsChain = 48,
    bootsChain = 49,
    helmetSteel = 50,
    plateSteel = 51,
    legsSteel = 52,
    bootsSteel = 53,
    helmetDiamond = 54,
    plateDiamond = 55,
    legsDiamond = 56,
    bootsDiamond = 57,
    helmetGold = 58,
    plateGold = 59,
    legsGold = 60,
    bootsGold = 61,
    flint = 62,
    porkRaw = 63,
    porkCooked = 64,
    painting = 65,
    appleGold = 66,
    sign = 67,
    doorWood = 68,
    bucketEmpty = 69,
    bucketWater = 70,
    bucketLava = 71,
    minecartEmpty = 72,
    saddle = 73,
    doorSteel = 74,
    redstone = 75,
    snowball = 76,
    boat = 77,
    leather = 78,
    bucketMilk = 79,
    brick = 80,
    clay = 81,
    reed = 82,
    paper = 83,
    book = 84,
    slimeBall = 85,
    minecartCrate = 86,
    minecartPowered = 87,
    egg = 88,
    compass = 89,
    fishingRod = 90,
    pocketSundial = 91,
    lightStoneDust = 92,
    fishRaw = 93,
    fishCooked = 94,
    dyePowder = 95,
    bone = 96,
    sugar = 97,
    cake = 98,
    bed = 99,
    redstoneRepeater = 100,
    cookie = 101,
    mapItem = 102,
    shears = 103,
    record13 = 2000,
    recordCat = 2001,
}

impl Item
{
    pub fn basic(id: i32) -> Rc<Item> { Item::construct(&ITEM, Box::new(()), id) }

    pub fn byId(id: i32) -> Option<Rc<Item>>
    {
        ensureInit();
        ITEMS_LIST.with(|list| list.borrow()[index(id)].clone())
    }

    pub fn byIdOrNpe(id: i32) -> Rc<Item> { Item::byId(id).orNpe() }

    pub fn itemRand() -> Rc<Random>
    {
        ensureInit();
        ITEM_RAND.with(|random| random.get().cloned()).orNpe()
    }

    pub fn registerBlockItems()
    {
        let set = |id: i32, item: Rc<Item>| register(id, &item);
        ensureInit();
        set(
            Block::cloth().m_blockID,
            crate::item::item_cloth::create(Block::cloth().m_blockID - 256).setItemName("cloth"),
        );
        set(Block::wood().m_blockID, crate::item::item_log::create(Block::wood().m_blockID - 256).setItemName("log"));
        set(
            Block::stairSingle().m_blockID,
            crate::item::item_slab::create(Block::stairSingle().m_blockID - 256).setItemName("stoneSlab"),
        );
        set(
            Block::sapling().m_blockID,
            crate::item::item_sapling::create(Block::sapling().m_blockID - 256).setItemName("sapling"),
        );
        set(
            Block::leaves().m_blockID,
            crate::item::item_leaves::create(Block::leaves().m_blockID - 256).setItemName("leaves"),
        );
        set(Block::pistonBase().m_blockID, crate::item::item_piston::create(Block::pistonBase().m_blockID - 256));
        set(
            Block::pistonStickyBase().m_blockID,
            crate::item::item_piston::create(Block::pistonStickyBase().m_blockID - 256),
        );
        for i in 0..256
        {
            let Some(block) = Block::byId(i)
            else
            {
                continue;
            };
            if Item::byId(i).is_some()
            {
                continue;
            }
            set(i, crate::item::item_block::create(i - 256));
            block.initializeBlock();
        }
    }
}

fn initItems()
{
    ITEM_RAND.with(|random| random.set(Rc::new(Random::new())).ok());
    crate::item::item_spade::create(0, EnumToolMaterial::Iron).setIconCoord(2, 5).setItemName("shovelIron");
    crate::item::item_pickaxe::create(1, EnumToolMaterial::Iron).setIconCoord(2, 6).setItemName("pickaxeIron");
    crate::item::item_axe::create(2, EnumToolMaterial::Iron).setIconCoord(2, 7).setItemName("hatchetIron");
    crate::item::item_flint_and_steel::create(3).setIconCoord(5, 0).setItemName("flintAndSteel");
    crate::item::item_food::create(4, 4, false).setIconCoord(10, 0).setItemName("apple");
    crate::item::item_bow::create(5).setIconCoord(5, 1).setItemName("bow");
    Item::basic(6).setIconCoord(5, 2).setItemName("arrow");
    crate::item::item_coal::create(7).setIconCoord(7, 0).setItemName("coal");
    Item::basic(8).setIconCoord(7, 3).setItemName("emerald");
    Item::basic(9).setIconCoord(7, 1).setItemName("ingotIron");
    Item::basic(10).setIconCoord(7, 2).setItemName("ingotGold");
    crate::item::item_sword::create(11, EnumToolMaterial::Iron).setIconCoord(2, 4).setItemName("swordIron");
    crate::item::item_sword::create(12, EnumToolMaterial::Wood).setIconCoord(0, 4).setItemName("swordWood");
    crate::item::item_spade::create(13, EnumToolMaterial::Wood).setIconCoord(0, 5).setItemName("shovelWood");
    crate::item::item_pickaxe::create(14, EnumToolMaterial::Wood).setIconCoord(0, 6).setItemName("pickaxeWood");
    crate::item::item_axe::create(15, EnumToolMaterial::Wood).setIconCoord(0, 7).setItemName("hatchetWood");
    crate::item::item_sword::create(16, EnumToolMaterial::Stone).setIconCoord(1, 4).setItemName("swordStone");
    crate::item::item_spade::create(17, EnumToolMaterial::Stone).setIconCoord(1, 5).setItemName("shovelStone");
    crate::item::item_pickaxe::create(18, EnumToolMaterial::Stone).setIconCoord(1, 6).setItemName("pickaxeStone");
    crate::item::item_axe::create(19, EnumToolMaterial::Stone).setIconCoord(1, 7).setItemName("hatchetStone");
    crate::item::item_sword::create(20, EnumToolMaterial::Emerald).setIconCoord(3, 4).setItemName("swordDiamond");
    crate::item::item_spade::create(21, EnumToolMaterial::Emerald).setIconCoord(3, 5).setItemName("shovelDiamond");
    crate::item::item_pickaxe::create(22, EnumToolMaterial::Emerald).setIconCoord(3, 6).setItemName("pickaxeDiamond");
    crate::item::item_axe::create(23, EnumToolMaterial::Emerald).setIconCoord(3, 7).setItemName("hatchetDiamond");
    Item::basic(24).setIconCoord(5, 3).setFull3D().setItemName("stick");
    Item::basic(25).setIconCoord(7, 4).setItemName("bowl");
    crate::item::item_soup::create(26, 10).setIconCoord(8, 4).setItemName("mushroomStew");
    crate::item::item_sword::create(27, EnumToolMaterial::Gold).setIconCoord(4, 4).setItemName("swordGold");
    crate::item::item_spade::create(28, EnumToolMaterial::Gold).setIconCoord(4, 5).setItemName("shovelGold");
    crate::item::item_pickaxe::create(29, EnumToolMaterial::Gold).setIconCoord(4, 6).setItemName("pickaxeGold");
    crate::item::item_axe::create(30, EnumToolMaterial::Gold).setIconCoord(4, 7).setItemName("hatchetGold");
    Item::basic(31).setIconCoord(8, 0).setItemName("string");
    Item::basic(32).setIconCoord(8, 1).setItemName("feather");
    Item::basic(33).setIconCoord(8, 2).setItemName("sulphur");
    crate::item::item_hoe::create(34, EnumToolMaterial::Wood).setIconCoord(0, 8).setItemName("hoeWood");
    crate::item::item_hoe::create(35, EnumToolMaterial::Stone).setIconCoord(1, 8).setItemName("hoeStone");
    crate::item::item_hoe::create(36, EnumToolMaterial::Iron).setIconCoord(2, 8).setItemName("hoeIron");
    crate::item::item_hoe::create(37, EnumToolMaterial::Emerald).setIconCoord(3, 8).setItemName("hoeDiamond");
    crate::item::item_hoe::create(38, EnumToolMaterial::Gold).setIconCoord(4, 8).setItemName("hoeGold");
    crate::item::item_seeds::create(39, Block::crops().m_blockID).setIconCoord(9, 0).setItemName("seeds");
    Item::basic(40).setIconCoord(9, 1).setItemName("wheat");
    crate::item::item_food::create(41, 5, false).setIconCoord(9, 2).setItemName("bread");
    crate::item::item_armor::create(42, 0, 0, 0).setIconCoord(0, 0).setItemName("helmetCloth");
    crate::item::item_armor::create(43, 0, 0, 1).setIconCoord(0, 1).setItemName("chestplateCloth");
    crate::item::item_armor::create(44, 0, 0, 2).setIconCoord(0, 2).setItemName("leggingsCloth");
    crate::item::item_armor::create(45, 0, 0, 3).setIconCoord(0, 3).setItemName("bootsCloth");
    crate::item::item_armor::create(46, 1, 1, 0).setIconCoord(1, 0).setItemName("helmetChain");
    crate::item::item_armor::create(47, 1, 1, 1).setIconCoord(1, 1).setItemName("chestplateChain");
    crate::item::item_armor::create(48, 1, 1, 2).setIconCoord(1, 2).setItemName("leggingsChain");
    crate::item::item_armor::create(49, 1, 1, 3).setIconCoord(1, 3).setItemName("bootsChain");
    crate::item::item_armor::create(50, 2, 2, 0).setIconCoord(2, 0).setItemName("helmetIron");
    crate::item::item_armor::create(51, 2, 2, 1).setIconCoord(2, 1).setItemName("chestplateIron");
    crate::item::item_armor::create(52, 2, 2, 2).setIconCoord(2, 2).setItemName("leggingsIron");
    crate::item::item_armor::create(53, 2, 2, 3).setIconCoord(2, 3).setItemName("bootsIron");
    crate::item::item_armor::create(54, 3, 3, 0).setIconCoord(3, 0).setItemName("helmetDiamond");
    crate::item::item_armor::create(55, 3, 3, 1).setIconCoord(3, 1).setItemName("chestplateDiamond");
    crate::item::item_armor::create(56, 3, 3, 2).setIconCoord(3, 2).setItemName("leggingsDiamond");
    crate::item::item_armor::create(57, 3, 3, 3).setIconCoord(3, 3).setItemName("bootsDiamond");
    crate::item::item_armor::create(58, 1, 4, 0).setIconCoord(4, 0).setItemName("helmetGold");
    crate::item::item_armor::create(59, 1, 4, 1).setIconCoord(4, 1).setItemName("chestplateGold");
    crate::item::item_armor::create(60, 1, 4, 2).setIconCoord(4, 2).setItemName("leggingsGold");
    crate::item::item_armor::create(61, 1, 4, 3).setIconCoord(4, 3).setItemName("bootsGold");
    Item::basic(62).setIconCoord(6, 0).setItemName("flint");
    crate::item::item_food::create(63, 3, true).setIconCoord(7, 5).setItemName("porkchopRaw");
    crate::item::item_food::create(64, 8, true).setIconCoord(8, 5).setItemName("porkchopCooked");
    crate::item::item_painting::create(65).setIconCoord(10, 1).setItemName("painting");
    crate::item::item_food::create(66, 42, false).setIconCoord(11, 0).setItemName("appleGold");
    crate::item::item_sign::create(67).setIconCoord(10, 2).setItemName("sign");
    crate::item::item_door::create(68, Material::wood()).setIconCoord(11, 2).setItemName("doorWood");
    crate::item::item_bucket::create(69, 0).setIconCoord(10, 4).setItemName("bucket");
    crate::item::item_bucket::create(70, Block::waterMoving().m_blockID)
        .setIconCoord(11, 4)
        .setItemName("bucketWater")
        .setContainerItem(Some(Item::bucketEmpty()));
    crate::item::item_bucket::create(71, Block::lavaMoving().m_blockID)
        .setIconCoord(12, 4)
        .setItemName("bucketLava")
        .setContainerItem(Some(Item::bucketEmpty()));
    crate::item::item_minecart::create(72, 0).setIconCoord(7, 8).setItemName("minecart");
    crate::item::item_saddle::create(73).setIconCoord(8, 6).setItemName("saddle");
    crate::item::item_door::create(74, Material::iron()).setIconCoord(12, 2).setItemName("doorIron");
    crate::item::item_redstone::create(75).setIconCoord(8, 3).setItemName("redstone");
    crate::item::item_snowball::create(76).setIconCoord(14, 0).setItemName("snowball");
    crate::item::item_boat::create(77).setIconCoord(8, 8).setItemName("boat");
    Item::basic(78).setIconCoord(7, 6).setItemName("leather");
    crate::item::item_bucket::create(79, -1)
        .setIconCoord(13, 4)
        .setItemName("milk")
        .setContainerItem(Some(Item::bucketEmpty()));
    Item::basic(80).setIconCoord(6, 1).setItemName("brick");
    Item::basic(81).setIconCoord(9, 3).setItemName("clay");
    crate::item::item_reed::create(82, &Block::reed()).setIconCoord(11, 1).setItemName("reeds");
    Item::basic(83).setIconCoord(10, 3).setItemName("paper");
    Item::basic(84).setIconCoord(11, 3).setItemName("book");
    Item::basic(85).setIconCoord(14, 1).setItemName("slimeball");
    crate::item::item_minecart::create(86, 1).setIconCoord(7, 9).setItemName("minecartChest");
    crate::item::item_minecart::create(87, 2).setIconCoord(7, 10).setItemName("minecartFurnace");
    crate::item::item_egg::create(88).setIconCoord(12, 0).setItemName("egg");
    Item::basic(89).setIconCoord(6, 3).setItemName("compass");
    crate::item::item_fishing_rod::create(90).setIconCoord(5, 4).setItemName("fishingRod");
    Item::basic(91).setIconCoord(6, 4).setItemName("clock");
    Item::basic(92).setIconCoord(9, 4).setItemName("yellowDust");
    crate::item::item_food::create(93, 2, false).setIconCoord(9, 5).setItemName("fishRaw");
    crate::item::item_food::create(94, 5, false).setIconCoord(10, 5).setItemName("fishCooked");
    crate::item::item_dye::create(95).setIconCoord(14, 4).setItemName("dyePowder");
    Item::basic(96).setIconCoord(12, 1).setItemName("bone").setFull3D();
    Item::basic(97).setIconCoord(13, 0).setItemName("sugar").setFull3D();
    crate::item::item_reed::create(98, &Block::cake()).setMaxStackSize(1).setIconCoord(13, 1).setItemName("cake");
    crate::item::item_bed::create(99).setMaxStackSize(1).setIconCoord(13, 2).setItemName("bed");
    crate::item::item_reed::create(100, &Block::redstoneRepeaterIdle()).setIconCoord(6, 5).setItemName("diode");
    crate::item::item_cookie::create(101, 1, false, 8).setIconCoord(12, 5).setItemName("cookie");
    crate::item::item_map::create(102).setIconCoord(12, 3).setItemName("map");
    crate::item::item_shears::create(103).setIconCoord(13, 5).setItemName("shears");
    crate::item::item_record::create(2000, "13").setIconCoord(0, 15).setItemName("record");
    crate::item::item_record::create(2001, "cat").setIconCoord(1, 15).setItemName("record");
    crate::stats::StatList::initItemStats();
}
