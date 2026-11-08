use crate::block::Block;
use crate::crafting::{CraftingManager, Ingredient};
use crate::item::ItemStack;

pub(super) fn addRecipes(manager: &CraftingManager)
{
    manager.addRecipe(
        ItemStack::withBlock(&Block::chest()),
        &["###", "# #", "###"],
        &[('#', Ingredient::Block(Block::planks()))],
    );
    manager.addRecipe(
        ItemStack::withBlock(&Block::stoneOvenIdle()),
        &["###", "# #", "###"],
        &[('#', Ingredient::Block(Block::cobblestone()))],
    );
    manager.addRecipe(
        ItemStack::withBlock(&Block::workbench()),
        &["##", "##"],
        &[('#', Ingredient::Block(Block::planks()))],
    );
    manager.addRecipe(
        ItemStack::withBlock(&Block::sandStone()),
        &["##", "##"],
        &[('#', Ingredient::Block(Block::sand()))],
    );
}
