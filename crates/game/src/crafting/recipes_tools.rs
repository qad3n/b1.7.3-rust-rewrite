use crate::block::Block;
use crate::crafting::{CraftingManager, Ingredient};
use crate::item::{Item, ItemStack};
use std::rc::Rc;

pub(super) fn addRecipes(manager: &CraftingManager)
{
    let recipePatterns: [&[&str]; 4] =
        [&["XXX", " # ", " # "], &["X", "#", "#"], &["XX", "X#", " #"], &["XX", " #", " #"]];
    let materials: [Ingredient; 5] = [
        Ingredient::Block(Block::planks()),
        Ingredient::Block(Block::cobblestone()),
        Ingredient::Item(Item::ingotIron()),
        Ingredient::Item(Item::diamond()),
        Ingredient::Item(Item::ingotGold()),
    ];
    let tools: [[Rc<Item>; 5]; 4] = [
        [Item::pickaxeWood(), Item::pickaxeStone(), Item::pickaxeSteel(), Item::pickaxeDiamond(), Item::pickaxeGold()],
        [Item::shovelWood(), Item::shovelStone(), Item::shovelSteel(), Item::shovelDiamond(), Item::shovelGold()],
        [Item::axeWood(), Item::axeStone(), Item::axeSteel(), Item::axeDiamond(), Item::axeGold()],
        [Item::hoeWood(), Item::hoeStone(), Item::hoeSteel(), Item::hoeDiamond(), Item::hoeGold()],
    ];
    for (i, object) in materials.iter().enumerate()
    {
        for (j, row) in tools.iter().enumerate()
        {
            manager.addRecipe(
                ItemStack::withItem(&row[i]),
                recipePatterns[j],
                &[('#', Ingredient::Item(Item::stick())), ('X', object.clone())],
            );
        }
    }

    manager.addRecipe(
        ItemStack::withItem(&Item::shears()),
        &[" #", "# "],
        &[('#', Ingredient::Item(Item::ingotIron()))],
    );
}
