use crate::block::{Block, Material};
use crate::item::{EnumToolMaterial, Item, ItemClass, item_tool};
use crate::tags::{self, Tag};
use std::rc::Rc;

// harvest levels of the tool tiers a block can ask for
mod tier
{
    pub const STONE: i32 = 1;
    pub const IRON: i32 = 2;
    pub const DIAMOND: i32 = 3;
}

fn canHarvestBlock(this: &Rc<Item>, block: &Rc<Block>) -> bool
{
    let level: i32 = item_tool::fields(this).m_toolMaterial.getHarvestLevel();
    let id: i32 = block.m_blockID;
    // beta asks for exactly a diamond tier here, not at least one
    if tags::has(Tag::NeedsDiamondTool, id)
    {
        return level == tier::DIAMOND;
    }
    if tags::has(Tag::NeedsIronTool, id)
    {
        return level >= tier::IRON;
    }
    if tags::has(Tag::NeedsStoneTool, id)
    {
        return level >= tier::STONE;
    }
    if Rc::ptr_eq(&block.m_blockMaterial, &Material::rock())
    {
        return true;
    }

    Rc::ptr_eq(&block.m_blockMaterial, &Material::iron())
}

pub const CLASS: ItemClass =
    ItemClass { m_name: "ItemPickaxe", m_parent: Some(&item_tool::CLASS), canHarvestBlock, ..item_tool::CLASS };

pub fn create(id: i32, tier: EnumToolMaterial) -> Rc<Item>
{
    item_tool::construct(&CLASS, id, 2, tier, Tag::MineablePickaxe)
}
