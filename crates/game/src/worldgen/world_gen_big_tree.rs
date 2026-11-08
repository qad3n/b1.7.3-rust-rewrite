use crate::math::math_helper;
use crate::world::World;
use crate::worldgen::WorldGenerator;
use jvm::field::Field;
use jvm::math;
use jvm::random::Random;
use jvm::throwable::OrNpe;
use std::cell::Cell;
use std::rc::Rc;

const MINOR_AXES: [usize; 6] = [2, 0, 0, 1, 2, 1];

pub struct WorldGenBigTree
{
    m_random: Random,
    m_worldObj: Field<Option<Rc<World>>>,
    m_basePos: Cell<[i32; 3]>,
    m_heightLimit: Cell<i32>,
    m_height: Cell<i32>,
    m_trunkScale: Cell<f64>,
    m_branchSlope: Cell<f64>,
    m_branchLengthScale: Cell<f64>,
    m_foliageDensity: Cell<f64>,
    m_trunkWidth: Cell<i32>,
    m_maxTrunkHeight: Cell<i32>,
    m_foliageClusterHeight: Cell<i32>,
    m_branches: Field<Vec<[i32; 4]>>,
}

impl WorldGenBigTree
{
    pub fn new() -> WorldGenBigTree
    {
        WorldGenBigTree {
            m_random: Random::new(),
            m_worldObj: Field::new(None),
            m_basePos: Cell::new([0, 0, 0]),
            m_heightLimit: Cell::new(0),
            m_height: Cell::new(0),
            m_trunkScale: Cell::new(0.618),
            m_branchSlope: Cell::new(0.381),
            m_branchLengthScale: Cell::new(1.0),
            m_foliageDensity: Cell::new(1.0),
            m_trunkWidth: Cell::new(1),
            m_maxTrunkHeight: Cell::new(12),
            m_foliageClusterHeight: Cell::new(4),
            m_branches: Field::new(Vec::new()),
        }
    }

    fn world(&self) -> Rc<World> { self.m_worldObj.get().orNpe() }

    fn makeBranches(&self)
    {
        let heightLimit: i32 = self.m_heightLimit.get();
        let basePos: [i32; 3] = self.m_basePos.get();
        let fch: i32 = self.m_foliageClusterHeight.get();
        self.m_height.set((heightLimit as f64 * self.m_trunkScale.get()) as i32);
        if self.m_height.get() >= heightLimit
        {
            self.m_height.set(heightLimit - 1);
        }
        let mut i: i32 = (1.382 + math::pow(self.m_foliageDensity.get() * heightLimit as f64 / 13.0, 2.0)) as i32;
        if i < 1
        {
            i = 1;
        }
        let mut aint: Vec<[i32; 4]> = vec![[0; 4]; (i * heightLimit) as usize];
        let mut j: i32 = basePos[1] + heightLimit - fch;
        let mut k: usize = 1;
        let l: i32 = basePos[1] + self.m_height.get();
        let mut i1: i32 = j - basePos[1];
        aint[0] = [basePos[0], j, basePos[2], l];
        j -= 1;
        while i1 >= 0
        {
            let mut j1: i32 = 0;
            let f: f32 = self.getTreeShape(i1);
            if f < 0.0f32
            {
                j -= 1;
                i1 -= 1;
                continue;
            }
            let d0: f64 = 0.5;
            while j1 < i
            {
                let d1: f64 = self.m_branchLengthScale.get() * (f as f64 * (self.m_random.nextFloat() as f64 + 0.328));
                let d2: f64 = self.m_random.nextFloat() as f64 * 2.0 * 3.14159;
                let k1: i32 = math_helper::floor_double(d1 * math::sin(d2) + basePos[0] as f64 + d0);
                let l1: i32 = math_helper::floor_double(d1 * math::cos(d2) + basePos[2] as f64 + d0);
                let aint1: [i32; 3] = [k1, j, l1];
                let aint2: [i32; 3] = [k1, j + fch, l1];
                if self.tryBranch(&aint1, &aint2) == -1
                {
                    let mut aint3: [i32; 3] = basePos;
                    let d3: f64 = math::sqrt(
                        math::pow((basePos[0] - aint1[0]).abs() as f64, 2.0)
                            + math::pow((basePos[2] - aint1[2]).abs() as f64, 2.0),
                    );
                    let d4: f64 = d3 * self.m_branchSlope.get();
                    if aint1[1] as f64 - d4 > l as f64
                    {
                        aint3[1] = l;
                    }
                    else
                    {
                        aint3[1] = (aint1[1] as f64 - d4) as i32;
                    }
                    if self.tryBranch(&aint3, &aint1) == -1
                    {
                        aint[k] = [k1, j, l1, aint3[1]];
                        k += 1;
                    }
                }
                j1 += 1;
            }
            j -= 1;
            i1 -= 1;
        }
        aint.truncate(k);
        self.m_branches.set(aint);
    }

    fn placeCluster(&self, x: i32, y: i32, z: i32, shape: f32, majorAxis: usize, clusterBlock: i32)
    {
        let world: Rc<World> = self.world();
        let i: i32 = (shape as f64 + 0.618) as i32;
        let b0: usize = MINOR_AXES[majorAxis];
        let b1: usize = MINOR_AXES[majorAxis + 3];
        let aint: [i32; 3] = [x, y, z];
        let mut aint1: [i32; 3] = [0, 0, 0];
        aint1[majorAxis] = aint[majorAxis];
        for j in -i..=i
        {
            aint1[b0] = aint[b0] + j;
            for k in -i..=i
            {
                let d0: f64 = math::sqrt(math::pow(j.abs() as f64 + 0.5, 2.0) + math::pow(k.abs() as f64 + 0.5, 2.0));
                if d0 > shape as f64
                {
                    continue;
                }
                aint1[b1] = aint[b1] + k;
                let l: i32 = world.getBlockId(aint1[0], aint1[1], aint1[2]);
                if l == 0 || l == 18
                {
                    world.setBlock(aint1[0], aint1[1], aint1[2], clusterBlock);
                }
            }
        }
    }

    fn getTreeShape(&self, height: i32) -> f32
    {
        let heightLimit: i32 = self.m_heightLimit.get();
        if (height as f64) < heightLimit as f32 as f64 * 0.3
        {
            return -1.618f32;
        }
        let f: f32 = heightLimit as f32 / 2.0f32;
        let f1: f32 = heightLimit as f32 / 2.0f32 - height as f32;
        let f2: f32 = if f1 == 0.0f32
        {
            f
        }
        else if f1.abs() >= f
        {
            0.0f32
        }
        else
        {
            math::sqrt(math::pow(f.abs() as f64, 2.0) - math::pow(f1.abs() as f64, 2.0)) as f32
        };
        f2 * 0.5f32
    }

    fn getClusterShape(&self, layer: i32) -> f32
    {
        if layer < 0 || layer >= self.m_foliageClusterHeight.get()
        {
            return -1.0f32;
        }
        if layer == 0 || layer == self.m_foliageClusterHeight.get() - 1
        {
            return 2.0f32;
        }
        3.0f32
    }

    fn placeFoliageCluster(&self, x: i32, baseY: i32, z: i32)
    {
        for i in baseY..baseY + self.m_foliageClusterHeight.get()
        {
            let f: f32 = self.getClusterShape(i - baseY);
            self.placeCluster(x, i, z, f, 1, 18);
        }
    }

    fn placeBranch(&self, from: &[i32; 3], to: &[i32; 3], log: i32)
    {
        let mut aint: [i32; 3] = [0, 0, 0];
        let mut b1: usize = 0;
        for b0 in 0..3
        {
            aint[b0] = to[b0] - from[b0];
            if aint[b0].abs() > aint[b1].abs()
            {
                b1 = b0;
            }
        }
        if aint[b1] == 0
        {
            return;
        }
        let world: Rc<World> = self.world();
        let b2: usize = MINOR_AXES[b1];
        let b3: usize = MINOR_AXES[b1 + 3];
        let b4: i32 = if aint[b1] > 0 { 1 } else { -1 };
        let d0: f64 = aint[b2] as f64 / aint[b1] as f64;
        let d1: f64 = aint[b3] as f64 / aint[b1] as f64;
        let mut aint1: [i32; 3] = [0, 0, 0];
        let i: i32 = aint[b1] + b4;
        let mut b5: i32 = 0;
        while b5 != i
        {
            aint1[b1] = math_helper::floor_double((from[b1] + b5) as f64 + 0.5);
            aint1[b2] = math_helper::floor_double(from[b2] as f64 + b5 as f64 * d0 + 0.5);
            aint1[b3] = math_helper::floor_double(from[b3] as f64 + b5 as f64 * d1 + 0.5);
            world.setBlock(aint1[0], aint1[1], aint1[2], log);
            b5 += b4;
        }
    }

    fn placeFoliage(&self)
    {
        let branches: Vec<[i32; 4]> = self.m_branches.get();
        for branch in branches
        {
            self.placeFoliageCluster(branch[0], branch[1], branch[2]);
        }
    }

    fn shouldPlaceBranch(&self, height: i32) -> bool { !((height as f64) < self.m_heightLimit.get() as f64 * 0.2) }

    fn placeTrunk(&self)
    {
        let basePos: [i32; 3] = self.m_basePos.get();
        let i: i32 = basePos[0];
        let j: i32 = basePos[1];
        let k: i32 = basePos[1] + self.m_height.get();
        let l: i32 = basePos[2];
        let mut aint: [i32; 3] = [i, j, l];
        let mut aint1: [i32; 3] = [i, k, l];
        self.placeBranch(&aint, &aint1, 17);
        if self.m_trunkWidth.get() == 2
        {
            aint[0] += 1;
            aint1[0] += 1;
            self.placeBranch(&aint, &aint1, 17);
            aint[2] += 1;
            aint1[2] += 1;
            self.placeBranch(&aint, &aint1, 17);
            aint[0] += -1;
            aint1[0] += -1;
            self.placeBranch(&aint, &aint1, 17);
        }
    }

    fn placeBranches(&self)
    {
        let basePos: [i32; 3] = self.m_basePos.get();
        let mut aint: [i32; 3] = basePos;
        let branches: Vec<[i32; 4]> = self.m_branches.get();
        for aint1 in branches
        {
            let aint2: [i32; 3] = [aint1[0], aint1[1], aint1[2]];
            aint[1] = aint1[3];
            let k: i32 = aint[1] - basePos[1];
            if self.shouldPlaceBranch(k)
            {
                self.placeBranch(&aint, &aint2, 17);
            }
        }
    }

    fn tryBranch(&self, from: &[i32; 3], to: &[i32; 3]) -> i32
    {
        let mut array: [i32; 3] = [0, 0, 0];
        let mut n: usize = 0;
        for i in 0..3
        {
            array[i] = to[i] - from[i];
            if array[i].abs() > array[n].abs()
            {
                n = i;
            }
        }
        if array[n] == 0
        {
            return -1;
        }
        let world: Rc<World> = self.world();
        let b: usize = MINOR_AXES[n];
        let b2: usize = MINOR_AXES[n + 3];
        let n2: i32 = if array[n] > 0 { 1 } else { -1 };
        let n3: f64 = array[b] as f64 / array[n] as f64;
        let n4: f64 = array[b2] as f64 / array[n] as f64;
        let mut array2: [i32; 3] = [0, 0, 0];
        let n5: i32 = array[n] + n2;
        let mut j: i32 = 0;
        while j != n5
        {
            array2[n] = from[n] + j;
            array2[b] = math_helper::floor_double(from[b] as f64 + j as f64 * n3);
            array2[b2] = math_helper::floor_double(from[b2] as f64 + j as f64 * n4);
            let blockId: i32 = world.getBlockId(array2[0], array2[1], array2[2]);
            if blockId != 0 && blockId != 18
            {
                break;
            }
            j += n2;
        }
        if j == n5
        {
            return -1;
        }
        j.abs()
    }

    fn canPlace(&self) -> bool
    {
        let basePos: [i32; 3] = self.m_basePos.get();
        let aint1: [i32; 3] = [basePos[0], basePos[1] + self.m_heightLimit.get() - 1, basePos[2]];
        let i: i32 = self.world().getBlockId(basePos[0], basePos[1] - 1, basePos[2]);
        if i != 2 && i != 3
        {
            return false;
        }
        let j: i32 = self.tryBranch(&basePos, &aint1);
        if j == -1
        {
            return true;
        }
        if j < 6
        {
            return false;
        }
        self.m_heightLimit.set(j);
        true
    }
}

impl WorldGenerator for WorldGenBigTree
{
    fn prepare(&self, d0: f64, d1: f64, d2: f64)
    {
        self.m_maxTrunkHeight.set((d0 * 12.0) as i32);
        if d0 > 0.5
        {
            self.m_foliageClusterHeight.set(5);
        }
        self.m_branchLengthScale.set(d1);
        self.m_foliageDensity.set(d2);
    }

    fn generate(&self, world: &Rc<World>, random: &Random, x: i32, y: i32, z: i32) -> bool
    {
        self.m_worldObj.set(Some(Rc::clone(world)));
        let i: i64 = random.nextLong();
        self.m_random.setSeed(i);
        self.m_basePos.set([x, y, z]);
        if self.m_heightLimit.get() == 0
        {
            self.m_heightLimit.set(5 + self.m_random.nextIntBounded(self.m_maxTrunkHeight.get()));
        }
        if !self.canPlace()
        {
            return false;
        }
        self.makeBranches();
        self.placeFoliage();
        self.placeTrunk();
        self.placeBranches();
        true
    }
}
