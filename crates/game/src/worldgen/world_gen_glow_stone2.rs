use crate::world::World;
use crate::worldgen::WorldGenerator;
use crate::worldgen::world_gen_glow_stone1::generateGlowStone;
use jvm::random::Random;
use std::rc::Rc;

pub struct WorldGenGlowStone2;

impl WorldGenerator for WorldGenGlowStone2
{
    fn generate(&self, world: &Rc<World>, random: &Random, x: i32, y: i32, z: i32) -> bool
    {
        generateGlowStone(world, random, x, y, z)
    }
}
