use crate::block::{BLOCK, Block, BlockClass, Material};
use crate::world::{IBlockAccess, World};
use crate::worldgen::ColorizerGrass;
use jvm::random::Random;
use std::rc::Rc;

fn getBlockTexture(_this: &Rc<Block>, world: &dyn IBlockAccess, x: i32, y: i32, z: i32, face: i32) -> i32
{
    if face == 1
    {
        return 0;
    }
    if face == 0
    {
        return 2;
    }
    let material: Rc<Material> = world.getBlockMaterial(x, y + 1, z);
    if Rc::ptr_eq(&material, &Material::snow()) || Rc::ptr_eq(&material, &Material::builtSnow())
    {
        return 68;
    }
    3
}

fn colorMultiplier(_this: &Rc<Block>, world: &dyn IBlockAccess, x: i32, _y: i32, z: i32) -> i32
{
    world.getWorldChunkManager().getBiomes(x, z, 1, 1);
    let d: f64 = world.getWorldChunkManager().temperature(0);
    let d2: f64 = world.getWorldChunkManager().humidity(0);
    ColorizerGrass::getGrassColor(d, d2)
}

fn updateTick(_this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32, random: &Random)
{
    if world.m_multiplayerWorld.get()
    {
        return;
    }
    if world.getBlockLightValue(x, y + 1, z) < 4 && Block::lightOpacity(world.getBlockId(x, y + 1, z)) > 2
    {
        if random.nextIntBounded(4) != 0
        {
            return;
        }
        world.setBlockWithNotify(x, y, z, Block::dirt().m_blockID);
    }
    else if world.getBlockLightValue(x, y + 1, z) >= 9
    {
        let n: i32 = x + random.nextIntBounded(3) - 1;
        let n2: i32 = y + random.nextIntBounded(5) - 3;
        let n3: i32 = z + random.nextIntBounded(3) - 1;
        let n4: i32 = world.getBlockId(n, n2 + 1, n3);
        if world.getBlockId(n, n2, n3) == Block::dirt().m_blockID
            && world.getBlockLightValue(n, n2 + 1, n3) >= 4
            && Block::lightOpacity(n4) <= 2
        {
            world.setBlockWithNotify(n, n2, n3, Block::grass().m_blockID);
        }
    }
}

pub const CLASS: BlockClass = BlockClass {
    m_name: "BlockGrass",
    m_parent: Some(&BLOCK),
    getBlockTexture,
    colorMultiplier,
    updateTick,
    idDropped: |_, _, random| Block::dirt().idDropped(0, random),
    ..BLOCK
};

pub fn create(id: i32) -> Rc<Block>
{
    let block: Rc<Block> = Block::construct(&CLASS, Box::new(()), id, Material::grassMaterial());
    block.m_blockIndexInTexture.set(3);
    block.setTickOnLoad(true)
}
