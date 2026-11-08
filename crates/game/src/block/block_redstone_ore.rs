use crate::block::{BLOCK, Block, BlockClass, Material};
use crate::item::Item;
use crate::world::World;
use jvm::random::Random;
use std::cell::Cell;
use std::rc::Rc;

#[derive(Default)]
pub struct OreExt
{
    m_lit: Cell<bool>,
}

fn interact(this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32)
{
    addParticles(world, x, y, z);
    if this.m_blockID == Block::oreRedstone().m_blockID
    {
        world.setBlockWithNotify(x, y, z, Block::oreRedstoneGlowing().m_blockID);
    }
}

fn addParticles(world: &Rc<World>, x: i32, y: i32, z: i32)
{
    let random: &Random = &world.m_rand;
    let d0: f64 = 0.0625;
    for i in 0..6
    {
        let mut d1: f64 = (x as f32 + random.nextFloat()) as f64;
        let mut d2: f64 = (y as f32 + random.nextFloat()) as f64;
        let mut d3: f64 = (z as f32 + random.nextFloat()) as f64;
        if i == 0 && !world.isBlockOpaqueCube(x, y + 1, z)
        {
            d2 = (y + 1) as f64 + d0;
        }
        if i == 1 && !world.isBlockOpaqueCube(x, y - 1, z)
        {
            d2 = (y + 0) as f64 - d0;
        }
        if i == 2 && !world.isBlockOpaqueCube(x, y, z + 1)
        {
            d3 = (z + 1) as f64 + d0;
        }
        if i == 3 && !world.isBlockOpaqueCube(x, y, z - 1)
        {
            d3 = (z + 0) as f64 - d0;
        }
        if i == 4 && !world.isBlockOpaqueCube(x + 1, y, z)
        {
            d1 = (x + 1) as f64 + d0;
        }
        if i == 5 && !world.isBlockOpaqueCube(x - 1, y, z)
        {
            d1 = (x + 0) as f64 - d0;
        }
        if d1 < x as f64
            || d1 > (x + 1) as f64
            || d2 < 0.0
            || d2 > (y + 1) as f64
            || d3 < z as f64
            || d3 > (z + 1) as f64
        {
            world.spawnParticle("reddust", d1, d2, d3, 0.0, 0.0, 0.0);
        }
    }
}

pub const CLASS: BlockClass = BlockClass {
    m_name: "BlockRedstoneOre",
    m_parent: Some(&BLOCK),
    tickRate: |_| 30,
    onBlockClicked: |this, world, x, y, z, player| {
        interact(this, world, x, y, z);
        (BLOCK.onBlockClicked)(this, world, x, y, z, player);
    },
    onEntityWalking: |this, world, x, y, z, entity| {
        interact(this, world, x, y, z);
        (BLOCK.onEntityWalking)(this, world, x, y, z, entity);
    },
    blockActivated: |this, world, x, y, z, player| {
        interact(this, world, x, y, z);
        (BLOCK.blockActivated)(this, world, x, y, z, player)
    },
    updateTick: |this, world, x, y, z, _| {
        if this.m_blockID == Block::oreRedstoneGlowing().m_blockID
        {
            world.setBlockWithNotify(x, y, z, Block::oreRedstone().m_blockID);
        }
    },
    idDropped: |_, _, _| Item::redstone().m_shiftedIndex,
    quantityDropped: |_, random| 4 + random.nextIntBounded(2),
    randomDisplayTick: |this, world, x, y, z, _| {
        if this.ext::<OreExt>().m_lit.get()
        {
            addParticles(world, x, y, z);
        }
    },
    ..BLOCK
};

pub fn create(id: i32, sprite: i32, lit: bool) -> Rc<Block>
{
    let mut block: Rc<Block> =
        Block::constructWithSprite(&CLASS, Box::new(OreExt::default()), id, sprite, Material::rock());
    if lit
    {
        block = block.setTickOnLoad(true);
    }
    block.ext::<OreExt>().m_lit.set(lit);
    block
}
