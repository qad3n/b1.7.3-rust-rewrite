use crate::worldgen::{
    BIOME_GEN_BASE, BiomeGenBase, BiomeGenBaseClass, SpawnListEntry, WorldGenBigTree, WorldGenForest, WorldGenTrees,
    WorldGenerator,
};
use jvm::random::Random;
use std::rc::Rc;

pub fn create() -> Rc<BiomeGenBase>
{
    let biome: Rc<BiomeGenBase> = BiomeGenBase::alloc(&BIOME_GEN_FOREST);
    biome.m_spawnableCreatureList.add(SpawnListEntry::new("EntityWolf", 2));
    biome
}

fn getRandomWorldGenForTrees(_this: &Rc<BiomeGenBase>, random: &Random) -> Box<dyn WorldGenerator>
{
    if random.nextIntBounded(5) == 0
    {
        return Box::new(WorldGenForest::new());
    }
    if random.nextIntBounded(3) == 0
    {
        return Box::new(WorldGenBigTree::new());
    }
    Box::new(WorldGenTrees::new())
}

pub const BIOME_GEN_FOREST: BiomeGenBaseClass = BiomeGenBaseClass {
    m_name: "BiomeGenForest",
    m_parent: Some(&BIOME_GEN_BASE),
    getRandomWorldGenForTrees,
    ..BIOME_GEN_BASE
};
