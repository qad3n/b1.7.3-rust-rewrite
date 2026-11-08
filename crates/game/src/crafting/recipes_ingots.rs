use crate::block::Block;
use crate::crafting::{CraftingManager, Ingredient};
use crate::item::{Item, ItemStack};
use std::rc::Rc;

pub(super) fn addRecipes(manager: &CraftingManager)
{
    let recipeItems: [(Rc<Block>, Rc<ItemStack>); 4] = [
        (Block::blockGold(), ItemStack::withItemCount(&Item::ingotGold(), 9)),
        (Block::blockSteel(), ItemStack::withItemCount(&Item::ingotIron(), 9)),
        (Block::blockDiamond(), ItemStack::withItemCount(&Item::diamond(), 9)),
        (Block::blockLapis(), ItemStack::withItemDamage(&Item::dyePowder(), 9, 4)),
    ];
    for (block, itemStack) in &recipeItems
    {
        manager.addRecipe(
            ItemStack::withBlock(block),
            &["###", "###", "###"],
            &[('#', Ingredient::Stack(Rc::clone(itemStack)))],
        );
        manager.addRecipe(Rc::clone(itemStack), &["#"], &[('#', Ingredient::Block(Rc::clone(block)))]);
    }
}
