use super::items::toolOf;
use game::block::Block;
use game::item::ItemStack;
use mod_protocol::spec::ToolClass;
use std::rc::Rc;

// only blocks with a tool component, or held mod tools, change the vanilla answer
pub fn canHarvest(block: &Rc<Block>, stack: Option<&Rc<ItemStack>>) -> Option<bool>
{
    let data = super::blockData(block.m_blockID)?;
    let tool = data.m_tool.clone()?;
    let held: Option<(ToolClass, i32)> = stack.and_then(|stack| toolOf(&stack.getItem()));
    Some(held.is_some_and(|(class, level)| class == tool.class && level >= tool.level))
}

pub fn strVsBlock(block: &Rc<Block>, stack: Option<&Rc<ItemStack>>) -> Option<f32>
{
    let data = super::blockData(block.m_blockID)?;
    let tool = data.m_tool.clone()?;
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
    Some(game::item::item_tool::fields(&item).m_toolMaterial.getEfficiencyOnProperMaterial())
}
