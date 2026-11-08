use crate::worldgen::{BIOME_GEN_BASE, BiomeGenBase, BiomeGenBaseClass, SpawnListEntry};
use std::rc::Rc;

pub fn create() -> Rc<BiomeGenBase>
{
    let biome: Rc<BiomeGenBase> = BiomeGenBase::alloc(&BIOME_GEN_SKY);
    biome.m_spawnableMonsterList.clear();
    biome.m_spawnableCreatureList.clear();
    biome.m_spawnableWaterCreatureList.clear();
    biome.m_spawnableCreatureList.add(SpawnListEntry::new("EntityChicken", 10));
    biome
}

pub const BIOME_GEN_SKY: BiomeGenBaseClass = BiomeGenBaseClass {
    m_name: "BiomeGenSky",
    m_parent: Some(&BIOME_GEN_BASE),
    getSkyColorByTemp: |_, _| 0xC0C0FF,
    ..BIOME_GEN_BASE
};
