use crate::world::World;
use crate::worldgen::WorldGenerator;
use crate::worldgen::world_gen_trees::generateTree;
use jvm::random::Random;
use std::rc::Rc;

pub struct WorldGenForest;

impl WorldGenForest
{
    pub fn new() -> WorldGenForest { WorldGenForest }
}

impl WorldGenerator for WorldGenForest
{
    fn generate(&self, world: &Rc<World>, random: &Random, x: i32, y: i32, z: i32) -> bool
    {
        generateTree(world, random, x, y, z, 5, Some(2))
    }
}
