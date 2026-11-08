use crate::block::{Block, Material};
use crate::item::{EnumToolMaterial, Item, ItemClass, item_tool};
use std::rc::Rc;

fn blocksEffectiveAgainst() -> Vec<Rc<Block>>
{
    vec![
        Block::cobblestone(),
        Block::stairDouble(),
        Block::stairSingle(),
        Block::stone(),
        Block::sandStone(),
        Block::cobblestoneMossy(),
        Block::oreIron(),
        Block::blockSteel(),
        Block::oreCoal(),
        Block::blockGold(),
        Block::oreGold(),
        Block::oreDiamond(),
        Block::blockDiamond(),
        Block::ice(),
        Block::netherrack(),
        Block::oreLapis(),
        Block::blockLapis(),
    ]
}

fn canHarvestBlock(this: &Rc<Item>, block: &Rc<Block>) -> bool
{
    let level: i32 = item_tool::fields(this).m_toolMaterial.getHarvestLevel();
    let is = |other: Rc<Block>| Rc::ptr_eq(block, &other);
    if is(Block::obsidian())
    {
        return level == 3;
    }
    if is(Block::blockDiamond()) || is(Block::oreDiamond())
    {
        return level >= 2;
    }
    if is(Block::blockGold()) || is(Block::oreGold())
    {
        return level >= 2;
    }
    if is(Block::blockSteel()) || is(Block::oreIron())
    {
        return level >= 1;
    }
    if is(Block::blockLapis()) || is(Block::oreLapis())
    {
        return level >= 1;
    }
    if is(Block::oreRedstone()) || is(Block::oreRedstoneGlowing())
    {
        return level >= 2;
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
    item_tool::construct(&CLASS, id, 2, tier, blocksEffectiveAgainst())
}
