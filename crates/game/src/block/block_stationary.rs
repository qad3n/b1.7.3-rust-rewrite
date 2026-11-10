use crate::block::{Block, BlockClass, Material, block_fluid};
use crate::fluid::FluidStyle;
use crate::world::World;
use jvm::random::Random;
use jvm::throwable::OrNpe;
use std::rc::Rc;

fn convertToFlowing(this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32)
{
    let i: i32 = world.getBlockMetadata(x, y, z);
    world.m_editingBlocks.set(true);
    let flowing: i32 = block_fluid::fluidOf(this).m_flowing.get();
    world.setBlockAndMetadata(x, y, z, flowing, i);
    world.markBlocksDirty(x, y, z, x, y, z);
    world.scheduleBlockUpdate(x, y, z, flowing, this.tickRate());
    world.m_editingBlocks.set(false);
}

fn isFlammable(world: &Rc<World>, x: i32, y: i32, z: i32) -> bool { world.getBlockMaterial(x, y, z).getBurning() }

fn updateTick(this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32, random: &Random)
{
    if block_fluid::fluidOf(this).m_style != FluidStyle::Lava
    {
        return;
    }
    let (mut x, mut y, mut z): (i32, i32, i32) = (x, y, z);
    let i: i32 = random.nextIntBounded(3);
    for _ in 0..i
    {
        x += random.nextIntBounded(3) - 1;
        y += 1;
        z += random.nextIntBounded(3) - 1;
        let k: i32 = world.getBlockId(x, y, z);
        if k == 0
        {
            if isFlammable(world, x - 1, y, z)
                || isFlammable(world, x + 1, y, z)
                || isFlammable(world, x, y, z - 1)
                || isFlammable(world, x, y, z + 1)
                || isFlammable(world, x, y - 1, z)
                || isFlammable(world, x, y + 1, z)
            {
                world.setBlockWithNotify(x, y, z, Block::FIRE);
                return;
            }
        }
        else if Block::byId(k).orNpe().m_blockMaterial.getIsSolid()
        {
            return;
        }
    }
}

pub const CLASS: BlockClass = BlockClass {
    m_name: "BlockStationary",
    m_parent: Some(&block_fluid::CLASS),
    onNeighborBlockChange: |this, world, x, y, z, neighborBlock| {
        (block_fluid::CLASS.onNeighborBlockChange)(this, world, x, y, z, neighborBlock);
        if world.getBlockId(x, y, z) == this.m_blockID
        {
            convertToFlowing(this, world, x, y, z);
        }
    },
    updateTick,
    ..block_fluid::CLASS
};

pub fn create(id: i32, material: Rc<Material>) -> Rc<Block>
{
    let lava: bool = crate::fluid::of(&material).is_some_and(|fluid| fluid.m_style == FluidStyle::Lava);
    let block: Rc<Block> = block_fluid::construct(&CLASS, Box::new(()), id, material).setTickOnLoad(false);
    if lava
    {
        return block.setTickOnLoad(true);
    }
    block
}
