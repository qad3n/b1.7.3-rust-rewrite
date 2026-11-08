mod world_provider_hell;
mod world_provider_sky;
mod world_provider_surface;

use crate::block::Block;
use crate::math::{Vec3D, math_helper};
use crate::world::{IChunkProvider, World};
use crate::worldgen::{ChunkProviderGenerate, WorldChunkManager};
use jvm::field::Field;
use jvm::throwable::OrNpe;
use std::cell::Cell;
use std::rc::Rc;
pub use world_provider_hell::WORLD_PROVIDER_HELL;
pub use world_provider_sky::WORLD_PROVIDER_SKY;
pub use world_provider_surface::WORLD_PROVIDER_SURFACE;

const PI_F: f32 = std::f32::consts::PI;

crate::vtable! {
    WorldProviderClass for WorldProvider {
        fn generateLightBrightnessTable();
        fn registerWorldChunkManager();
        fn getChunkProvider() -> Rc<dyn IChunkProvider>;
        fn canCoordinateBeSpawn(x: i32, z: i32) -> bool;
        fn calculateCelestialAngle(time: i64, tickDelta: f32) -> f32;
        fn calcSunriseSunsetColors(timeOfDay: f32, tickDelta: f32) -> Option<[f32; 4]>;
        fn getFogColor(timeOfDay: f32, tickDelta: f32) -> Rc<Vec3D>;
        fn canRespawnHere() -> bool;
        fn getCloudHeight() -> f32;
        fn hasGround() -> bool;
    }
}

pub struct WorldProvider
{
    pub m_class: &'static WorldProviderClass,
    pub m_worldObj: Field<Option<Rc<World>>>,
    pub m_worldChunkMgr: Field<Option<Rc<WorldChunkManager>>>,
    pub m_isNether: Cell<bool>,
    pub m_isHellWorld: Cell<bool>,
    pub m_hasNoSky: Cell<bool>,
    pub m_lightBrightnessTable: Field<[f32; 16]>,
    pub m_worldType: Cell<i32>,
    m_colorsSunriseSunset: Field<[f32; 4]>,
}

impl WorldProvider
{
    pub fn alloc(class: &'static WorldProviderClass) -> Rc<Self>
    {
        Rc::new(Self {
            m_class: class,
            m_worldObj: Field::new(None),
            m_worldChunkMgr: Field::new(None),
            m_isNether: Cell::new(false),
            m_isHellWorld: Cell::new(false),
            m_hasNoSky: Cell::new(false),
            m_lightBrightnessTable: Field::new([0.0; 16]),
            m_worldType: Cell::new(0),
            m_colorsSunriseSunset: Field::new([0.0; 4]),
        })
    }

    pub fn world(&self) -> Rc<World> { self.m_worldObj.get().orNpe() }

    pub fn brightness(&self, light: i32) -> f32 { self.m_lightBrightnessTable.with(|table| table[light as usize]) }

    pub fn registerWorld(self: &Rc<Self>, world: &Rc<World>)
    {
        self.m_worldObj.set(Some(Rc::clone(world)));
        self.registerWorldChunkManager();
        self.generateLightBrightnessTable();
    }

    pub fn getProviderForDimension(id: i32) -> Rc<WorldProvider>
    {
        match id
        {
            -1 => Self::alloc(&WORLD_PROVIDER_HELL),
            0 => Self::alloc(&WORLD_PROVIDER_SURFACE),
            1 => Self::alloc(&WORLD_PROVIDER_SKY),
            _ => jvm::throwable::npe(),
        }
    }
}

fn generateLightBrightnessTable(this: &Rc<WorldProvider>)
{
    let f: f32 = 0.05;
    this.m_lightBrightnessTable.update(|table| {
        for i in 0..=15
        {
            let f1: f32 = 1.0 - i as f32 / 15.0;
            table[i] = (1.0 - f1) / (f1 * 3.0 + 1.0) * (1.0 - f) + f;
        }
    });
}

fn calculateCelestialAngle(_this: &Rc<WorldProvider>, time: i64, tickDelta: f32) -> f32
{
    let n: i32 = (time % 24000) as i32;
    let mut f: f32 = (n as f32 + tickDelta) / 24000.0 - 0.25;
    if f < 0.0
    {
        f += 1.0;
    }
    if f > 1.0
    {
        f -= 1.0;
    }
    let f2: f32 = f;
    f = 1.0 - ((jvm::math::cos(f as f64 * jvm::math::PI) + 1.0) / 2.0) as f32;
    f = f2 + (f - f2) / 3.0;
    f
}

fn calcSunriseSunsetColors(this: &Rc<WorldProvider>, timeOfDay: f32, _tickDelta: f32) -> Option<[f32; 4]>
{
    let f: f32 = 0.4;
    let f1: f32 = math_helper::cos(timeOfDay * PI_F * 2.0) - 0.0;
    let f2: f32 = -0.0;
    if f1 >= f2 - f && f1 <= f2 + f
    {
        let f3: f32 = (f1 - f2) / f * 0.5 + 0.5;
        let mut f4: f32 = 1.0 - (1.0 - math_helper::sin(f3 * PI_F)) * 0.99;
        f4 *= f4;
        let colors: [f32; 4] = [f3 * 0.3 + 0.7, f3 * f3 * 0.7 + 0.2, f3 * f3 * 0.0 + 0.2, f4];
        this.m_colorsSunriseSunset.set(colors);
        return Some(colors);
    }
    None
}

fn getFogColor(_this: &Rc<WorldProvider>, timeOfDay: f32, _tickDelta: f32) -> Rc<Vec3D>
{
    let mut f: f32 = math_helper::cos(timeOfDay * PI_F * 2.0) * 2.0 + 0.5;
    if f < 0.0
    {
        f = 0.0;
    }
    if f > 1.0
    {
        f = 1.0;
    }
    let mut f1: f32 = 0.7529412;
    let mut f2: f32 = 0.84705883;
    let mut f3: f32 = 1.0;
    f1 *= f * 0.94 + 0.06;
    f2 *= f * 0.94 + 0.06;
    f3 *= f * 0.91 + 0.09;
    Vec3D::createVector(f1 as f64, f2 as f64, f3 as f64)
}

pub const WORLD_PROVIDER: WorldProviderClass = WorldProviderClass {
    m_name: "WorldProvider",
    m_parent: None,
    m_interfaces: &[],
    generateLightBrightnessTable,
    registerWorldChunkManager: |this| this.m_worldChunkMgr.set(Some(WorldChunkManager::new(&this.world()))),
    getChunkProvider: |this| {
        let world: Rc<World> = this.world();
        ChunkProviderGenerate::new(&world, world.getRandomSeed())
    },
    canCoordinateBeSpawn: |this, x, z| this.world().getFirstUncoveredBlock(x, z) == Block::sand().m_blockID,
    calculateCelestialAngle,
    calcSunriseSunsetColors,
    getFogColor,
    canRespawnHere: |_| true,
    getCloudHeight: |_| 108.0,
    hasGround: |_| true,
};
