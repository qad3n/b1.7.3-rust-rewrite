use crate::block::{Block, BlockClass, Material, block_breakable};
use crate::entity::Entity;
use crate::world::{EnumSkyBlock, World};
use std::rc::Rc;

fn harvestBlock(this: &Rc<Block>, world: &Rc<World>, player: &Rc<Entity>, x: i32, y: i32, z: i32, metadata: i32)
{
    (block_breakable::CLASS.harvestBlock)(this, world, player, x, y, z, metadata);
    let material: Rc<Material> = world.getBlockMaterial(x, y - 1, z);
    if material.getIsSolid() || material.getIsLiquid()
    {
        world.setBlockWithNotify(x, y, z, Block::waterMoving().m_blockID);
    }
}

fn updateTick(this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32, _random: &jvm::random::Random)
{
    if world.getSavedLightValue(EnumSkyBlock::Block, x, y, z) > 11 - Block::lightOpacity(this.m_blockID)
    {
        this.dropBlockAsItem(world, x, y, z, world.getBlockMetadata(x, y, z));
        world.setBlockWithNotify(x, y, z, Block::waterStill().m_blockID);
    }
}

pub const CLASS: BlockClass = BlockClass {
    m_name: "BlockIce",
    m_parent: Some(&block_breakable::CLASS),
    getRenderBlockPass: |_| 1,
    shouldSideBeRendered: |this, world, x, y, z, face| {
        (block_breakable::CLASS.shouldSideBeRendered)(this, world, x, y, z, 1 - face)
    },
    harvestBlock,
    quantityDropped: |_, _| 0,
    updateTick,
    getMobilityFlag: |_| 0,
    ..block_breakable::CLASS
};

pub fn create(id: i32, sprite: i32) -> Rc<Block>
{
    let block: Rc<Block> = block_breakable::construct(&CLASS, id, sprite, Material::ice(), false);
    block.m_slipperiness.set(0.98);
    block.setTickOnLoad(true)
}
