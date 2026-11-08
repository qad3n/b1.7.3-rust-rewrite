use crate::block::block_pressure_plate::EnumMobType;
use crate::block::{BLOCK, Block, Material, StepSound};
use crate::tileentity::TileEntityKind;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

thread_local! {
    static INIT: Cell<u8> = const { Cell::new(0) };
    static BLOCKS_LIST: RefCell<Vec<Option<Rc<Block>>>> = RefCell::new(vec![None; 256]);
    static TICK_ON_LOAD: RefCell<[bool; 256]> = const { RefCell::new([false; 256]) };
    static OPAQUE_CUBE_LOOKUP: RefCell<[bool; 256]> = const { RefCell::new([false; 256]) };
    static IS_BLOCK_CONTAINER: RefCell<[bool; 256]> = const { RefCell::new([false; 256]) };
    static LIGHT_OPACITY: RefCell<[i32; 256]> = const { RefCell::new([0; 256]) };
    static CAN_BLOCK_GRASS: RefCell<[bool; 256]> = const { RefCell::new([false; 256]) };
    static LIGHT_VALUE: RefCell<[i32; 256]> = const { RefCell::new([0; 256]) };
    static UPDATE_CLIENTS: RefCell<[bool; 256]> = const { RefCell::new([false; 256]) };
}

pub fn ensureInit()
{
    if INIT.with(Cell::get) != 0
    {
        return;
    }
    INIT.with(|state| state.set(1));
    initBlocks();
    INIT.with(|state| state.set(2));
}

fn index(id: i32) -> usize
{
    if !(0..256).contains(&id)
    {
        jvm::throwable::throw(&jvm::throwable::ARRAY_INDEX, &id.to_string());
    }
    id as usize
}

pub(super) fn register(id: i32, block: &Rc<Block>)
{
    BLOCKS_LIST.with(|list| list.borrow_mut()[index(id)] = Some(Rc::clone(block)));
}

pub(super) fn setOpaqueCubeLookup(id: i32, value: bool)
{
    OPAQUE_CUBE_LOOKUP.with(|table| table.borrow_mut()[index(id)] = value);
}

pub(super) fn setLightOpacityValue(id: i32, value: i32)
{
    LIGHT_OPACITY.with(|table| table.borrow_mut()[index(id)] = value);
}

pub(super) fn setCanBlockGrass(id: i32, value: bool)
{
    CAN_BLOCK_GRASS.with(|table| table.borrow_mut()[index(id)] = value);
}

pub(super) fn setIsBlockContainer(id: i32, value: bool)
{
    IS_BLOCK_CONTAINER.with(|table| table.borrow_mut()[index(id)] = value);
}

pub(super) fn setUpdateClients(id: i32, value: bool)
{
    UPDATE_CLIENTS.with(|table| table.borrow_mut()[index(id)] = value);
}

pub(super) fn setLightValueEntry(id: i32, value: i32)
{
    LIGHT_VALUE.with(|table| table.borrow_mut()[index(id)] = value);
}

pub(super) fn setTickOnLoadEntry(id: i32, value: bool)
{
    TICK_ON_LOAD.with(|table| table.borrow_mut()[index(id)] = value);
}

macro_rules! named_blocks {
    ($($name:ident = $id:expr,)*) => {
        impl Block
        {
            $(pub fn $name() -> Rc<Block> { Block::named($id) })*
        }
    };
}

named_blocks! {
    stone = 1,
    grass = 2,
    dirt = 3,
    cobblestone = 4,
    planks = 5,
    sapling = 6,
    bedrock = 7,
    waterMoving = 8,
    waterStill = 9,
    lavaMoving = 10,
    lavaStill = 11,
    sand = 12,
    gravel = 13,
    oreGold = 14,
    oreIron = 15,
    oreCoal = 16,
    wood = 17,
    leaves = 18,
    sponge = 19,
    glass = 20,
    oreLapis = 21,
    blockLapis = 22,
    dispenser = 23,
    sandStone = 24,
    musicBlock = 25,
    blockBed = 26,
    railPowered = 27,
    railDetector = 28,
    pistonStickyBase = 29,
    web = 30,
    tallGrass = 31,
    deadBush = 32,
    pistonBase = 33,
    pistonExtension = 34,
    cloth = 35,
    pistonMoving = 36,
    plantYellow = 37,
    plantRed = 38,
    mushroomBrown = 39,
    mushroomRed = 40,
    blockGold = 41,
    blockSteel = 42,
    stairDouble = 43,
    stairSingle = 44,
    brick = 45,
    tnt = 46,
    bookShelf = 47,
    cobblestoneMossy = 48,
    obsidian = 49,
    torchWood = 50,
    fire = 51,
    mobSpawner = 52,
    stairCompactPlanks = 53,
    chest = 54,
    redstoneWire = 55,
    oreDiamond = 56,
    blockDiamond = 57,
    workbench = 58,
    crops = 59,
    tilledField = 60,
    stoneOvenIdle = 61,
    stoneOvenActive = 62,
    signPost = 63,
    doorWood = 64,
    ladder = 65,
    rail = 66,
    stairCompactCobblestone = 67,
    signWall = 68,
    lever = 69,
    pressurePlateStone = 70,
    doorSteel = 71,
    pressurePlatePlanks = 72,
    oreRedstone = 73,
    oreRedstoneGlowing = 74,
    torchRedstoneIdle = 75,
    torchRedstoneActive = 76,
    button = 77,
    snow = 78,
    ice = 79,
    blockSnow = 80,
    cactus = 81,
    blockClay = 82,
    reed = 83,
    jukebox = 84,
    fence = 85,
    pumpkin = 86,
    netherrack = 87,
    slowSand = 88,
    glowStone = 89,
    portal = 90,
    pumpkinLantern = 91,
    cake = 92,
    redstoneRepeaterIdle = 93,
    redstoneRepeaterActive = 94,
    lockedChest = 95,
    trapdoor = 96,
}

impl Block
{
    pub fn basic(id: i32, sprite: i32, material: Rc<Material>) -> Rc<Block>
    {
        Block::constructWithSprite(&BLOCK, Box::new(()), id, sprite, material)
    }

    pub fn byId(id: i32) -> Option<Rc<Block>>
    {
        ensureInit();
        BLOCKS_LIST.with(|list| list.borrow()[index(id)].clone())
    }

    fn named(id: i32) -> Rc<Block>
    {
        match Block::byId(id)
        {
            Some(block) => block,
            None => jvm::throwable::npe(),
        }
    }

    pub fn exists(id: i32) -> bool { BLOCKS_LIST.with(|list| list.borrow()[index(id)].is_some()) }

    pub fn tickOnLoad(id: i32) -> bool
    {
        ensureInit();
        TICK_ON_LOAD.with(|table| table.borrow()[index(id)])
    }

    pub fn opaqueCubeLookup(id: i32) -> bool
    {
        ensureInit();
        OPAQUE_CUBE_LOOKUP.with(|table| table.borrow()[index(id)])
    }

    pub fn isBlockContainer(id: i32) -> bool
    {
        ensureInit();
        IS_BLOCK_CONTAINER.with(|table| table.borrow()[index(id)])
    }

    pub fn lightOpacity(id: i32) -> i32
    {
        ensureInit();
        LIGHT_OPACITY.with(|table| table.borrow()[index(id)])
    }

    pub fn canBlockGrass(id: i32) -> bool
    {
        ensureInit();
        CAN_BLOCK_GRASS.with(|table| table.borrow()[index(id)])
    }

    pub fn lightValue(id: i32) -> i32
    {
        ensureInit();
        LIGHT_VALUE.with(|table| table.borrow()[index(id)])
    }

    pub fn updateClients(id: i32) -> bool
    {
        ensureInit();
        UPDATE_CLIENTS.with(|table| table.borrow()[index(id)])
    }

    pub fn setIsBlockContainerEntry(id: i32, value: bool) { setIsBlockContainer(id, value); }

    pub fn setTickOnLoadFlag(id: i32, value: bool) { setTickOnLoadEntry(id, value); }

    pub fn setLightOpacityEntry(id: i32, value: i32) { setLightOpacityValue(id, value); }
}

fn stone() -> Rc<Block> { Block::named(1) }

fn planks() -> Rc<Block> { Block::named(5) }

fn cobblestone() -> Rc<Block> { Block::named(4) }

fn initBlocks()
{
    crate::block::block_stone::create(1, 1)
        .setHardness(1.5)
        .setResistance(10.0)
        .setStepSound(StepSound::sounds(|s| Rc::clone(&s.m_stone)))
        .setBlockName("stone");
    crate::block::block_grass::create(2)
        .setHardness(0.6)
        .setStepSound(StepSound::sounds(|s| Rc::clone(&s.m_grass)))
        .setBlockName("grass");
    crate::block::block_dirt::create(3, 2)
        .setHardness(0.5)
        .setStepSound(StepSound::sounds(|s| Rc::clone(&s.m_gravel)))
        .setBlockName("dirt");
    Block::basic(4, 16, Material::rock())
        .setHardness(2.0)
        .setResistance(10.0)
        .setStepSound(StepSound::sounds(|s| Rc::clone(&s.m_stone)))
        .setBlockName("stonebrick");
    Block::basic(5, 4, Material::wood())
        .setHardness(2.0)
        .setResistance(5.0)
        .setStepSound(StepSound::sounds(|s| Rc::clone(&s.m_wood)))
        .setBlockName("wood")
        .disableNeighborNotifyOnMetadataChange();
    crate::block::block_sapling::create(6, 15)
        .setHardness(0.0)
        .setStepSound(StepSound::sounds(|s| Rc::clone(&s.m_grass)))
        .setBlockName("sapling")
        .disableNeighborNotifyOnMetadataChange();
    Block::basic(7, 17, Material::rock())
        .setBlockUnbreakable()
        .setResistance(6000000.0)
        .setStepSound(StepSound::sounds(|s| Rc::clone(&s.m_stone)))
        .setBlockName("bedrock")
        .disableStats();
    crate::block::block_flowing::create(8, Material::water())
        .setHardness(100.0)
        .setLightOpacity(3)
        .setBlockName("water")
        .disableStats()
        .disableNeighborNotifyOnMetadataChange();
    crate::block::block_stationary::create(9, Material::water())
        .setHardness(100.0)
        .setLightOpacity(3)
        .setBlockName("water")
        .disableStats()
        .disableNeighborNotifyOnMetadataChange();
    crate::block::block_flowing::create(10, Material::lava())
        .setHardness(0.0)
        .setLightValue(1.0)
        .setLightOpacity(255)
        .setBlockName("lava")
        .disableStats()
        .disableNeighborNotifyOnMetadataChange();
    crate::block::block_stationary::create(11, Material::lava())
        .setHardness(100.0)
        .setLightValue(1.0)
        .setLightOpacity(255)
        .setBlockName("lava")
        .disableStats()
        .disableNeighborNotifyOnMetadataChange();
    crate::block::block_sand::create(12, 18)
        .setHardness(0.5)
        .setStepSound(StepSound::sounds(|s| Rc::clone(&s.m_sand)))
        .setBlockName("sand");
    crate::block::block_gravel::create(13, 19)
        .setHardness(0.6)
        .setStepSound(StepSound::sounds(|s| Rc::clone(&s.m_gravel)))
        .setBlockName("gravel");
    crate::block::block_ore::create(14, 32)
        .setHardness(3.0)
        .setResistance(5.0)
        .setStepSound(StepSound::sounds(|s| Rc::clone(&s.m_stone)))
        .setBlockName("oreGold");
    crate::block::block_ore::create(15, 33)
        .setHardness(3.0)
        .setResistance(5.0)
        .setStepSound(StepSound::sounds(|s| Rc::clone(&s.m_stone)))
        .setBlockName("oreIron");
    crate::block::block_ore::create(16, 34)
        .setHardness(3.0)
        .setResistance(5.0)
        .setStepSound(StepSound::sounds(|s| Rc::clone(&s.m_stone)))
        .setBlockName("oreCoal");
    crate::block::block_log::create(17)
        .setHardness(2.0)
        .setStepSound(StepSound::sounds(|s| Rc::clone(&s.m_wood)))
        .setBlockName("log")
        .disableNeighborNotifyOnMetadataChange();
    crate::block::block_leaves::create(18, 52)
        .setHardness(0.2)
        .setLightOpacity(1)
        .setStepSound(StepSound::sounds(|s| Rc::clone(&s.m_grass)))
        .setBlockName("leaves")
        .disableStats()
        .disableNeighborNotifyOnMetadataChange();
    crate::block::block_sponge::create(19)
        .setHardness(0.6)
        .setStepSound(StepSound::sounds(|s| Rc::clone(&s.m_grass)))
        .setBlockName("sponge");
    crate::block::block_glass::create(20, 49, Material::glass(), false)
        .setHardness(0.3)
        .setStepSound(StepSound::sounds(|s| Rc::clone(&s.m_glass)))
        .setBlockName("glass");
    crate::block::block_ore::create(21, 160)
        .setHardness(3.0)
        .setResistance(5.0)
        .setStepSound(StepSound::sounds(|s| Rc::clone(&s.m_stone)))
        .setBlockName("oreLapis");
    Block::basic(22, 144, Material::rock())
        .setHardness(3.0)
        .setResistance(5.0)
        .setStepSound(StepSound::sounds(|s| Rc::clone(&s.m_stone)))
        .setBlockName("blockLapis");
    crate::block::block_dispenser::create(23)
        .setHardness(3.5)
        .setStepSound(StepSound::sounds(|s| Rc::clone(&s.m_stone)))
        .setBlockName("dispenser")
        .disableNeighborNotifyOnMetadataChange();
    crate::block::block_sand_stone::create(24)
        .setStepSound(StepSound::sounds(|s| Rc::clone(&s.m_stone)))
        .setHardness(0.8)
        .setBlockName("sandStone");
    crate::block::block_note::create(25)
        .setHardness(0.8)
        .setBlockName("musicBlock")
        .disableNeighborNotifyOnMetadataChange();
    crate::block::block_bed::create(26)
        .setHardness(0.2)
        .setBlockName("bed")
        .disableStats()
        .disableNeighborNotifyOnMetadataChange();
    crate::block::block_rail::create(27, 179, true)
        .setHardness(0.7)
        .setStepSound(StepSound::sounds(|s| Rc::clone(&s.m_metal)))
        .setBlockName("goldenRail")
        .disableNeighborNotifyOnMetadataChange();
    crate::block::block_detector_rail::create(28, 195)
        .setHardness(0.7)
        .setStepSound(StepSound::sounds(|s| Rc::clone(&s.m_metal)))
        .setBlockName("detectorRail")
        .disableNeighborNotifyOnMetadataChange();
    crate::block::block_piston_base::create(29, 106, true)
        .setBlockName("pistonStickyBase")
        .disableNeighborNotifyOnMetadataChange();
    crate::block::block_web::create(30, 11).setLightOpacity(1).setHardness(4.0).setBlockName("web");
    crate::block::block_tall_grass::create(31, 39)
        .setHardness(0.0)
        .setStepSound(StepSound::sounds(|s| Rc::clone(&s.m_grass)))
        .setBlockName("tallgrass");
    crate::block::block_dead_bush::create(32, 55)
        .setHardness(0.0)
        .setStepSound(StepSound::sounds(|s| Rc::clone(&s.m_grass)))
        .setBlockName("deadbush");
    crate::block::block_piston_base::create(33, 107, false)
        .setBlockName("pistonBase")
        .disableNeighborNotifyOnMetadataChange();
    crate::block::block_piston_extension::create(34, 107).disableNeighborNotifyOnMetadataChange();
    crate::block::block_cloth::create()
        .setHardness(0.8)
        .setStepSound(StepSound::sounds(|s| Rc::clone(&s.m_cloth)))
        .setBlockName("cloth")
        .disableNeighborNotifyOnMetadataChange();
    crate::block::block_piston_moving::create(36);
    crate::block::block_flower::create(37, 13)
        .setHardness(0.0)
        .setStepSound(StepSound::sounds(|s| Rc::clone(&s.m_grass)))
        .setBlockName("flower");
    crate::block::block_flower::create(38, 12)
        .setHardness(0.0)
        .setStepSound(StepSound::sounds(|s| Rc::clone(&s.m_grass)))
        .setBlockName("rose");
    crate::block::block_mushroom::create(39, 29)
        .setHardness(0.0)
        .setStepSound(StepSound::sounds(|s| Rc::clone(&s.m_grass)))
        .setLightValue(0.125)
        .setBlockName("mushroom");
    crate::block::block_mushroom::create(40, 28)
        .setHardness(0.0)
        .setStepSound(StepSound::sounds(|s| Rc::clone(&s.m_grass)))
        .setBlockName("mushroom");
    crate::block::block_ore_storage::create(41, 23)
        .setHardness(3.0)
        .setResistance(10.0)
        .setStepSound(StepSound::sounds(|s| Rc::clone(&s.m_metal)))
        .setBlockName("blockGold");
    crate::block::block_ore_storage::create(42, 22)
        .setHardness(5.0)
        .setResistance(10.0)
        .setStepSound(StepSound::sounds(|s| Rc::clone(&s.m_metal)))
        .setBlockName("blockIron");
    crate::block::block_step::create(43, true)
        .setHardness(2.0)
        .setResistance(10.0)
        .setStepSound(StepSound::sounds(|s| Rc::clone(&s.m_stone)))
        .setBlockName("stoneSlab");
    crate::block::block_step::create(44, false)
        .setHardness(2.0)
        .setResistance(10.0)
        .setStepSound(StepSound::sounds(|s| Rc::clone(&s.m_stone)))
        .setBlockName("stoneSlab");
    Block::basic(45, 7, Material::rock())
        .setHardness(2.0)
        .setResistance(10.0)
        .setStepSound(StepSound::sounds(|s| Rc::clone(&s.m_stone)))
        .setBlockName("brick");
    crate::block::block_tnt::create(46, 8)
        .setHardness(0.0)
        .setStepSound(StepSound::sounds(|s| Rc::clone(&s.m_grass)))
        .setBlockName("tnt");
    crate::block::block_bookshelf::create(47, 35)
        .setHardness(1.5)
        .setStepSound(StepSound::sounds(|s| Rc::clone(&s.m_wood)))
        .setBlockName("bookshelf");
    Block::basic(48, 36, Material::rock())
        .setHardness(2.0)
        .setResistance(10.0)
        .setStepSound(StepSound::sounds(|s| Rc::clone(&s.m_stone)))
        .setBlockName("stoneMoss");
    crate::block::block_obsidian::create(49, 37)
        .setHardness(10.0)
        .setResistance(2000.0)
        .setStepSound(StepSound::sounds(|s| Rc::clone(&s.m_stone)))
        .setBlockName("obsidian");
    crate::block::block_torch::create(50, 80)
        .setHardness(0.0)
        .setLightValue(0.9375)
        .setStepSound(StepSound::sounds(|s| Rc::clone(&s.m_wood)))
        .setBlockName("torch")
        .disableNeighborNotifyOnMetadataChange();
    crate::block::block_fire::create(51, 31)
        .setHardness(0.0)
        .setLightValue(1.0)
        .setStepSound(StepSound::sounds(|s| Rc::clone(&s.m_wood)))
        .setBlockName("fire")
        .disableStats()
        .disableNeighborNotifyOnMetadataChange();
    crate::block::block_mob_spawner::create(52, 65)
        .setHardness(5.0)
        .setStepSound(StepSound::sounds(|s| Rc::clone(&s.m_metal)))
        .setBlockName("mobSpawner")
        .disableStats();
    crate::block::block_stairs::create(53, planks()).setBlockName("stairsWood").disableNeighborNotifyOnMetadataChange();
    crate::block::block_chest::create(54)
        .setHardness(2.5)
        .setStepSound(StepSound::sounds(|s| Rc::clone(&s.m_wood)))
        .setBlockName("chest")
        .disableNeighborNotifyOnMetadataChange();
    crate::block::block_redstone_wire::create(55, 164)
        .setHardness(0.0)
        .setStepSound(StepSound::sounds(|s| Rc::clone(&s.m_powder)))
        .setBlockName("redstoneDust")
        .disableStats()
        .disableNeighborNotifyOnMetadataChange();
    crate::block::block_ore::create(56, 50)
        .setHardness(3.0)
        .setResistance(5.0)
        .setStepSound(StepSound::sounds(|s| Rc::clone(&s.m_stone)))
        .setBlockName("oreDiamond");
    crate::block::block_ore_storage::create(57, 24)
        .setHardness(5.0)
        .setResistance(10.0)
        .setStepSound(StepSound::sounds(|s| Rc::clone(&s.m_metal)))
        .setBlockName("blockDiamond");
    crate::block::block_workbench::create(58)
        .setHardness(2.5)
        .setStepSound(StepSound::sounds(|s| Rc::clone(&s.m_wood)))
        .setBlockName("workbench");
    crate::block::block_crops::create(59, 88)
        .setHardness(0.0)
        .setStepSound(StepSound::sounds(|s| Rc::clone(&s.m_grass)))
        .setBlockName("crops")
        .disableStats()
        .disableNeighborNotifyOnMetadataChange();
    crate::block::block_farmland::create(60)
        .setHardness(0.6)
        .setStepSound(StepSound::sounds(|s| Rc::clone(&s.m_gravel)))
        .setBlockName("farmland");
    crate::block::block_furnace::create(61, false)
        .setHardness(3.5)
        .setStepSound(StepSound::sounds(|s| Rc::clone(&s.m_stone)))
        .setBlockName("furnace")
        .disableNeighborNotifyOnMetadataChange();
    crate::block::block_furnace::create(62, true)
        .setHardness(3.5)
        .setStepSound(StepSound::sounds(|s| Rc::clone(&s.m_stone)))
        .setLightValue(0.875)
        .setBlockName("furnace")
        .disableNeighborNotifyOnMetadataChange();
    crate::block::block_sign::create(63, TileEntityKind::Sign, true)
        .setHardness(1.0)
        .setStepSound(StepSound::sounds(|s| Rc::clone(&s.m_wood)))
        .setBlockName("sign")
        .disableStats()
        .disableNeighborNotifyOnMetadataChange();
    crate::block::block_door::create(64, Material::wood())
        .setHardness(3.0)
        .setStepSound(StepSound::sounds(|s| Rc::clone(&s.m_wood)))
        .setBlockName("doorWood")
        .disableStats()
        .disableNeighborNotifyOnMetadataChange();
    crate::block::block_ladder::create(65, 83)
        .setHardness(0.4)
        .setStepSound(StepSound::sounds(|s| Rc::clone(&s.m_wood)))
        .setBlockName("ladder")
        .disableNeighborNotifyOnMetadataChange();
    crate::block::block_rail::create(66, 128, false)
        .setHardness(0.7)
        .setStepSound(StepSound::sounds(|s| Rc::clone(&s.m_metal)))
        .setBlockName("rail")
        .disableNeighborNotifyOnMetadataChange();
    crate::block::block_stairs::create(67, cobblestone())
        .setBlockName("stairsStone")
        .disableNeighborNotifyOnMetadataChange();
    crate::block::block_sign::create(68, TileEntityKind::Sign, false)
        .setHardness(1.0)
        .setStepSound(StepSound::sounds(|s| Rc::clone(&s.m_wood)))
        .setBlockName("sign")
        .disableStats()
        .disableNeighborNotifyOnMetadataChange();
    crate::block::block_lever::create(69, 96)
        .setHardness(0.5)
        .setStepSound(StepSound::sounds(|s| Rc::clone(&s.m_wood)))
        .setBlockName("lever")
        .disableNeighborNotifyOnMetadataChange();
    crate::block::block_pressure_plate::create(
        70,
        stone().m_blockIndexInTexture.get(),
        EnumMobType::Mobs,
        Material::rock(),
    )
    .setHardness(0.5)
    .setStepSound(StepSound::sounds(|s| Rc::clone(&s.m_stone)))
    .setBlockName("pressurePlate")
    .disableNeighborNotifyOnMetadataChange();
    crate::block::block_door::create(71, Material::iron())
        .setHardness(5.0)
        .setStepSound(StepSound::sounds(|s| Rc::clone(&s.m_metal)))
        .setBlockName("doorIron")
        .disableStats()
        .disableNeighborNotifyOnMetadataChange();
    crate::block::block_pressure_plate::create(
        72,
        planks().m_blockIndexInTexture.get(),
        EnumMobType::Everything,
        Material::wood(),
    )
    .setHardness(0.5)
    .setStepSound(StepSound::sounds(|s| Rc::clone(&s.m_wood)))
    .setBlockName("pressurePlate")
    .disableNeighborNotifyOnMetadataChange();
    crate::block::block_redstone_ore::create(73, 51, false)
        .setHardness(3.0)
        .setResistance(5.0)
        .setStepSound(StepSound::sounds(|s| Rc::clone(&s.m_stone)))
        .setBlockName("oreRedstone")
        .disableNeighborNotifyOnMetadataChange();
    crate::block::block_redstone_ore::create(74, 51, true)
        .setLightValue(0.625)
        .setHardness(3.0)
        .setResistance(5.0)
        .setStepSound(StepSound::sounds(|s| Rc::clone(&s.m_stone)))
        .setBlockName("oreRedstone")
        .disableNeighborNotifyOnMetadataChange();
    crate::block::block_redstone_torch::create(75, 115, false)
        .setHardness(0.0)
        .setStepSound(StepSound::sounds(|s| Rc::clone(&s.m_wood)))
        .setBlockName("notGate")
        .disableNeighborNotifyOnMetadataChange();
    crate::block::block_redstone_torch::create(76, 99, true)
        .setHardness(0.0)
        .setLightValue(0.5)
        .setStepSound(StepSound::sounds(|s| Rc::clone(&s.m_wood)))
        .setBlockName("notGate")
        .disableNeighborNotifyOnMetadataChange();
    crate::block::block_button::create(77, stone().m_blockIndexInTexture.get())
        .setHardness(0.5)
        .setStepSound(StepSound::sounds(|s| Rc::clone(&s.m_stone)))
        .setBlockName("button")
        .disableNeighborNotifyOnMetadataChange();
    crate::block::block_snow::create(78, 66)
        .setHardness(0.1)
        .setStepSound(StepSound::sounds(|s| Rc::clone(&s.m_cloth)))
        .setBlockName("snow");
    crate::block::block_ice::create(79, 67)
        .setHardness(0.5)
        .setLightOpacity(3)
        .setStepSound(StepSound::sounds(|s| Rc::clone(&s.m_glass)))
        .setBlockName("ice");
    crate::block::block_snow_block::create(80, 66)
        .setHardness(0.2)
        .setStepSound(StepSound::sounds(|s| Rc::clone(&s.m_cloth)))
        .setBlockName("snow");
    crate::block::block_cactus::create(81, 70)
        .setHardness(0.4)
        .setStepSound(StepSound::sounds(|s| Rc::clone(&s.m_cloth)))
        .setBlockName("cactus");
    crate::block::block_clay::create(82, 72)
        .setHardness(0.6)
        .setStepSound(StepSound::sounds(|s| Rc::clone(&s.m_gravel)))
        .setBlockName("clay");
    crate::block::block_reed::create(83, 73)
        .setHardness(0.0)
        .setStepSound(StepSound::sounds(|s| Rc::clone(&s.m_grass)))
        .setBlockName("reeds")
        .disableStats();
    crate::block::block_juke_box::create(84, 74)
        .setHardness(2.0)
        .setResistance(10.0)
        .setStepSound(StepSound::sounds(|s| Rc::clone(&s.m_stone)))
        .setBlockName("jukebox")
        .disableNeighborNotifyOnMetadataChange();
    crate::block::block_fence::create(85, 4)
        .setHardness(2.0)
        .setResistance(5.0)
        .setStepSound(StepSound::sounds(|s| Rc::clone(&s.m_wood)))
        .setBlockName("fence")
        .disableNeighborNotifyOnMetadataChange();
    crate::block::block_pumpkin::create(86, 102, false)
        .setHardness(1.0)
        .setStepSound(StepSound::sounds(|s| Rc::clone(&s.m_wood)))
        .setBlockName("pumpkin")
        .disableNeighborNotifyOnMetadataChange();
    crate::block::block_netherrack::create(87, 103)
        .setHardness(0.4)
        .setStepSound(StepSound::sounds(|s| Rc::clone(&s.m_stone)))
        .setBlockName("hellrock");
    crate::block::block_soul_sand::create(88, 104)
        .setHardness(0.5)
        .setStepSound(StepSound::sounds(|s| Rc::clone(&s.m_sand)))
        .setBlockName("hellsand");
    crate::block::block_glow_stone::create(89, 105, Material::rock())
        .setHardness(0.3)
        .setStepSound(StepSound::sounds(|s| Rc::clone(&s.m_glass)))
        .setLightValue(1.0)
        .setBlockName("lightgem");
    crate::block::block_portal::create(90, 14)
        .setHardness(-1.0)
        .setStepSound(StepSound::sounds(|s| Rc::clone(&s.m_glass)))
        .setLightValue(0.75)
        .setBlockName("portal");
    crate::block::block_pumpkin::create(91, 102, true)
        .setHardness(1.0)
        .setStepSound(StepSound::sounds(|s| Rc::clone(&s.m_wood)))
        .setLightValue(1.0)
        .setBlockName("litpumpkin")
        .disableNeighborNotifyOnMetadataChange();
    crate::block::block_cake::create(92, 121)
        .setHardness(0.5)
        .setStepSound(StepSound::sounds(|s| Rc::clone(&s.m_cloth)))
        .setBlockName("cake")
        .disableStats()
        .disableNeighborNotifyOnMetadataChange();
    crate::block::block_redstone_repeater::create(93, false)
        .setHardness(0.0)
        .setStepSound(StepSound::sounds(|s| Rc::clone(&s.m_wood)))
        .setBlockName("diode")
        .disableStats()
        .disableNeighborNotifyOnMetadataChange();
    crate::block::block_redstone_repeater::create(94, true)
        .setHardness(0.0)
        .setLightValue(0.625)
        .setStepSound(StepSound::sounds(|s| Rc::clone(&s.m_wood)))
        .setBlockName("diode")
        .disableStats()
        .disableNeighborNotifyOnMetadataChange();
    crate::block::block_locked_chest::create(95)
        .setHardness(0.0)
        .setLightValue(1.0)
        .setStepSound(StepSound::sounds(|s| Rc::clone(&s.m_wood)))
        .setBlockName("lockedchest")
        .setTickOnLoad(true)
        .disableNeighborNotifyOnMetadataChange();
    crate::block::block_trap_door::create(96, Material::wood())
        .setHardness(3.0)
        .setStepSound(StepSound::sounds(|s| Rc::clone(&s.m_wood)))
        .setBlockName("trapdoor")
        .disableStats()
        .disableNeighborNotifyOnMetadataChange();
    crate::item::Item::registerBlockItems();
    CAN_BLOCK_GRASS.with(|table| table.borrow_mut()[0] = true);
    crate::stats::StatList::initBlockStats();
}
