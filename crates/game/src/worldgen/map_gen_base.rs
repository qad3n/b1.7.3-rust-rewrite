use crate::world::{IChunkProvider, World};
use jvm::random::Random;
use std::cell::Cell;
use std::rc::Rc;

crate::vtable! {
    MapGenBaseClass for MapGenBase {
        fn recursiveGenerate(world: &Rc<World>, startChunkX: i32, startChunkZ: i32, chunkX: i32, chunkZ: i32, blocks: &mut [i8]);
    }
}

pub struct MapGenBase
{
    pub m_class: &'static MapGenBaseClass,
    pub m_range: Cell<i32>,
    pub m_rand: Random,
}

impl MapGenBase
{
    pub fn alloc(class: &'static MapGenBaseClass) -> Rc<MapGenBase>
    {
        Rc::new(MapGenBase { m_class: class, m_range: Cell::new(8), m_rand: Random::new() })
    }

    pub fn place(
        self: &Rc<Self>,
        _source: &dyn IChunkProvider,
        world: &Rc<World>,
        chunkX: i32,
        chunkZ: i32,
        blocks: &mut [i8],
    )
    {
        let n: i32 = self.m_range.get();
        self.m_rand.setSeed(world.getRandomSeed());
        let l: i64 = self.m_rand.nextLong() / 2 * 2 + 1;
        let l2: i64 = self.m_rand.nextLong() / 2 * 2 + 1;
        for i in chunkX - n..=chunkX + n
        {
            for j in chunkZ - n..=chunkZ + n
            {
                self.m_rand.setSeed((i as i64 * l + j as i64 * l2) ^ world.getRandomSeed());
                self.recursiveGenerate(world, i, j, chunkX, chunkZ, blocks);
            }
        }
    }
}

pub const MAP_GEN_BASE: MapGenBaseClass = MapGenBaseClass {
    m_name: "MapGenBase",
    m_parent: None,
    m_interfaces: &[],
    recursiveGenerate: |_, _, _, _, _, _, _| {},
};
