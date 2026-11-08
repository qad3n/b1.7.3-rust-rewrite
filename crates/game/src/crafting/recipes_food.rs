use crate::block::Block;
use crate::crafting::{CraftingManager, Ingredient};
use crate::item::{Item, ItemStack};

pub(super) fn addRecipes(manager: &CraftingManager)
{
    manager.addRecipe(
        ItemStack::withItem(&Item::bowlSoup()),
        &["Y", "X", "#"],
        &[
            ('X', Ingredient::Block(Block::mushroomBrown())),
            ('Y', Ingredient::Block(Block::mushroomRed())),
            ('#', Ingredient::Item(Item::bowlEmpty())),
        ],
    );
    manager.addRecipe(
        ItemStack::withItem(&Item::bowlSoup()),
        &["Y", "X", "#"],
        &[
            ('X', Ingredient::Block(Block::mushroomRed())),
            ('Y', Ingredient::Block(Block::mushroomBrown())),
            ('#', Ingredient::Item(Item::bowlEmpty())),
        ],
    );
    manager.addRecipe(
        ItemStack::withItemCount(&Item::cookie(), 8),
        &["#X#"],
        &[
            ('X', Ingredient::Stack(ItemStack::withItemDamage(&Item::dyePowder(), 1, 3))),
            ('#', Ingredient::Item(Item::wheat())),
        ],
    );
}
