use super::{PI_F, WORLD_PROVIDER, WorldProvider, WorldProviderClass};
use crate::block::Block;
use crate::math::{Vec3D, math_helper};
use crate::world::World;
use crate::worldgen::{BiomeGenBase, ChunkProviderSky, WorldChunkManager};
use jvm::throwable::OrNpe;
use std::rc::Rc;

fn getFogColor(_this: &Rc<WorldProvider>, timeOfDay: f32, _tickDelta: f32) -> Rc<Vec3D>
{
    let i: i32 = 8421536;
    let mut f: f32 = math_helper::cos(timeOfDay * PI_F * 2.0) * 2.0 + 0.5;
    if f < 0.0
    {
        f = 0.0;
    }
    if f > 1.0
    {
        f = 1.0;
    }
    let mut f1: f32 = (i >> 16 & 0xFF) as f32 / 255.0;
    let mut f2: f32 = (i >> 8 & 0xFF) as f32 / 255.0;
    let mut f3: f32 = (i & 0xFF) as f32 / 255.0;
    f1 *= f * 0.94 + 0.06;
    f2 *= f * 0.94 + 0.06;
    f3 *= f * 0.91 + 0.09;
    Vec3D::createVector(f1 as f64, f2 as f64, f3 as f64)
}

pub const WORLD_PROVIDER_SKY: WorldProviderClass = WorldProviderClass {
    m_name: "WorldProviderSky",
    m_parent: Some(&WORLD_PROVIDER),
    registerWorldChunkManager: |this| {
        this.m_worldChunkMgr.set(Some(WorldChunkManager::newHell(BiomeGenBase::sky(), 0.5, 0.0)));
        this.m_worldType.set(1);
    },
    getChunkProvider: |this| {
        let world: Rc<World> = this.world();
        ChunkProviderSky::new(&world, world.getRandomSeed())
    },
    calculateCelestialAngle: |_, _, _| 0.0,
    calcSunriseSunsetColors: |_, _, _| None,
    getFogColor,
    hasGround: |_| false,
    getCloudHeight: |_| 8.0,
    canCoordinateBeSpawn: |this, x, z| {
        let n: i32 = this.world().getFirstUncoveredBlock(x, z);
        if n == 0
        {
            return false;
        }
        Block::byId(n).orNpe().m_blockMaterial.getIsSolid()
    },
    ..WORLD_PROVIDER
};
