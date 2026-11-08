use crate::block::Block;
use crate::item::{Item, ItemClass, item_block};
use crate::worldgen::ColorizerFoliage;
use std::rc::Rc;

pub const CLASS: ItemClass = ItemClass {
    m_name: "ItemLeaves",
    m_parent: Some(&item_block::CLASS),
    getPlacedBlockMetadata: |_, metadata| metadata | 8,
    getIconFromDamage: |_, metadata| Block::leaves().getBlockTextureFromSideAndMetadata(0, metadata),
    getColorFromDamage: |_, metadata| {
        if (metadata & 1) == 1
        {
            return ColorizerFoliage::getFoliageColorPine();
        }
        if (metadata & 2) == 2
        {
            return ColorizerFoliage::getFoliageColorBirch();
        }
        ColorizerFoliage::getDefaultColor()
    },
    ..item_block::CLASS
};

pub fn create(n: i32) -> Rc<Item> { item_block::construct(&CLASS, n).setMaxDamage(0).setHasSubtypes(true) }
