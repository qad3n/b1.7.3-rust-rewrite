use crate::block::Block;
use crate::worldgen::{
    BIOME_GEN_DESERT, BIOME_GEN_RAINFOREST, BIOME_GEN_SWAMP, EnumCreatureType, SpawnListEntry, WorldGenBigTree,
    WorldGenTrees, WorldGenerator,
};
use jvm::array_list::JArrayList;
use jvm::random::Random;
use jvm::throwable::OrNpe;
use std::cell::Cell;
use std::rc::Rc;

crate::vtable! {
    BiomeGenBaseClass for BiomeGenBase {
        fn getRandomWorldGenForTrees(random: &Random) -> Box<dyn WorldGenerator>;
        fn getSkyColorByTemp(temperature: f32) -> i32;
    }
}

pub struct BiomeGenBase
{
    pub m_class: &'static BiomeGenBaseClass,
    pub m_biomeName: Cell<&'static str>,
    pub m_color: Cell<i32>,
    pub m_topBlock: Cell<i8>,
    pub m_fillerBlock: Cell<i8>,
    pub m_mutatedColor: Cell<i32>,
    pub m_spawnableMonsterList: Rc<JArrayList<SpawnListEntry>>,
    pub m_spawnableCreatureList: Rc<JArrayList<SpawnListEntry>>,
    pub m_spawnableWaterCreatureList: Rc<JArrayList<SpawnListEntry>>,
    m_enableSnow: Cell<bool>,
    m_enableRain: Cell<bool>,
}

struct Biomes
{
    m_rainforest: Rc<BiomeGenBase>,
    m_swampland: Rc<BiomeGenBase>,
    m_seasonalForest: Rc<BiomeGenBase>,
    m_forest: Rc<BiomeGenBase>,
    m_savanna: Rc<BiomeGenBase>,
    m_shrubland: Rc<BiomeGenBase>,
    m_taiga: Rc<BiomeGenBase>,
    m_desert: Rc<BiomeGenBase>,
    m_plains: Rc<BiomeGenBase>,
    m_iceDesert: Rc<BiomeGenBase>,
    m_tundra: Rc<BiomeGenBase>,
    m_hell: Rc<BiomeGenBase>,
    m_sky: Rc<BiomeGenBase>,
    m_biomeLookupTable: Vec<Option<Rc<BiomeGenBase>>>,
}

mod lookup
{
    pub const SIZE: i32 = 64;
    pub const MAX: f32 = 63.0;
}

thread_local! {
    static BIOMES: Biomes = Biomes::init();
}

macro_rules! named_biomes {
    ($($name:ident => $field:ident,)*) => {
        impl BiomeGenBase
        {
            $(pub fn $name() -> Rc<BiomeGenBase> { BIOMES.with(|biomes| Rc::clone(&biomes.$field)) })*
        }
    };
}

named_biomes! {
    rainforest => m_rainforest,
    swampland => m_swampland,
    seasonalForest => m_seasonalForest,
    forest => m_forest,
    savanna => m_savanna,
    shrubland => m_shrubland,
    taiga => m_taiga,
    desert => m_desert,
    plains => m_plains,
    iceDesert => m_iceDesert,
    tundra => m_tundra,
    hell => m_hell,
    sky => m_sky,
}

impl Biomes
{
    fn init() -> Biomes
    {
        let mut biomes: Biomes = Biomes {
            m_rainforest: BiomeGenBase::alloc(&BIOME_GEN_RAINFOREST)
                .setColor(588342)
                .setBiomeName("Rainforest")
                .setMutatedColor(2094168),
            m_swampland: BiomeGenBase::alloc(&BIOME_GEN_SWAMP)
                .setColor(522674)
                .setBiomeName("Swampland")
                .setMutatedColor(9154376),
            m_seasonalForest: BiomeGenBase::alloc(&BIOME_GEN_BASE).setColor(10215459).setBiomeName("Seasonal Forest"),
            m_forest: crate::worldgen::biome_gen_forest::create()
                .setColor(353825)
                .setBiomeName("Forest")
                .setMutatedColor(5159473),
            m_savanna: BiomeGenBase::alloc(&BIOME_GEN_DESERT).setColor(14278691).setBiomeName("Savanna"),
            m_shrubland: BiomeGenBase::alloc(&BIOME_GEN_BASE).setColor(10595616).setBiomeName("Shrubland"),
            m_taiga: crate::worldgen::biome_gen_taiga::create()
                .setColor(3060051)
                .setBiomeName("Taiga")
                .setEnableSnow()
                .setMutatedColor(8107825),
            m_desert: BiomeGenBase::alloc(&BIOME_GEN_DESERT).setColor(16421912).setBiomeName("Desert").setDisableRain(),
            m_plains: BiomeGenBase::alloc(&BIOME_GEN_DESERT).setColor(16767248).setBiomeName("Plains"),
            m_iceDesert: BiomeGenBase::alloc(&BIOME_GEN_DESERT)
                .setColor(16772499)
                .setBiomeName("Ice Desert")
                .setEnableSnow()
                .setDisableRain()
                .setMutatedColor(12899129),
            m_tundra: BiomeGenBase::alloc(&BIOME_GEN_BASE)
                .setColor(5762041)
                .setBiomeName("Tundra")
                .setEnableSnow()
                .setMutatedColor(12899129),
            m_hell: crate::worldgen::biome_gen_hell::create().setColor(0xFF0000).setBiomeName("Hell").setDisableRain(),
            m_sky: crate::worldgen::biome_gen_sky::create().setColor(0x8080FF).setBiomeName("Sky").setDisableRain(),
            m_biomeLookupTable: vec![None; (lookup::SIZE * lookup::SIZE) as usize],
        };
        biomes.generateBiomeLookup();
        biomes
    }

    fn generateBiomeLookup(&mut self)
    {
        for i in 0..lookup::SIZE
        {
            for j in 0..lookup::SIZE
            {
                self.m_biomeLookupTable[(i + j * lookup::SIZE) as usize] =
                    Some(self.getBiome(i as f32 / lookup::MAX, j as f32 / lookup::MAX));
            }
        }
        let sand: i8 = Block::sand().m_blockID as i8;
        self.m_desert.m_fillerBlock.set(sand);
        self.m_desert.m_topBlock.set(sand);
        self.m_iceDesert.m_fillerBlock.set(sand);
        self.m_iceDesert.m_topBlock.set(sand);
    }

    fn getBiome(&self, temperature: f32, mut downfall: f32) -> Rc<BiomeGenBase>
    {
        downfall *= temperature;
        let biome: &Rc<BiomeGenBase> = if temperature < 0.1f32
        {
            &self.m_tundra
        }
        else if downfall < 0.2f32
        {
            if temperature < 0.5f32
            {
                &self.m_tundra
            }
            else if temperature < 0.95f32
            {
                &self.m_savanna
            }
            else
            {
                &self.m_desert
            }
        }
        else if downfall > 0.5f32 && temperature < 0.7f32
        {
            &self.m_swampland
        }
        else if temperature < 0.5f32
        {
            &self.m_taiga
        }
        else if temperature < 0.97f32
        {
            if downfall < 0.35f32
            {
                &self.m_shrubland
            }
            else
            {
                &self.m_forest
            }
        }
        else if downfall < 0.45f32
        {
            &self.m_plains
        }
        else if downfall < 0.9f32
        {
            &self.m_seasonalForest
        }
        else
        {
            &self.m_rainforest
        };
        Rc::clone(biome)
    }
}

impl BiomeGenBase
{
    pub fn alloc(class: &'static BiomeGenBaseClass) -> Rc<BiomeGenBase>
    {
        let biome: BiomeGenBase = BiomeGenBase {
            m_class: class,
            m_biomeName: Cell::new(""),
            m_color: Cell::new(0),
            m_topBlock: Cell::new(Block::grass().m_blockID as i8),
            m_fillerBlock: Cell::new(Block::dirt().m_blockID as i8),
            m_mutatedColor: Cell::new(5169201),
            m_spawnableMonsterList: Rc::new(JArrayList::new()),
            m_spawnableCreatureList: Rc::new(JArrayList::new()),
            m_spawnableWaterCreatureList: Rc::new(JArrayList::new()),
            m_enableSnow: Cell::new(false),
            m_enableRain: Cell::new(true),
        };
        biome.m_spawnableMonsterList.add(SpawnListEntry::new("EntitySpider", 10));
        biome.m_spawnableMonsterList.add(SpawnListEntry::new("EntityZombie", 10));
        biome.m_spawnableMonsterList.add(SpawnListEntry::new("EntitySkeleton", 10));
        biome.m_spawnableMonsterList.add(SpawnListEntry::new("EntityCreeper", 10));
        biome.m_spawnableMonsterList.add(SpawnListEntry::new("EntitySlime", 10));
        biome.m_spawnableCreatureList.add(SpawnListEntry::new("EntitySheep", 12));
        biome.m_spawnableCreatureList.add(SpawnListEntry::new("EntityPig", 10));
        biome.m_spawnableCreatureList.add(SpawnListEntry::new("EntityChicken", 10));
        biome.m_spawnableCreatureList.add(SpawnListEntry::new("EntityCow", 8));
        biome.m_spawnableWaterCreatureList.add(SpawnListEntry::new("EntitySquid", 10));
        Rc::new(biome)
    }

    fn setDisableRain(self: Rc<Self>) -> Rc<Self>
    {
        self.m_enableRain.set(false);
        self
    }

    fn setEnableSnow(self: Rc<Self>) -> Rc<Self>
    {
        self.m_enableSnow.set(true);
        self
    }

    fn setBiomeName(self: Rc<Self>, name: &'static str) -> Rc<Self>
    {
        self.m_biomeName.set(name);
        self
    }

    fn setMutatedColor(self: Rc<Self>, color: i32) -> Rc<Self>
    {
        self.m_mutatedColor.set(color);
        self
    }

    fn setColor(self: Rc<Self>, color: i32) -> Rc<Self>
    {
        self.m_color.set(color);
        self
    }

    pub fn getBiomeFromLookup(temperature: f64, downfall: f64) -> Rc<BiomeGenBase>
    {
        let n: i32 = (temperature * lookup::MAX as f64) as i32;
        let n2: i32 = (downfall * lookup::MAX as f64) as i32;
        BIOMES.with(|biomes| biomes.m_biomeLookupTable[(n + n2 * lookup::SIZE) as usize].clone()).orNpe()
    }

    pub fn getBiome(temperature: f32, downfall: f32) -> Rc<BiomeGenBase>
    {
        BIOMES.with(|biomes| biomes.getBiome(temperature, downfall))
    }

    pub fn getSpawnableList(&self, category: EnumCreatureType) -> Rc<JArrayList<SpawnListEntry>>
    {
        match category
        {
            EnumCreatureType::Monster => Rc::clone(&self.m_spawnableMonsterList),
            EnumCreatureType::Creature => Rc::clone(&self.m_spawnableCreatureList),
            EnumCreatureType::WaterCreature => Rc::clone(&self.m_spawnableWaterCreatureList),
        }
    }

    pub fn getEnableSnow(&self) -> bool { self.m_enableSnow.get() }

    pub fn canSpawnLightningBolt(&self) -> bool
    {
        if self.m_enableSnow.get()
        {
            return false;
        }
        self.m_enableRain.get()
    }
}

fn getRandomWorldGenForTrees(_this: &Rc<BiomeGenBase>, random: &Random) -> Box<dyn WorldGenerator>
{
    if random.nextIntBounded(10) == 0
    {
        return Box::new(WorldGenBigTree::new());
    }
    Box::new(WorldGenTrees::new())
}

fn getSkyColorByTemp(_this: &Rc<BiomeGenBase>, mut temperature: f32) -> i32
{
    temperature /= 3.0f32;
    if temperature < -1.0f32
    {
        temperature = -1.0f32;
    }
    if temperature > 1.0f32
    {
        temperature = 1.0f32;
    }
    hsbToRgb(0.62222224f32 - temperature * 0.05f32, 0.5f32 + temperature * 0.1f32, 1.0f32)
}

fn hsbToRgb(hue: f32, saturation: f32, brightness: f32) -> i32
{
    const OPAQUE: u32 = 0xFF000000;
    let channel = |value: f32| -> i32 { (value * 255.0f32 + 0.5f32) as i32 };
    let (r, g, b): (i32, i32, i32) = if saturation == 0.0f32
    {
        let value: i32 = channel(brightness);
        (value, value, value)
    }
    else
    {
        let h: f32 = (hue - jvm::math::floor(hue as f64) as f32) * 6.0f32;
        let f: f32 = h - jvm::math::floor(h as f64) as f32;
        let p: f32 = brightness * (1.0f32 - saturation);
        let q: f32 = brightness * (1.0f32 - saturation * f);
        let t: f32 = brightness * (1.0f32 - saturation * (1.0f32 - f));
        match h as i32
        {
            0 => (channel(brightness), channel(t), channel(p)),
            1 => (channel(q), channel(brightness), channel(p)),
            2 => (channel(p), channel(brightness), channel(t)),
            3 => (channel(p), channel(q), channel(brightness)),
            4 => (channel(t), channel(p), channel(brightness)),
            5 => (channel(brightness), channel(p), channel(q)),
            _ => (0, 0, 0),
        }
    };
    (OPAQUE as i32) | (r << 16) | (g << 8) | b
}

pub const BIOME_GEN_BASE: BiomeGenBaseClass = BiomeGenBaseClass {
    m_name: "BiomeGenBase",
    m_parent: None,
    m_interfaces: &[],
    getRandomWorldGenForTrees,
    getSkyColorByTemp,
};
