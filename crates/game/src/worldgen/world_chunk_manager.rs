use crate::world::World;
use crate::worldgen::BiomeGenBase;
use common::noise::NoiseGeneratorOctaves2;
use common::world::ChunkCoordIntPair;
use jvm::field::Field;
use jvm::random::Random;
use jvm::throwable::OrNpe;
use std::rc::Rc;

crate::vtable! {
    WorldChunkManagerClass for WorldChunkManager {
        fn getBiomeGenAtChunkCoord(pos: ChunkCoordIntPair) -> Rc<BiomeGenBase>;
        fn getBiomeGenAt(x: i32, z: i32) -> Rc<BiomeGenBase>;
        fn getTemperature(x: i32, z: i32) -> f64;
        fn getBiomes(x: i32, z: i32, sizeX: i32, sizeZ: i32) -> Vec<Option<Rc<BiomeGenBase>>>;
        fn getTemperatures(temperatures: Option<Vec<f64>>, x: i32, z: i32, sizeX: i32, sizeZ: i32) -> Vec<f64>;
        fn loadBlockGeneratorData(
            biomes: Option<Vec<Option<Rc<BiomeGenBase>>>>,
            x: i32,
            z: i32,
            sizeX: i32,
            sizeZ: i32
        ) -> Vec<Option<Rc<BiomeGenBase>>>;
    }
}

pub struct WorldChunkManager
{
    pub m_class: &'static WorldChunkManagerClass,
    m_temperatureNoise: Option<NoiseGeneratorOctaves2>,
    m_downfallNoise: Option<NoiseGeneratorOctaves2>,
    m_biomeNoise: Option<NoiseGeneratorOctaves2>,
    pub m_temperature: Field<Option<Vec<f64>>>,
    pub m_humidity: Field<Option<Vec<f64>>>,
    pub m_noises: Field<Option<Vec<f64>>>,
    pub m_biomesBuffer: Field<Option<Vec<Option<Rc<BiomeGenBase>>>>>,
    m_hell: Option<HellExt>,
}

struct HellExt
{
    m_biome: Rc<BiomeGenBase>,
    m_hellTemperature: f64,
    m_downfall: f64,
}

impl WorldChunkManager
{
    fn alloc(
        class: &'static WorldChunkManagerClass,
        noises: Option<[NoiseGeneratorOctaves2; 3]>,
        hell: Option<HellExt>,
    ) -> Rc<WorldChunkManager>
    {
        let (temperatureNoise, downfallNoise, biomeNoise): (
            Option<NoiseGeneratorOctaves2>,
            Option<NoiseGeneratorOctaves2>,
            Option<NoiseGeneratorOctaves2>,
        ) = match noises
        {
            Some([temperature, downfall, biome]) => (Some(temperature), Some(downfall), Some(biome)),
            None => (None, None, None),
        };
        Rc::new(WorldChunkManager {
            m_class: class,
            m_temperatureNoise: temperatureNoise,
            m_downfallNoise: downfallNoise,
            m_biomeNoise: biomeNoise,
            m_temperature: Field::new(None),
            m_humidity: Field::new(None),
            m_noises: Field::new(None),
            m_biomesBuffer: Field::new(None),
            m_hell: hell,
        })
    }

    pub fn new(world: &Rc<World>) -> Rc<WorldChunkManager>
    {
        let temperatureNoise: NoiseGeneratorOctaves2 =
            NoiseGeneratorOctaves2::new(&Random::withSeed(world.getRandomSeed() * 9871i64), 4);
        let downfallNoise: NoiseGeneratorOctaves2 =
            NoiseGeneratorOctaves2::new(&Random::withSeed(world.getRandomSeed() * 39811i64), 4);
        let biomeNoise: NoiseGeneratorOctaves2 =
            NoiseGeneratorOctaves2::new(&Random::withSeed(world.getRandomSeed() * 543321i64), 2);
        Self::alloc(&WORLD_CHUNK_MANAGER, Some([temperatureNoise, downfallNoise, biomeNoise]), None)
    }

    pub fn newHell(biome: Rc<BiomeGenBase>, temperature: f64, downfall: f64) -> Rc<WorldChunkManager>
    {
        let hell: HellExt = HellExt { m_biome: biome, m_hellTemperature: temperature, m_downfall: downfall };
        Self::alloc(&WORLD_CHUNK_MANAGER_HELL, None, Some(hell))
    }

    fn hell(&self) -> &HellExt { self.m_hell.as_ref().expect("hell chunk manager holds hell fields") }

    fn temperatureNoise(&self) -> &NoiseGeneratorOctaves2 { self.m_temperatureNoise.as_ref().orNpe() }

    pub fn temperature(&self, index: usize) -> f64 { self.m_temperature.with(|values| values.as_ref().orNpe()[index]) }

    pub fn humidity(&self, index: usize) -> f64 { self.m_humidity.with(|values| values.as_ref().orNpe()[index]) }
}

fn getBiomes(this: &Rc<WorldChunkManager>, x: i32, z: i32, sizeX: i32, sizeZ: i32) -> Vec<Option<Rc<BiomeGenBase>>>
{
    let biomes: Vec<Option<Rc<BiomeGenBase>>> =
        this.loadBlockGeneratorData(this.m_biomesBuffer.take(), x, z, sizeX, sizeZ);
    this.m_biomesBuffer.set(Some(biomes.clone()));
    biomes
}

fn getTemperatures(
    this: &Rc<WorldChunkManager>,
    temperatures: Option<Vec<f64>>,
    x: i32,
    z: i32,
    sizeX: i32,
    sizeZ: i32,
) -> Vec<f64>
{
    let temperatures: Vec<f64> = match temperatures
    {
        Some(values) if values.len() as i32 >= sizeX * sizeZ => values,
        _ => vec![0.0; (sizeX * sizeZ) as usize],
    };
    let mut temperatures: Vec<f64> = this.temperatureNoise().getRegion(
        Some(temperatures),
        x as f64,
        z as f64,
        sizeX,
        sizeZ,
        0.025f32 as f64,
        0.025f32 as f64,
        0.25,
    );
    let noises: Vec<f64> = this.m_biomeNoise.as_ref().orNpe().getRegion(
        this.m_noises.take(),
        x as f64,
        z as f64,
        sizeX,
        sizeZ,
        0.25,
        0.25,
        0.5882352941176471,
    );
    let mut i: usize = 0;
    for _ in 0..sizeX
    {
        for _ in 0..sizeZ
        {
            let d0: f64 = noises[i] * 1.1 + 0.5;
            let d1: f64 = 0.01;
            let d2: f64 = 1.0 - d1;
            let mut d3: f64 = (temperatures[i] * 0.15 + 0.7) * d2 + d0 * d1;
            d3 = 1.0 - (1.0 - d3) * (1.0 - d3);
            if d3 < 0.0
            {
                d3 = 0.0;
            }
            if d3 > 1.0
            {
                d3 = 1.0;
            }
            temperatures[i] = d3;
            i += 1;
        }
    }
    this.m_noises.set(Some(noises));
    temperatures
}

fn loadBlockGeneratorData(
    this: &Rc<WorldChunkManager>,
    biomes: Option<Vec<Option<Rc<BiomeGenBase>>>>,
    x: i32,
    z: i32,
    sizeX: i32,
    sizeZ: i32,
) -> Vec<Option<Rc<BiomeGenBase>>>
{
    let mut biomes: Vec<Option<Rc<BiomeGenBase>>> = match biomes
    {
        Some(values) if values.len() as i32 >= sizeX * sizeZ => values,
        _ => vec![None; (sizeX * sizeZ) as usize],
    };
    let mut temperature: Vec<f64> = this.temperatureNoise().getRegion(
        this.m_temperature.take(),
        x as f64,
        z as f64,
        sizeX,
        sizeX,
        0.025f32 as f64,
        0.025f32 as f64,
        0.25,
    );
    let mut humidity: Vec<f64> = this.m_downfallNoise.as_ref().orNpe().getRegion(
        this.m_humidity.take(),
        x as f64,
        z as f64,
        sizeX,
        sizeX,
        0.05f32 as f64,
        0.05f32 as f64,
        0.3333333333333333,
    );
    let noises: Vec<f64> = this.m_biomeNoise.as_ref().orNpe().getRegion(
        this.m_noises.take(),
        x as f64,
        z as f64,
        sizeX,
        sizeX,
        0.25,
        0.25,
        0.5882352941176471,
    );
    let mut i: usize = 0;
    for _ in 0..sizeX
    {
        for _ in 0..sizeZ
        {
            let d0: f64 = noises[i] * 1.1 + 0.5;
            let mut d1: f64 = 0.01;
            let mut d2: f64 = 1.0 - d1;
            let mut d3: f64 = (temperature[i] * 0.15 + 0.7) * d2 + d0 * d1;
            d1 = 0.002;
            d2 = 1.0 - d1;
            let mut d4: f64 = (humidity[i] * 0.15 + 0.5) * d2 + d0 * d1;
            d3 = 1.0 - (1.0 - d3) * (1.0 - d3);
            if d3 < 0.0
            {
                d3 = 0.0;
            }
            if d4 < 0.0
            {
                d4 = 0.0;
            }
            if d3 > 1.0
            {
                d3 = 1.0;
            }
            if d4 > 1.0
            {
                d4 = 1.0;
            }
            temperature[i] = d3;
            humidity[i] = d4;
            biomes[i] = Some(BiomeGenBase::getBiomeFromLookup(d3, d4));
            i += 1;
        }
    }
    this.m_temperature.set(Some(temperature));
    this.m_humidity.set(Some(humidity));
    this.m_noises.set(Some(noises));
    biomes
}

pub const WORLD_CHUNK_MANAGER: WorldChunkManagerClass = WorldChunkManagerClass {
    m_name: "WorldChunkManager",
    m_parent: None,
    m_interfaces: &[],
    getBiomeGenAtChunkCoord: |this, pos| this.getBiomeGenAt(pos.m_chunkXPos << 4, pos.m_chunkZPos << 4),
    getBiomeGenAt: |this, x, z| this.getBiomes(x, z, 1, 1)[0].clone().orNpe(),
    getTemperature: |this, x, z| {
        let temperature: Vec<f64> = this.temperatureNoise().getRegion(
            this.m_temperature.take(),
            x as f64,
            z as f64,
            1,
            1,
            0.025f32 as f64,
            0.025f32 as f64,
            0.5,
        );
        let value: f64 = temperature[0];
        this.m_temperature.set(Some(temperature));
        value
    },
    getBiomes,
    getTemperatures,
    loadBlockGeneratorData,
};

fn loadBlockGeneratorDataHell(
    this: &Rc<WorldChunkManager>,
    biomes: Option<Vec<Option<Rc<BiomeGenBase>>>>,
    _x: i32,
    _z: i32,
    sizeX: i32,
    sizeZ: i32,
) -> Vec<Option<Rc<BiomeGenBase>>>
{
    let n: usize = (sizeX * sizeZ) as usize;
    let mut biomes: Vec<Option<Rc<BiomeGenBase>>> = match biomes
    {
        Some(values) if values.len() >= n => values,
        _ => vec![None; n],
    };
    let tooShort: bool = this.m_temperature.with(|values| values.as_ref().is_none_or(|values| values.len() < n));
    if tooShort
    {
        this.m_temperature.set(Some(vec![0.0; n]));
        this.m_humidity.set(Some(vec![0.0; n]));
    }
    let hell: &HellExt = this.hell();
    biomes[..n].fill(Some(Rc::clone(&hell.m_biome)));
    this.m_humidity.update(|values| values.as_mut().orNpe()[..n].fill(hell.m_downfall));
    this.m_temperature.update(|values| values.as_mut().orNpe()[..n].fill(hell.m_hellTemperature));
    biomes
}

pub const WORLD_CHUNK_MANAGER_HELL: WorldChunkManagerClass = WorldChunkManagerClass {
    m_name: "WorldChunkManagerHell",
    m_parent: Some(&WORLD_CHUNK_MANAGER),
    getBiomeGenAtChunkCoord: |this, _| Rc::clone(&this.hell().m_biome),
    getBiomeGenAt: |this, _, _| Rc::clone(&this.hell().m_biome),
    getTemperature: |this, _, _| this.hell().m_hellTemperature,
    getTemperatures: |this, temperatures, _, _, sizeX, sizeZ| {
        let n: usize = (sizeX * sizeZ) as usize;
        let mut temperatures: Vec<f64> = match temperatures
        {
            Some(values) if values.len() >= n => values,
            _ => vec![0.0; n],
        };
        temperatures[..n].fill(this.hell().m_hellTemperature);
        temperatures
    },
    getBiomes,
    loadBlockGeneratorData: loadBlockGeneratorDataHell,
    ..WORLD_CHUNK_MANAGER
};
