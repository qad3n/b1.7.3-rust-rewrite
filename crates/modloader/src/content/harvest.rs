use super::items::{materialOf, toolOf};
use game::block::Block;
use game::item::{Item, ItemStack};
use game::tags::Tag;
use mod_protocol::spec::{ToolClass, ToolSpec};
use std::rc::Rc;

// tool class and the mineable tag its effective blocks are in
const MINEABLE: [(ToolClass, Tag); 3] = [
    (ToolClass::Pickaxe, Tag::MineablePickaxe),
    (ToolClass::Axe, Tag::MineableAxe),
    (ToolClass::Shovel, Tag::MineableShovel),
];

// vanilla tools answer tagged blocks themselves, a mod tool mines them at its own speed
fn taggedSpeed(block: &Rc<Block>, stack: &Rc<ItemStack>) -> Option<f32>
{
    let item: Rc<Item> = stack.getItem();
    let (class, _) = toolOf(&item)?;
    let (_, tag) = MINEABLE.iter().find(|(row, _)| *row == class)?;
    if !game::tags::has(*tag, block.m_blockID)
    {
        return None;
    }

    super::itemData(item.m_shiftedIndex).and_then(|data| data.m_tool.as_ref().map(|tool| tool.speed))
}

// a modify change wins over the block's own spec
fn toolFor(block: &Rc<Block>) -> Option<ToolSpec>
{
    crate::modify::tool(block.m_blockID).or_else(|| super::blockData(block.m_blockID)?.m_tool.clone())
}

// only blocks with a tool component, or held mod tools, change the vanilla answer
// as in forge the component gates only materials that need a tool, so dirt and wood still drop by hand
pub fn canHarvest(block: &Rc<Block>, stack: Option<&Rc<ItemStack>>) -> Option<bool>
{
    if block.m_blockMaterial.getIsHarvestable()
    {
        return None;
    }
    let tool: ToolSpec = toolFor(block)?;
    let held: Option<(ToolClass, i32)> = stack.and_then(|stack| toolOf(&stack.getItem()));
    Some(held.is_some_and(|(class, level)| class == tool.class && level >= tool.level))
}

pub fn strVsBlock(block: &Rc<Block>, stack: Option<&Rc<ItemStack>>) -> Option<f32>
{
    let Some(tool) = toolFor(block)
    else
    {
        return taggedSpeed(block, stack?);
    };
    let stack: &Rc<ItemStack> = stack?;
    let item: Rc<game::item::Item> = stack.getItem();
    let (class, _) = toolOf(&item)?;
    if class != tool.class
    {
        return None;
    }
    if let Some(modTool) = super::itemData(item.m_shiftedIndex).and_then(|data| data.m_tool.clone())
    {
        return Some(modTool.speed);
    }
    toolSpeed(&item)
}

fn toolSpeed(item: &Item) -> Option<f32> { materialOf(item).map(|material| material.getEfficiencyOnProperMaterial()) }
