use crate::block::{Block, Material};
use crate::item::item::EnumToolMaterial;
use crate::item::{ITEM, Item};
use jvm::random::Random;
use jvm::throwable::OrNpe;
use std::cell::{Cell, OnceCell, RefCell};
use std::rc::Rc;

thread_local!
{
    static INIT: Cell<u8> = const { Cell::new(0) };
    static ITEM_RAND: OnceCell<Rc<Random>> = const { OnceCell::new() };
    static ITEMS_LIST: RefCell<Vec<Option<Rc<Item>>>> = RefCell::new(vec![None; 32000]);
}

pub(crate) fn ensureInit()
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

pub(crate) fn register(id: i32, item: &Rc<Item>)
{
    ITEMS_LIST.with(|list| list.borrow_mut()[index(id)] = Some(Rc::clone(item)));
}

macro_rules! named_items
{
    ($($name:ident = $id:literal $key:literal,)*) =>
    {
        impl Item
        {
            // the registry name of every vanilla item beside its numeric id, in id order
            pub const NAMES: &'static [(i32, &'static str)] = &[$((256 + $id, $key),)*];

            $(pub fn $name() -> Rc<Item> { Item::byId(256 + $id).orNpe() })*
        }
    };
}

named_items!
{
    shovelSteel = 0 "iron_shovel",
    pickaxeSteel = 1 "iron_pickaxe",
    axeSteel = 2 "iron_axe",
    flintAndSteel = 3 "flint_and_steel",
    appleRed = 4 "apple",
    bow = 5 "bow",
    arrow = 6 "arrow",
    coal = 7 "coal",
    diamond = 8 "diamond",
    ingotIron = 9 "iron_ingot",
    ingotGold = 10 "gold_ingot",
    swordSteel = 11 "iron_sword",
    swordWood = 12 "wooden_sword",
    shovelWood = 13 "wooden_shovel",
    pickaxeWood = 14 "wooden_pickaxe",
    axeWood = 15 "wooden_axe",
    swordStone = 16 "stone_sword",
    shovelStone = 17 "stone_shovel",
    pickaxeStone = 18 "stone_pickaxe",
    axeStone = 19 "stone_axe",
    swordDiamond = 20 "diamond_sword",
    shovelDiamond = 21 "diamond_shovel",
    pickaxeDiamond = 22 "diamond_pickaxe",
    axeDiamond = 23 "diamond_axe",
    stick = 24 "stick",
    bowlEmpty = 25 "bowl",
    bowlSoup = 26 "mushroom_stew",
    swordGold = 27 "golden_sword",
    shovelGold = 28 "golden_shovel",
    pickaxeGold = 29 "golden_pickaxe",
    axeGold = 30 "golden_axe",
    silk = 31 "string",
    feather = 32 "feather",
    gunpowder = 33 "gunpowder",
    hoeWood = 34 "wooden_hoe",
    hoeStone = 35 "stone_hoe",
    hoeSteel = 36 "iron_hoe",
    hoeDiamond = 37 "diamond_hoe",
    hoeGold = 38 "golden_hoe",
    seeds = 39 "wheat_seeds",
    wheat = 40 "wheat",
    bread = 41 "bread",
    helmetLeather = 42 "leather_helmet",
    plateLeather = 43 "leather_chestplate",
    legsLeather = 44 "leather_leggings",
    bootsLeather = 45 "leather_boots",
    helmetChain = 46 "chainmail_helmet",
    plateChain = 47 "chainmail_chestplate",
    legsChain = 48 "chainmail_leggings",
    bootsChain = 49 "chainmail_boots",
    helmetSteel = 50 "iron_helmet",
    plateSteel = 51 "iron_chestplate",
    legsSteel = 52 "iron_leggings",
    bootsSteel = 53 "iron_boots",
    helmetDiamond = 54 "diamond_helmet",
    plateDiamond = 55 "diamond_chestplate",
    legsDiamond = 56 "diamond_leggings",
    bootsDiamond = 57 "diamond_boots",
    helmetGold = 58 "golden_helmet",
    plateGold = 59 "golden_chestplate",
    legsGold = 60 "golden_leggings",
    bootsGold = 61 "golden_boots",
    flint = 62 "flint",
    porkRaw = 63 "porkchop",
    porkCooked = 64 "cooked_porkchop",
    painting = 65 "painting",
    appleGold = 66 "golden_apple",
    sign = 67 "sign",
    doorWood = 68 "wooden_door",
    bucketEmpty = 69 "bucket",
    bucketWater = 70 "water_bucket",
    bucketLava = 71 "lava_bucket",
    minecartEmpty = 72 "minecart",
    saddle = 73 "saddle",
    doorSteel = 74 "iron_door",
    redstone = 75 "redstone",
    snowball = 76 "snowball",
    boat = 77 "boat",
    leather = 78 "leather",
    bucketMilk = 79 "milk_bucket",
    brick = 80 "brick",
    clay = 81 "clay_ball",
    reed = 82 "reeds",
    paper = 83 "paper",
    book = 84 "book",
    slimeBall = 85 "slime_ball",
    minecartCrate = 86 "chest_minecart",
    minecartPowered = 87 "furnace_minecart",
    egg = 88 "egg",
    compass = 89 "compass",
    fishingRod = 90 "fishing_rod",
    pocketSundial = 91 "clock",
    lightStoneDust = 92 "glowstone_dust",
    fishRaw = 93 "fish",
    fishCooked = 94 "cooked_fish",
    dyePowder = 95 "dye",
    bone = 96 "bone",
    sugar = 97 "sugar",
    cake = 98 "cake",
    bed = 99 "bed",
    redstoneRepeater = 100 "repeater",
    cookie = 101 "cookie",
    mapItem = 102 "filled_map",
    shears = 103 "shears",
    record13 = 2000 "record_13",
    recordCat = 2001 "record_cat",
}

impl Item
{
    pub fn basic(id: i32) -> Rc<Self> { Self::construct(&ITEM, Box::new(()), id) }

    pub fn byId(id: i32) -> Option<Rc<Self>>
    {
        ensureInit();
        ITEMS_LIST.with(|list| list.borrow()[index(id)].clone())
    }

    pub fn byIdOrNpe(id: i32) -> Rc<Self> { Self::byId(id).orNpe() }

    // the mod loader rebuilds mod items when a world or server maps them to other ids
    pub fn unregister(id: i32) { ITEMS_LIST.with(|list| list.borrow_mut()[index(id)] = None); }

    pub fn itemRand() -> Rc<Random>
    {
        ensureInit();
        ITEM_RAND.with(|random| random.get().cloned()).orNpe()
    }

    pub fn registerBlockItems()
    {
        let set = |id: i32, item: Rc<Self>| register(id, &item);

        ensureInit();
        set(Block::CLOTH, crate::item::item_cloth::create(Block::CLOTH - 256).setItemName("cloth"));
        set(Block::WOOD, crate::item::item_log::create(Block::WOOD - 256).setItemName("log"));
        set(Block::STAIR_SINGLE, crate::item::item_slab::create(Block::STAIR_SINGLE - 256).setItemName("stoneSlab"));
        set(Block::SAPLING, crate::item::item_sapling::create(Block::SAPLING - 256).setItemName("sapling"));
        set(Block::LEAVES, crate::item::item_leaves::create(Block::LEAVES - 256).setItemName("leaves"));
        set(Block::PISTON_BASE, crate::item::item_piston::create(Block::PISTON_BASE - 256));
        set(Block::PISTON_STICKY_BASE, crate::item::item_piston::create(Block::PISTON_STICKY_BASE - 256));

        for i in 0..256
        {
            let Some(block) = Block::byId(i)
            else
            {
                continue;
            };
            if Self::byId(i).is_some()
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
    crate::item::item_food::create(4, 4).setIconCoord(10, 0).setItemName("apple");
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
    crate::item::item_seeds::create(39, Block::CROPS).setIconCoord(9, 0).setItemName("seeds");
    Item::basic(40).setIconCoord(9, 1).setItemName("wheat");
    crate::item::item_food::create(41, 5).setIconCoord(9, 2).setItemName("bread");
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
    crate::item::item_food::create(63, 3).setIconCoord(7, 5).setItemName("porkchopRaw");
    crate::item::item_food::create(64, 8).setIconCoord(8, 5).setItemName("porkchopCooked");
    crate::item::item_painting::create(65).setIconCoord(10, 1).setItemName("painting");
    crate::item::item_food::create(66, 42).setIconCoord(11, 0).setItemName("appleGold");
    crate::item::item_sign::create(67).setIconCoord(10, 2).setItemName("sign");
    crate::item::item_door::create(68, Material::wood()).setIconCoord(11, 2).setItemName("doorWood");
    crate::item::item_bucket::create(69, 0).setIconCoord(10, 4).setItemName("bucket");
    crate::item::item_bucket::create(70, Block::WATER_MOVING)
        .setIconCoord(11, 4)
        .setItemName("bucketWater")
        .setContainerItem(Some(Item::bucketEmpty()));
    crate::item::item_bucket::create(71, Block::LAVA_MOVING)
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
    crate::item::item_food::create(93, 2).setIconCoord(9, 5).setItemName("fishRaw");
    crate::item::item_food::create(94, 5).setIconCoord(10, 5).setItemName("fishCooked");
    crate::item::item_dye::create(95).setIconCoord(14, 4).setItemName("dyePowder");
    Item::basic(96).setIconCoord(12, 1).setItemName("bone").setFull3D();
    Item::basic(97).setIconCoord(13, 0).setItemName("sugar").setFull3D();
    crate::item::item_reed::create(98, &Block::cake()).setMaxStackSize(1).setIconCoord(13, 1).setItemName("cake");
    crate::item::item_bed::create(99).setMaxStackSize(1).setIconCoord(13, 2).setItemName("bed");
    crate::item::item_reed::create(100, &Block::redstoneRepeaterIdle()).setIconCoord(6, 5).setItemName("diode");
    crate::item::item_cookie::create(101, 1, 8).setIconCoord(12, 5).setItemName("cookie");
    crate::item::item_map::create(102).setIconCoord(12, 3).setItemName("map");
    crate::item::item_shears::create(103).setIconCoord(13, 5).setItemName("shears");
    crate::item::item_record::create(2000, "13").setIconCoord(0, 15).setItemName("record");
    crate::item::item_record::create(2001, "cat").setIconCoord(1, 15).setItemName("record");
    crate::stats::StatList::initItemStats();
}
