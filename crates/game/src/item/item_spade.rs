use crate::block::Block;
use crate::item::{EnumToolMaterial, Item, ItemClass, item_tool};
use std::rc::Rc;

fn blocksEffectiveAgainst() -> Vec<Rc<Block>>
{
    vec![
        Block::grass(),
        Block::dirt(),
        Block::sand(),
        Block::gravel(),
        Block::snow(),
        Block::blockSnow(),
        Block::blockClay(),
        Block::tilledField(),
    ]
}

pub const CLASS: ItemClass = ItemClass {
    m_name: "ItemSpade",
    m_parent: Some(&item_tool::CLASS),
    canHarvestBlock: |_, block| {
        if Rc::ptr_eq(block, &Block::snow())
        {
            return true;
        }
        Rc::ptr_eq(block, &Block::blockSnow())
    },
    ..item_tool::CLASS
};

pub fn create(id: i32, tier: EnumToolMaterial) -> Rc<Item>
{
    item_tool::construct(&CLASS, id, 1, tier, blocksEffectiveAgainst())
}
