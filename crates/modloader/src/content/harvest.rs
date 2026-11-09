use super::items::{materialOf, toolOf};
use game::block::Block;
use game::item::{Item, ItemStack};
use mod_protocol::Identifier;
use mod_protocol::abi::RegistryKind;
use mod_protocol::spec::{ToolClass, ToolSpec};
use std::rc::Rc;

// tool class, vanilla tool class name and the mineable tag its effective blocks seed
const MINEABLE: [(ToolClass, &str, &str); 3] = [
    (ToolClass::Pickaxe, "ItemPickaxe", "mineable/pickaxe"),
    (ToolClass::Axe, "ItemAxe", "mineable/axe"),
    (ToolClass::Shovel, "ItemSpade", "mineable/shovel"),
];

// vanilla tool lists become minecraft:mineable tags that mods can add their blocks to
pub fn seedTags()
{
    for (_, className, tag) in MINEABLE
    {
        let blocks: Vec<Identifier> = crate::registry::with(|registries| {
            let vanilla = registries.m_items.rows().filter(|row| row.m_id.isVanilla());
            let Some(tool) =
                vanilla.filter_map(|row| Item::byId(row.m_numeric)).find(|item| item.className() == className)
            else
            {
                return Vec::new();
            };
            game::item::item_tool::effectiveBlocks(&tool)
                .iter()
                .filter_map(|block| registries.m_blocks.byNumeric(block.m_blockID).map(|row| row.m_id.clone()))
                .collect()
        });
        super::tags::add(RegistryKind::Block, Identifier::minecraft(tag), blocks, false);
    }
}

// a block tagged for a tool class mines at that tool's speed, vanilla lists still answer their own blocks
fn taggedSpeed(block: &Rc<Block>, stack: &Rc<ItemStack>) -> Option<f32>
{
    let item: Rc<Item> = stack.getItem();
    let (class, _) = toolOf(&item)?;
    let (_, _, tag) = MINEABLE.iter().find(|(row, _, _)| *row == class)?;
    let id: Identifier =
        crate::registry::with(|registries| registries.m_blocks.byNumeric(block.m_blockID).map(|row| row.m_id.clone()))?;
    if !super::tags::contains(RegistryKind::Block, &Identifier::minecraft(tag), &id)
    {
        return None;
    }
    if let Some(modTool) = super::itemData(item.m_shiftedIndex).and_then(|data| data.m_tool.clone())
    {
        return Some(modTool.speed);
    }
    if game::item::item_tool::effectiveBlocks(&item).iter().any(|effective| Rc::ptr_eq(effective, block))
    {
        return None;
    }

    toolSpeed(&item)
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
