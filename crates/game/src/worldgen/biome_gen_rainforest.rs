use crate::worldgen::{
    BIOME_GEN_BASE, BiomeGenBase, BiomeGenBaseClass, WorldGenBigTree, WorldGenTrees, WorldGenerator,
};
use jvm::random::Random;
use std::rc::Rc;

fn getRandomWorldGenForTrees(_this: &Rc<BiomeGenBase>, random: &Random) -> Box<dyn WorldGenerator>
{
    if random.nextIntBounded(3) == 0
    {
        return Box::new(WorldGenBigTree::new());
    }
    Box::new(WorldGenTrees::new())
}

pub const BIOME_GEN_RAINFOREST: BiomeGenBaseClass = BiomeGenBaseClass {
    m_name: "BiomeGenRainforest",
    m_parent: Some(&BIOME_GEN_BASE),
    getRandomWorldGenForTrees,
    ..BIOME_GEN_BASE
};
