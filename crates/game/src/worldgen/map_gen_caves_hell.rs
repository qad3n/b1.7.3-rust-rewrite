use crate::block::Block;
use crate::math::math_helper;
use crate::world::World;
use crate::worldgen::{MAP_GEN_BASE, MapGenBase, MapGenBaseClass};
use jvm::random::Random;
use std::rc::Rc;

const PI_F: f32 = jvm::math::PI as f32;
const HALF_PI_F: f32 = (jvm::math::PI / 2.0) as f32;

pub fn create() -> Rc<MapGenBase> { MapGenBase::alloc(&MAP_GEN_CAVES_HELL) }

fn carveRoom(this: &Rc<MapGenBase>, chunkX: i32, chunkZ: i32, blocks: &mut [i8], x: f64, y: f64, z: f64)
{
    let baseWidth: f32 = 1.0f32 + this.m_rand.nextFloat() * 6.0f32;
    carveTunnel(this, chunkX, chunkZ, blocks, x, y, z, baseWidth, 0.0, 0.0, -1, -1, 0.5);
}

fn carveTunnel(
    this: &Rc<MapGenBase>,
    chunkX: i32,
    chunkZ: i32,
    blocks: &mut [i8],
    mut x: f64,
    mut y: f64,
    mut z: f64,
    baseWidth: f32,
    mut yaw: f32,
    mut pitch: f32,
    mut tunnel: i32,
    mut tunnelCount: i32,
    widthHeightRatio: f64,
)
{
    let lavaMoving: i32 = Block::lavaMoving().m_blockID;
    let lavaStill: i32 = Block::lavaStill().m_blockID;
    let netherrack: i32 = Block::netherrack().m_blockID;
    let dirt: i32 = Block::dirt().m_blockID;
    let grass: i32 = Block::grass().m_blockID;
    let d0: f64 = (chunkX * 16 + 8) as f64;
    let d1: f64 = (chunkZ * 16 + 8) as f64;
    let mut f: f32 = 0.0f32;
    let mut f1: f32 = 0.0f32;
    let random: Random = Random::withSeed(this.m_rand.nextLong());
    if tunnelCount <= 0
    {
        let i: i32 = this.m_range.get() * 16 - 16;
        tunnelCount = i - random.nextIntBounded(i / 4);
    }
    let mut flag1: bool = false;
    if tunnel == -1
    {
        tunnel = tunnelCount / 2;
        flag1 = true;
    }
    let j: i32 = random.nextIntBounded(tunnelCount / 2) + tunnelCount / 4;
    let flag: bool = random.nextIntBounded(6) == 0;
    while tunnel < tunnelCount
    {
        let d2: f64 = 1.5 + (math_helper::sin(tunnel as f32 * PI_F / tunnelCount as f32) * baseWidth * 1.0f32) as f64;
        let d3: f64 = d2 * widthHeightRatio;
        let f2: f32 = math_helper::cos(pitch);
        let f3: f32 = math_helper::sin(pitch);
        x += (math_helper::cos(yaw) * f2) as f64;
        y += f3 as f64;
        z += (math_helper::sin(yaw) * f2) as f64;
        if flag
        {
            pitch *= 0.92f32;
        }
        else
        {
            pitch *= 0.7f32;
        }
        pitch += f1 * 0.1f32;
        yaw += f * 0.1f32;
        f1 *= 0.9f32;
        f *= 0.75f32;
        f1 += (random.nextFloat() - random.nextFloat()) * random.nextFloat() * 2.0f32;
        f += (random.nextFloat() - random.nextFloat()) * random.nextFloat() * 4.0f32;
        if !flag1 && tunnel == j && baseWidth > 1.0f32
        {
            let width: f32 = random.nextFloat() * 0.5f32 + 0.5f32;
            carveTunnel(
                this,
                chunkX,
                chunkZ,
                blocks,
                x,
                y,
                z,
                width,
                yaw - HALF_PI_F,
                pitch / 3.0f32,
                tunnel,
                tunnelCount,
                1.0,
            );
            let width: f32 = random.nextFloat() * 0.5f32 + 0.5f32;
            carveTunnel(
                this,
                chunkX,
                chunkZ,
                blocks,
                x,
                y,
                z,
                width,
                yaw + HALF_PI_F,
                pitch / 3.0f32,
                tunnel,
                tunnelCount,
                1.0,
            );
            return;
        }
        if flag1 || random.nextIntBounded(4) != 0
        {
            let d4: f64 = x - d0;
            let d5: f64 = z - d1;
            let d6: f64 = (tunnelCount - tunnel) as f64;
            let d7: f64 = (baseWidth + 2.0f32 + 16.0f32) as f64;
            if d4 * d4 + d5 * d5 - d6 * d6 > d7 * d7
            {
                return;
            }
            if !(x < d0 - 16.0 - d2 * 2.0)
                && !(z < d1 - 16.0 - d2 * 2.0)
                && !(x > d0 + 16.0 + d2 * 2.0)
                && !(z > d1 + 16.0 + d2 * 2.0)
            {
                let mut l2: i32 = math_helper::floor_double(x - d2) - chunkX * 16 - 1;
                let mut k: i32 = math_helper::floor_double(x + d2) - chunkX * 16 + 1;
                let mut i3: i32 = math_helper::floor_double(y - d3) - 1;
                let mut l: i32 = math_helper::floor_double(y + d3) + 1;
                let mut j3: i32 = math_helper::floor_double(z - d2) - chunkZ * 16 - 1;
                let mut i1: i32 = math_helper::floor_double(z + d2) - chunkZ * 16 + 1;
                if l2 < 0
                {
                    l2 = 0;
                }
                if k > 16
                {
                    k = 16;
                }
                if i3 < 1
                {
                    i3 = 1;
                }
                if l > 120
                {
                    l = 120;
                }
                if j3 < 0
                {
                    j3 = 0;
                }
                if i1 > 16
                {
                    i1 = 16;
                }
                let mut flag2: bool = false;
                let mut j1: i32 = l2;
                while !flag2 && j1 < k
                {
                    let mut k1: i32 = j3;
                    while !flag2 && k1 < i1
                    {
                        let mut l1: i32 = l + 1;
                        while !flag2 && l1 >= i3 - 1
                        {
                            let i2: i32 = (j1 * 16 + k1) * 128 + l1;
                            if (0..128).contains(&l1)
                            {
                                let block: i32 = blocks[i2 as usize] as i32;
                                if block == lavaMoving || block == lavaStill
                                {
                                    flag2 = true;
                                }
                                if l1 != i3 - 1 && j1 != l2 && j1 != k - 1 && k1 != j3 && k1 != i1 - 1
                                {
                                    l1 = i3;
                                }
                            }
                            l1 -= 1;
                        }
                        k1 += 1;
                    }
                    j1 += 1;
                }
                if !flag2
                {
                    for k3 in l2..k
                    {
                        let d10: f64 = ((k3 + chunkX * 16) as f64 + 0.5 - x) / d2;
                        for l3 in j3..i1
                        {
                            let d8: f64 = ((l3 + chunkZ * 16) as f64 + 0.5 - z) / d2;
                            let mut j2: i32 = (k3 * 16 + l3) * 128 + l;
                            for k2 in (i3..l).rev()
                            {
                                let d9: f64 = (k2 as f64 + 0.5 - y) / d3;
                                if d9 > -0.7 && d10 * d10 + d9 * d9 + d8 * d8 < 1.0
                                {
                                    let b0: i32 = blocks[j2 as usize] as i32;
                                    if b0 == netherrack || b0 == dirt || b0 == grass
                                    {
                                        blocks[j2 as usize] = 0;
                                    }
                                }
                                j2 -= 1;
                            }
                        }
                    }
                    if flag1
                    {
                        break;
                    }
                }
            }
        }
        tunnel += 1;
    }
}

fn recursiveGenerate(
    this: &Rc<MapGenBase>,
    _world: &Rc<World>,
    startChunkX: i32,
    startChunkZ: i32,
    chunkX: i32,
    chunkZ: i32,
    blocks: &mut [i8],
)
{
    let rand: &Random = &this.m_rand;
    let mut i: i32 = rand.nextIntBounded(rand.nextIntBounded(rand.nextIntBounded(10) + 1) + 1);
    if rand.nextIntBounded(5) != 0
    {
        i = 0;
    }
    for _ in 0..i
    {
        let d0: f64 = (startChunkX * 16 + rand.nextIntBounded(16)) as f64;
        let d1: f64 = rand.nextIntBounded(128) as f64;
        let d2: f64 = (startChunkZ * 16 + rand.nextIntBounded(16)) as f64;
        let mut k: i32 = 1;
        if rand.nextIntBounded(4) == 0
        {
            carveRoom(this, chunkX, chunkZ, blocks, d0, d1, d2);
            k += rand.nextIntBounded(4);
        }
        for _ in 0..k
        {
            let f: f32 = rand.nextFloat() * PI_F * 2.0f32;
            let f1: f32 = (rand.nextFloat() - 0.5f32) * 2.0f32 / 8.0f32;
            let f2: f32 = rand.nextFloat() * 2.0f32 + rand.nextFloat();
            carveTunnel(this, chunkX, chunkZ, blocks, d0, d1, d2, f2 * 2.0f32, f, f1, 0, 0, 0.5);
        }
    }
}

pub const MAP_GEN_CAVES_HELL: MapGenBaseClass =
    MapGenBaseClass { m_name: "MapGenCavesHell", m_parent: Some(&MAP_GEN_BASE), recursiveGenerate, ..MAP_GEN_BASE };
