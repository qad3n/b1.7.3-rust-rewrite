use crate::block::{Block, BlockClass, Material, block_breakable};
use std::rc::Rc;

pub const CLASS: BlockClass = BlockClass {
    m_name: "BlockGlass",
    m_parent: Some(&block_breakable::CLASS),
    quantityDropped: |_, _| 0,
    getRenderBlockPass: |_| 0,
    ..block_breakable::CLASS
};

pub fn create(id: i32, sprite: i32, material: Rc<Material>, culling: bool) -> Rc<Block>
{
    block_breakable::construct(&CLASS, id, sprite, material, culling)
}
