use crate::block::{Block, BlockClass, Material, block_container};
use crate::tileentity::tile_entity_mob_spawner;
use std::rc::Rc;

pub const CLASS: BlockClass = BlockClass {
    m_name: "BlockMobSpawner",
    m_parent: Some(&block_container::CLASS),
    getBlockEntity: |_| tile_entity_mob_spawner::create(),
    idDropped: |_, _, _| 0,
    quantityDropped: |_, _| 0,
    isOpaqueCube: |_| false,
    ..block_container::CLASS
};

pub fn create(id: i32, sprite: i32) -> Rc<Block>
{
    block_container::constructWithSprite(&CLASS, Box::new(()), id, sprite, Material::rock())
}
