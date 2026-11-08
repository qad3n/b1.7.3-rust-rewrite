use crate::worldgen::{BIOME_GEN_BASE, BiomeGenBase, BiomeGenBaseClass, SpawnListEntry};
use std::rc::Rc;

pub fn create() -> Rc<BiomeGenBase>
{
    let biome: Rc<BiomeGenBase> = BiomeGenBase::alloc(&BIOME_GEN_HELL);
    biome.m_spawnableMonsterList.clear();
    biome.m_spawnableCreatureList.clear();
    biome.m_spawnableWaterCreatureList.clear();
    biome.m_spawnableMonsterList.add(SpawnListEntry::new("EntityGhast", 10));
    biome.m_spawnableMonsterList.add(SpawnListEntry::new("EntityPigZombie", 10));
    biome
}

pub const BIOME_GEN_HELL: BiomeGenBaseClass =
    BiomeGenBaseClass { m_name: "BiomeGenHell", m_parent: Some(&BIOME_GEN_BASE), ..BIOME_GEN_BASE };
