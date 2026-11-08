use crate::block::Block;
use crate::crafting::{CraftingManager, Ingredient};
use crate::item::{Item, ItemStack};
use std::rc::Rc;

pub(super) fn addRecipes(manager: &CraftingManager)
{
    let recipePatterns: [&[&str]; 1] = [&["X", "X", "#"]];
    let materials: [Ingredient; 5] = [
        Ingredient::Block(Block::planks()),
        Ingredient::Block(Block::cobblestone()),
        Ingredient::Item(Item::ingotIron()),
        Ingredient::Item(Item::diamond()),
        Ingredient::Item(Item::ingotGold()),
    ];
    let weapons: [[Rc<Item>; 5]; 1] =
        [[Item::swordWood(), Item::swordStone(), Item::swordSteel(), Item::swordDiamond(), Item::swordGold()]];
    for (i, object) in materials.iter().enumerate()
    {
        for (j, row) in weapons.iter().enumerate()
        {
            manager.addRecipe(
                ItemStack::withItem(&row[i]),
                recipePatterns[j],
                &[('#', Ingredient::Item(Item::stick())), ('X', object.clone())],
            );
        }
    }

    manager.addRecipe(
        ItemStack::withItemCount(&Item::bow(), 1),
        &[" #X", "# X", " #X"],
        &[('X', Ingredient::Item(Item::silk())), ('#', Ingredient::Item(Item::stick()))],
    );
    manager.addRecipe(
        ItemStack::withItemCount(&Item::arrow(), 4),
        &["X", "#", "Y"],
        &[
            ('Y', Ingredient::Item(Item::feather())),
            ('X', Ingredient::Item(Item::flint())),
            ('#', Ingredient::Item(Item::stick())),
        ],
    );
}
