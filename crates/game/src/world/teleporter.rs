use crate::block::Block;
use crate::entity::Entity;
use crate::math::math_helper;
use crate::world::World;
use jvm::random::Random;
use std::rc::Rc;

pub struct Teleporter
{
    m_random: Random,
}

impl Teleporter
{
    pub fn new() -> Teleporter { Teleporter { m_random: Random::new() } }

    pub fn onDimensionChanged(&self, world: &Rc<World>, entity: &Rc<Entity>)
    {
        if self.findNetherPortal(world, entity)
        {
            return;
        }
        self.generateNetherPortal(world, entity);
        self.findNetherPortal(world, entity);
    }

    pub fn findNetherPortal(&self, world: &Rc<World>, entity: &Rc<Entity>) -> bool
    {
        let short1: i32 = 128;
        let portal: i32 = Block::portal().m_blockID;
        let mut d0: f64 = -1.0;
        let mut i: i32 = 0;
        let mut j: i32 = 0;
        let mut k: i32 = 0;
        let l: i32 = math_helper::floor_double(entity.posX());
        let i1: i32 = math_helper::floor_double(entity.posZ());
        for j1 in l - short1..=l + short1
        {
            let d1: f64 = j1 as f64 + 0.5 - entity.posX();
            for l1 in i1 - short1..=i1 + short1
            {
                let d2: f64 = l1 as f64 + 0.5 - entity.posZ();
                let mut i2: i32 = 127;
                while i2 >= 0
                {
                    if world.getBlockId(j1, i2, l1) == portal
                    {
                        while world.getBlockId(j1, i2 - 1, l1) == portal
                        {
                            i2 -= 1;
                        }
                        let d4: f64 = i2 as f64 + 0.5 - entity.posY();
                        let d5: f64 = d1 * d1 + d4 * d4 + d2 * d2;
                        if d0 < 0.0 || d5 < d0
                        {
                            d0 = d5;
                            i = j1;
                            j = i2;
                            k = l1;
                        }
                    }
                    i2 -= 1;
                }
            }
        }
        if d0 < 0.0
        {
            return false;
        }
        let j2: i32 = i;
        let k2: i32 = j;
        let k1: i32 = k;
        let mut d6: f64 = j2 as f64 + 0.5;
        let d3: f64 = k2 as f64 + 0.5;
        let mut d7: f64 = k1 as f64 + 0.5;
        if world.getBlockId(j2 - 1, k2, k1) == portal
        {
            d6 -= 0.5;
        }
        if world.getBlockId(j2 + 1, k2, k1) == portal
        {
            d6 += 0.5;
        }
        if world.getBlockId(j2, k2, k1 - 1) == portal
        {
            d7 -= 0.5;
        }
        if world.getBlockId(j2, k2, k1 + 1) == portal
        {
            d7 += 0.5;
        }
        entity.setLocationAndAngles(d6, d3, d7, entity.m_rotationYaw.get(), 0.0);
        entity.m_motionX.set(0.0);
        entity.m_motionY.set(0.0);
        entity.m_motionZ.set(0.0);
        true
    }

    fn fitsWide(world: &World, i2: i32, i3: i32, k2: i32, k3: i32, l3: i32) -> bool
    {
        for i4 in 0..3
        {
            for j4 in 0..4
            {
                for k4 in -1..4
                {
                    let l4: i32 = i2 + (j4 - 1) * k3 + i4 * l3;
                    let i5: i32 = i3 + k4;
                    let j5: i32 = k2 + (j4 - 1) * l3 - i4 * k3;
                    if k4 < 0 && !world.getBlockMaterial(l4, i5, j5).isSolid()
                        || k4 >= 0 && !world.isAirBlock(l4, i5, j5)
                    {
                        return false;
                    }
                }
            }
        }
        true
    }

    fn fitsNarrow(world: &World, k5: i32, i7: i32, j6: i32, l8: i32, l9: i32) -> bool
    {
        for l10 in 0..4
        {
            for l11 in -1..4
            {
                let l12: i32 = k5 + (l10 - 1) * l8;
                let i13: i32 = i7 + l11;
                let j13: i32 = j6 + (l10 - 1) * l9;
                if l11 < 0 && !world.getBlockMaterial(l12, i13, j13).isSolid()
                    || l11 >= 0 && !world.isAirBlock(l12, i13, j13)
                {
                    return false;
                }
            }
        }
        true
    }

    pub fn generateNetherPortal(&self, world: &Rc<World>, entity: &Rc<Entity>) -> bool
    {
        let b0: i32 = 16;
        let mut d0: f64 = -1.0;
        let i: i32 = math_helper::floor_double(entity.posX());
        let j: i32 = math_helper::floor_double(entity.posY());
        let k: i32 = math_helper::floor_double(entity.posZ());
        let mut l: i32 = i;
        let mut i1: i32 = j;
        let mut j1: i32 = k;
        let mut k1: i32 = 0;
        let l1: i32 = self.m_random.nextIntBounded(4);
        for i2 in i - b0..=i + b0
        {
            let d1: f64 = i2 as f64 + 0.5 - entity.posX();
            for k2 in k - b0..=k + b0
            {
                let d2: f64 = k2 as f64 + 0.5 - entity.posZ();
                let mut i3: i32 = 127;
                'label296: while i3 >= 0
                {
                    if world.isAirBlock(i2, i3, k2)
                    {
                        while i3 > 0 && world.isAirBlock(i2, i3 - 1, k2)
                        {
                            i3 -= 1;
                        }
                        for j3 in l1..l1 + 4
                        {
                            let mut k3: i32 = j3 % 2;
                            let mut l3: i32 = 1 - k3;
                            if j3 % 4 >= 2
                            {
                                k3 = -k3;
                                l3 = -l3;
                            }
                            if !Teleporter::fitsWide(world, i2, i3, k2, k3, l3)
                            {
                                i3 -= 1;
                                continue 'label296;
                            }
                            let d5: f64 = i3 as f64 + 0.5 - entity.posY();
                            let d7: f64 = d1 * d1 + d5 * d5 + d2 * d2;
                            if d0 < 0.0 || d7 < d0
                            {
                                d0 = d7;
                                l = i2;
                                i1 = i3;
                                j1 = k2;
                                k1 = j3 % 4;
                            }
                        }
                    }
                    i3 -= 1;
                }
            }
        }
        if d0 < 0.0
        {
            for k5 in i - b0..=i + b0
            {
                let d3: f64 = k5 as f64 + 0.5 - entity.posX();
                for j6 in k - b0..=k + b0
                {
                    let d4: f64 = j6 as f64 + 0.5 - entity.posZ();
                    let mut i7: i32 = 127;
                    'label233: while i7 >= 0
                    {
                        if world.isAirBlock(k5, i7, j6)
                        {
                            while world.isAirBlock(k5, i7.wrapping_sub(1), j6)
                            {
                                i7 = i7.wrapping_sub(1);
                            }
                            for l7 in l1..l1 + 2
                            {
                                let l8: i32 = l7 % 2;
                                let l9: i32 = 1 - l8;
                                if !Teleporter::fitsNarrow(world, k5, i7, j6, l8, l9)
                                {
                                    i7 -= 1;
                                    continue 'label233;
                                }
                                let d6: f64 = i7 as f64 + 0.5 - entity.posY();
                                let d8: f64 = d3 * d3 + d6 * d6 + d4 * d4;
                                if d0 < 0.0 || d8 < d0
                                {
                                    d0 = d8;
                                    l = k5;
                                    i1 = i7;
                                    j1 = j6;
                                    k1 = l7 % 2;
                                }
                            }
                        }
                        i7 -= 1;
                    }
                }
            }
        }
        let l5: i32 = k1;
        let i6: i32 = l;
        let mut j2: i32 = i1;
        let k6: i32 = j1;
        let mut l6: i32 = l5 % 2;
        let mut l2: i32 = 1 - l6;
        if l5 % 4 >= 2
        {
            l6 = -l6;
            l2 = -l2;
        }
        if d0 < 0.0
        {
            i1 = i1.clamp(70, 118);
            j2 = i1;
            for j7 in -1..=1
            {
                for i8 in 1..3
                {
                    for i9 in -1..3
                    {
                        let i10: i32 = i6 + (i8 - 1) * l6 + j7 * l2;
                        let i11: i32 = j2 + i9;
                        let i12: i32 = k6 + (i8 - 1) * l2 - j7 * l6;
                        let flag: bool = i9 < 0;
                        world.setBlockWithNotify(i10, i11, i12, if flag { Block::obsidian().m_blockID } else { 0 });
                    }
                }
            }
        }
        for _ in 0..4
        {
            world.m_editingBlocks.set(true);
            for j8 in 0..4
            {
                for j9 in -1..4
                {
                    let j10: i32 = i6 + (j8 - 1) * l6;
                    let j11: i32 = j2 + j9;
                    let j12: i32 = k6 + (j8 - 1) * l2;
                    let flag1: bool = j8 == 0 || j8 == 3 || j9 == -1 || j9 == 3;
                    let block: i32 = if flag1
                    {
                        Block::obsidian().m_blockID
                    }
                    else
                    {
                        Block::portal().m_blockID
                    };
                    world.setBlockWithNotify(j10, j11, j12, block);
                }
            }
            world.m_editingBlocks.set(false);
            for k8 in 0..4
            {
                for k9 in -1..4
                {
                    let k10: i32 = i6 + (k8 - 1) * l6;
                    let k11: i32 = j2 + k9;
                    let k12: i32 = k6 + (k8 - 1) * l2;
                    world.notifyBlocksOfNeighborChange(k10, k11, k12, world.getBlockId(k10, k11, k12));
                }
            }
        }
        true
    }
}
