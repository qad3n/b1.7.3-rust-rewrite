use crate::block::{Block, Material};
use crate::world::{EnumSkyBlock, World};
use crate::worldgen::WorldGenerator;
use jvm::random::Random;
use jvm::throwable::OrNpe;
use std::rc::Rc;

pub struct WorldGenLakes
{
    m_liquid: i32,
}

impl WorldGenLakes
{
    pub fn new(liquid: i32) -> WorldGenLakes { WorldGenLakes { m_liquid: liquid } }
}

fn index(x: i32, z: i32, y: i32) -> usize { ((x * 16 + z) * 8 + y) as usize }

fn isEdge(shape: &[bool], x: i32, z: i32, y: i32) -> bool
{
    !shape[index(x, z, y)]
        && (x < 15 && shape[index(x + 1, z, y)]
            || x > 0 && shape[index(x - 1, z, y)]
            || z < 15 && shape[index(x, z + 1, y)]
            || z > 0 && shape[index(x, z - 1, y)]
            || y < 7 && shape[index(x, z, y + 1)]
            || y > 0 && shape[index(x, z, y - 1)])
}

impl WorldGenerator for WorldGenLakes
{
    fn generate(&self, world: &Rc<World>, random: &Random, mut x: i32, mut y: i32, mut z: i32) -> bool
    {
        x -= 8;
        z -= 8;
        while y > 0 && world.isAirBlock(x, y, z)
        {
            y -= 1;
        }
        y -= 4;
        let mut shape: Vec<bool> = vec![false; 2048];
        let i: i32 = random.nextIntBounded(4) + 4;
        for _ in 0..i
        {
            let d0: f64 = random.nextDouble() * 6.0 + 3.0;
            let d1: f64 = random.nextDouble() * 4.0 + 2.0;
            let d2: f64 = random.nextDouble() * 6.0 + 3.0;
            let d3: f64 = random.nextDouble() * (16.0 - d0 - 2.0) + 1.0 + d0 / 2.0;
            let d4: f64 = random.nextDouble() * (8.0 - d1 - 4.0) + 2.0 + d1 / 2.0;
            let d5: f64 = random.nextDouble() * (16.0 - d2 - 2.0) + 1.0 + d2 / 2.0;
            for l in 1..15
            {
                for i1 in 1..15
                {
                    for j1 in 1..7
                    {
                        let d6: f64 = (l as f64 - d3) / (d0 / 2.0);
                        let d7: f64 = (j1 as f64 - d4) / (d1 / 2.0);
                        let d8: f64 = (i1 as f64 - d5) / (d2 / 2.0);
                        let d9: f64 = d6 * d6 + d7 * d7 + d8 * d8;
                        if d9 < 1.0
                        {
                            shape[index(l, i1, j1)] = true;
                        }
                    }
                }
            }
        }
        for k1 in 0..16
        {
            for k2 in 0..16
            {
                for k in 0..8
                {
                    if isEdge(&shape, k1, k2, k)
                    {
                        let material: Rc<Material> = world.getBlockMaterial(x + k1, y + k, z + k2);
                        if k >= 4 && material.getIsLiquid()
                        {
                            return false;
                        }
                        if k < 4 && !material.isSolid() && world.getBlockId(x + k1, y + k, z + k2) != self.m_liquid
                        {
                            return false;
                        }
                    }
                }
            }
        }
        for l1 in 0..16
        {
            for l2 in 0..16
            {
                for k3 in 0..8
                {
                    if shape[index(l1, l2, k3)]
                    {
                        world.setBlock(x + l1, y + k3, z + l2, if k3 >= 4 { 0 } else { self.m_liquid });
                    }
                }
            }
        }
        for i2 in 0..16
        {
            for i3 in 0..16
            {
                for l3 in 4..8
                {
                    if shape[index(i2, i3, l3)]
                        && world.getBlockId(x + i2, y + l3 - 1, z + i3) == Block::dirt().m_blockID
                        && world.getSavedLightValue(EnumSkyBlock::Sky, x + i2, y + l3, z + i3) > 0
                    {
                        world.setBlock(x + i2, y + l3 - 1, z + i3, Block::grass().m_blockID);
                    }
                }
            }
        }
        if Rc::ptr_eq(&Block::byId(self.m_liquid).orNpe().m_blockMaterial, &Material::lava())
        {
            for j2 in 0..16
            {
                for j3 in 0..16
                {
                    for i4 in 0..8
                    {
                        if isEdge(&shape, j2, j3, i4)
                            && (i4 < 4 || random.nextIntBounded(2) != 0)
                            && world.getBlockMaterial(x + j2, y + i4, z + j3).isSolid()
                        {
                            world.setBlock(x + j2, y + i4, z + j3, Block::stone().m_blockID);
                        }
                    }
                }
            }
        }
        true
    }
}
