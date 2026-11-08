use crate::worldgen::{
    BIOME_GEN_BASE, BiomeGenBase, BiomeGenBaseClass, SpawnListEntry, WorldGenTaiga1, WorldGenTaiga2, WorldGenerator,
};
use jvm::random::Random;
use std::rc::Rc;

pub fn create() -> Rc<BiomeGenBase>
{
    let biome: Rc<BiomeGenBase> = BiomeGenBase::alloc(&BIOME_GEN_TAIGA);
    biome.m_spawnableCreatureList.add(SpawnListEntry::new("EntityWolf", 2));
    biome
}

fn getRandomWorldGenForTrees(_this: &Rc<BiomeGenBase>, random: &Random) -> Box<dyn WorldGenerator>
{
    if random.nextIntBounded(3) == 0
    {
        return Box::new(WorldGenTaiga1::new());
    }
    Box::new(WorldGenTaiga2::new())
}

pub const BIOME_GEN_TAIGA: BiomeGenBaseClass = BiomeGenBaseClass {
    m_name: "BiomeGenTaiga",
    m_parent: Some(&BIOME_GEN_BASE),
    getRandomWorldGenForTrees,
    ..BIOME_GEN_BASE
};
