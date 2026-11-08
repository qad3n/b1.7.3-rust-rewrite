use crate::block::Block;
use crate::entity::Entity;
use crate::math::math_helper::{floor_double, sqrt_double};
use crate::math::{AxisAlignedBB, Vec3D};
use crate::world::World;
use common::world::ChunkPosition;
use jvm::array_list::JArrayList;
use jvm::field::Field;
use jvm::hash_map::JHashSet;
use jvm::random::Random;
use jvm::throwable::OrNpe;
use std::cell::Cell;
use std::rc::Rc;

pub struct Explosion
{
    pub m_isFlaming: Cell<bool>,
    m_explosionRNG: Random,
    m_worldObj: Rc<World>,
    pub m_explosionX: Cell<f64>,
    pub m_explosionY: Cell<f64>,
    pub m_explosionZ: Cell<f64>,
    pub m_exploder: Option<Rc<Entity>>,
    pub m_explosionSize: Cell<f32>,
    pub m_destroyedBlockPositions: Field<JHashSet<ChunkPosition>>,
}

impl Explosion
{
    pub fn new(world: &Rc<World>, source: Option<&Rc<Entity>>, x: f64, y: f64, z: f64, power: f32) -> Rc<Explosion>
    {
        Rc::new(Explosion {
            m_isFlaming: Cell::new(false),
            m_explosionRNG: Random::new(),
            m_worldObj: Rc::clone(world),
            m_explosionX: Cell::new(x),
            m_explosionY: Cell::new(y),
            m_explosionZ: Cell::new(z),
            m_exploder: source.cloned(),
            m_explosionSize: Cell::new(power),
            m_destroyedBlockPositions: Field::new(JHashSet::new()),
        })
    }

    fn destroyedBlockList(&self) -> JArrayList<ChunkPosition>
    {
        let arraylist: JArrayList<ChunkPosition> = JArrayList::new();
        arraylist.addAll(&self.m_destroyedBlockPositions.with(JHashSet::items));
        arraylist
    }

    pub fn doExplosionA(&self)
    {
        let world: &Rc<World> = &self.m_worldObj;
        let f: f32 = self.m_explosionSize.get();
        let b0: i32 = 16;
        for i in 0..b0
        {
            for j in 0..b0
            {
                for k in 0..b0
                {
                    if !(i == 0 || i == b0 - 1 || j == 0 || j == b0 - 1 || k == 0 || k == b0 - 1)
                    {
                        continue;
                    }
                    let mut d0: f64 = (i as f32 / (b0 as f32 - 1.0) * 2.0 - 1.0) as f64;
                    let mut d1: f64 = (j as f32 / (b0 as f32 - 1.0) * 2.0 - 1.0) as f64;
                    let mut d2: f64 = (k as f32 / (b0 as f32 - 1.0) * 2.0 - 1.0) as f64;
                    let d3: f64 = jvm::math::sqrt(d0 * d0 + d1 * d1 + d2 * d2);
                    d0 /= d3;
                    d1 /= d3;
                    d2 /= d3;
                    let mut f1: f32 = self.m_explosionSize.get() * (0.7 + world.m_rand.nextFloat() * 0.6);
                    let mut d5: f64 = self.m_explosionX.get();
                    let mut d6: f64 = self.m_explosionY.get();
                    let mut d7: f64 = self.m_explosionZ.get();
                    let f2: f32 = 0.3;
                    while f1 > 0.0
                    {
                        let l1: i32 = floor_double(d5);
                        let i2: i32 = floor_double(d6);
                        let j2: i32 = floor_double(d7);
                        let k2: i32 = world.getBlockId(l1, i2, j2);
                        if k2 > 0
                        {
                            f1 -= (Block::byId(k2).orNpe().getExplosionResistance(self.m_exploder.as_ref()) + 0.3) * f2;
                        }
                        if f1 > 0.0
                        {
                            self.m_destroyedBlockPositions.update(|set| set.add(ChunkPosition::new(l1, i2, j2)));
                        }
                        d5 += d0 * f2 as f64;
                        d6 += d1 * f2 as f64;
                        d7 += d2 * f2 as f64;
                        f1 -= f2 * 0.75;
                    }
                }
            }
        }

        self.m_explosionSize.set(self.m_explosionSize.get() * 2.0);
        let size: f64 = self.m_explosionSize.get() as f64;
        let l2: i32 = floor_double(self.m_explosionX.get() - size - 1.0);
        let i3: i32 = floor_double(self.m_explosionX.get() + size + 1.0);
        let j3: i32 = floor_double(self.m_explosionY.get() - size - 1.0);
        let k3: i32 = floor_double(self.m_explosionY.get() + size + 1.0);
        let l: i32 = floor_double(self.m_explosionZ.get() - size - 1.0);
        let l3: i32 = floor_double(self.m_explosionZ.get() + size + 1.0);
        let list: Rc<JArrayList<Rc<Entity>>> = world.getEntitiesWithinAABBExcludingEntity(
            self.m_exploder.as_ref(),
            &AxisAlignedBB::getBoundingBoxFromPool(l2 as f64, j3 as f64, l as f64, i3 as f64, k3 as f64, l3 as f64),
        );
        let vec3d: Rc<Vec3D> =
            Vec3D::createVector(self.m_explosionX.get(), self.m_explosionY.get(), self.m_explosionZ.get());
        let mut i1: i32 = 0;
        while i1 < list.size()
        {
            let entity: Rc<Entity> = list.get(i1);
            i1 += 1;
            let d4: f64 = entity.getDistance(self.m_explosionX.get(), self.m_explosionY.get(), self.m_explosionZ.get())
                / self.m_explosionSize.get() as f64;
            if d4 > 1.0
            {
                continue;
            }
            let mut d9: f64 = entity.posX() - self.m_explosionX.get();
            let mut d10: f64 = entity.posY() - self.m_explosionY.get();
            let mut d11: f64 = entity.posZ() - self.m_explosionZ.get();
            let d12: f64 = sqrt_double(d9 * d9 + d10 * d10 + d11 * d11) as f64;
            d9 /= d12;
            d10 /= d12;
            d11 /= d12;
            let d13: f64 = world.getBlockDensity(&vec3d, &entity.m_boundingBox) as f64;
            let d14: f64 = (1.0 - d4) * d13;
            entity.attackEntityFrom(
                self.m_exploder.as_ref(),
                ((d14 * d14 + d14) / 2.0 * 8.0 * self.m_explosionSize.get() as f64 + 1.0) as i32,
            );
            let d8: f64 = d14;
            entity.m_motionX.set(entity.m_motionX.get() + d9 * d8);
            entity.m_motionY.set(entity.m_motionY.get() + d10 * d8);
            entity.m_motionZ.set(entity.m_motionZ.get() + d11 * d8);
        }

        self.m_explosionSize.set(f);
        let arraylist: JArrayList<ChunkPosition> = self.destroyedBlockList();
        if self.m_isFlaming.get()
        {
            for i4 in (0..arraylist.size()).rev()
            {
                let chunkposition: ChunkPosition = arraylist.get(i4);
                let j4: i32 = chunkposition.m_x;
                let k4: i32 = chunkposition.m_y;
                let j1: i32 = chunkposition.m_z;
                let l4: i32 = world.getBlockId(j4, k4, j1);
                let k1: i32 = world.getBlockId(j4, k4 - 1, j1);
                if l4 == 0 && Block::opaqueCubeLookup(k1) && self.m_explosionRNG.nextIntBounded(3) == 0
                {
                    world.setBlockWithNotify(j4, k4, j1, Block::fire().m_blockID);
                }
            }
        }
    }

    pub fn doExplosionB(&self, createFire: bool)
    {
        let world: &Rc<World> = &self.m_worldObj;
        world.playSoundEffect(
            self.m_explosionX.get(),
            self.m_explosionY.get(),
            self.m_explosionZ.get(),
            "random.explode",
            4.0,
            (1.0 + (world.m_rand.nextFloat() - world.m_rand.nextFloat()) * 0.2) * 0.7,
        );
        let arraylist: JArrayList<ChunkPosition> = self.destroyedBlockList();
        for i in (0..arraylist.size()).rev()
        {
            let chunkposition: ChunkPosition = arraylist.get(i);
            let j: i32 = chunkposition.m_x;
            let k: i32 = chunkposition.m_y;
            let l: i32 = chunkposition.m_z;
            let i1: i32 = world.getBlockId(j, k, l);
            if createFire
            {
                let d0: f64 = (j as f32 + world.m_rand.nextFloat()) as f64;
                let d1: f64 = (k as f32 + world.m_rand.nextFloat()) as f64;
                let d2: f64 = (l as f32 + world.m_rand.nextFloat()) as f64;
                let mut d3: f64 = d0 - self.m_explosionX.get();
                let mut d4: f64 = d1 - self.m_explosionY.get();
                let mut d5: f64 = d2 - self.m_explosionZ.get();
                let d6: f64 = sqrt_double(d3 * d3 + d4 * d4 + d5 * d5) as f64;
                d3 /= d6;
                d4 /= d6;
                d5 /= d6;
                let mut d7: f64 = 0.5 / (d6 / self.m_explosionSize.get() as f64 + 0.1);
                d7 *= (world.m_rand.nextFloat() * world.m_rand.nextFloat() + 0.3) as f64;
                d3 *= d7;
                d4 *= d7;
                d5 *= d7;
                world.spawnParticle(
                    "explode",
                    (d0 + self.m_explosionX.get() * 1.0) / 2.0,
                    (d1 + self.m_explosionY.get() * 1.0) / 2.0,
                    (d2 + self.m_explosionZ.get() * 1.0) / 2.0,
                    d3,
                    d4,
                    d5,
                );
                world.spawnParticle("smoke", d0, d1, d2, d3, d4, d5);
            }
            if i1 > 0
            {
                Block::byId(i1).orNpe().dropBlockAsItemWithChance(world, j, k, l, world.getBlockMetadata(j, k, l), 0.3);
                world.setBlockWithNotify(j, k, l, 0);
                Block::byId(i1).orNpe().onBlockDestroyedByExplosion(world, j, k, l);
            }
        }
    }
}
