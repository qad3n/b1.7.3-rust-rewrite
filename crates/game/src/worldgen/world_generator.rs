use crate::world::World;
use jvm::random::Random;
use std::rc::Rc;

pub trait WorldGenerator
{
    fn generate(&self, world: &Rc<World>, random: &Random, x: i32, y: i32, z: i32) -> bool;

    fn prepare(&self, _d0: f64, _d1: f64, _d2: f64) {}
}
