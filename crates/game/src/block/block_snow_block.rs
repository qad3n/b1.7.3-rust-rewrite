use crate::block::{BLOCK, Block, BlockClass, Material};
use crate::item::Item;
use crate::world::EnumSkyBlock;
use std::rc::Rc;

pub const CLASS: BlockClass = BlockClass {
    m_name: "BlockSnowBlock",
    m_parent: Some(&BLOCK),
    idDropped: |_, _, _| Item::snowball().m_shiftedIndex,
    quantityDropped: |_, _| 4,
    updateTick: |this, world, x, y, z, _| {
        if world.getSavedLightValue(EnumSkyBlock::Block, x, y, z) > 11
        {
            this.dropBlockAsItem(world, x, y, z, world.getBlockMetadata(x, y, z));
            world.setBlockWithNotify(x, y, z, 0);
        }
    },
    ..BLOCK
};

pub fn create(id: i32, sprite: i32) -> Rc<Block>
{
    Block::constructWithSprite(&CLASS, Box::new(()), id, sprite, Material::builtSnow()).setTickOnLoad(true)
}
