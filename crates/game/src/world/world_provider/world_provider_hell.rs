use super::{WORLD_PROVIDER, WorldProvider, WorldProviderClass};
use crate::block::Block;
use crate::math::Vec3D;
use crate::world::World;
use crate::worldgen::{BiomeGenBase, ChunkProviderHell, WorldChunkManager};
use std::rc::Rc;

fn generateLightBrightnessTable(this: &Rc<WorldProvider>)
{
    let f: f32 = 0.1;
    this.m_lightBrightnessTable.update(|table| {
        for i in 0..=15
        {
            let f2: f32 = 1.0 - i as f32 / 15.0;
            table[i] = (1.0 - f2) / (f2 * 3.0 + 1.0) * (1.0 - f) + f;
        }
    });
}

fn canCoordinateBeSpawn(this: &Rc<WorldProvider>, x: i32, z: i32) -> bool
{
    let n: i32 = this.world().getFirstUncoveredBlock(x, z);
    if n == Block::bedrock().m_blockID || n == 0
    {
        return false;
    }
    Block::opaqueCubeLookup(n)
}

pub const WORLD_PROVIDER_HELL: WorldProviderClass = WorldProviderClass {
    m_name: "WorldProviderHell",
    m_parent: Some(&WORLD_PROVIDER),
    registerWorldChunkManager: |this| {
        this.m_worldChunkMgr.set(Some(WorldChunkManager::newHell(BiomeGenBase::hell(), 1.0, 0.0)));
        this.m_isNether.set(true);
        this.m_isHellWorld.set(true);
        this.m_hasNoSky.set(true);
        this.m_worldType.set(-1);
    },
    getFogColor: |_, _, _| Vec3D::createVector(0.2f32 as f64, 0.03f32 as f64, 0.03f32 as f64),
    generateLightBrightnessTable,
    getChunkProvider: |this| {
        let world: Rc<World> = this.world();
        ChunkProviderHell::new(&world, world.getRandomSeed())
    },
    canCoordinateBeSpawn,
    calculateCelestialAngle: |_, _, _| 0.5,
    canRespawnHere: |_| false,
    ..WORLD_PROVIDER
};
