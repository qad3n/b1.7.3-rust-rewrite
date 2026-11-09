use crate::block::{BLOCK, Block, BlockClass, Material};
use crate::item::Item;
use jvm::random::Random;
use std::rc::Rc;

fn idDropped(this: &Rc<Block>, _metadata: i32, _random: &Random) -> i32
{
    if this.m_blockID == Block::ORE_COAL
    {
        return Item::coal().m_shiftedIndex;
    }
    if this.m_blockID == Block::ORE_DIAMOND
    {
        return Item::diamond().m_shiftedIndex;
    }
    if this.m_blockID == Block::ORE_LAPIS
    {
        return Item::dyePowder().m_shiftedIndex;
    }
    this.m_blockID
}

pub const CLASS: BlockClass = BlockClass {
    m_name: "BlockOre",
    m_parent: Some(&BLOCK),
    idDropped,
    quantityDropped: |this, random| {
        if this.m_blockID == Block::ORE_LAPIS
        {
            4 + random.nextIntBounded(5)
        }
        else
        {
            1
        }
    },
    damageDropped: |this, _| {
        if this.m_blockID == Block::ORE_LAPIS
        {
            4
        }
        else
        {
            0
        }
    },
    ..BLOCK
};

pub fn create(id: i32, sprite: i32) -> Rc<Block>
{
    Block::constructWithSprite(&CLASS, Box::new(()), id, sprite, Material::rock())
}
