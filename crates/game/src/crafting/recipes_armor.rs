use crate::block::Block;
use crate::crafting::{CraftingManager, Ingredient};
use crate::item::{Item, ItemStack};
use std::rc::Rc;

pub(super) fn addRecipes(manager: &CraftingManager)
{
    let recipePatterns: [&[&str]; 4] =
        [&["XXX", "X X"], &["X X", "XXX", "XXX"], &["XXX", "X X", "X X"], &["X X", "X X"]];
    let materials: [Ingredient; 5] = [
        Ingredient::Item(Item::leather()),
        Ingredient::Block(Block::fire()),
        Ingredient::Item(Item::ingotIron()),
        Ingredient::Item(Item::diamond()),
        Ingredient::Item(Item::ingotGold()),
    ];
    let armor: [[Rc<Item>; 5]; 4] = [
        [Item::helmetLeather(), Item::helmetChain(), Item::helmetSteel(), Item::helmetDiamond(), Item::helmetGold()],
        [Item::plateLeather(), Item::plateChain(), Item::plateSteel(), Item::plateDiamond(), Item::plateGold()],
        [Item::legsLeather(), Item::legsChain(), Item::legsSteel(), Item::legsDiamond(), Item::legsGold()],
        [Item::bootsLeather(), Item::bootsChain(), Item::bootsSteel(), Item::bootsDiamond(), Item::bootsGold()],
    ];
    for (i, object) in materials.iter().enumerate()
    {
        for (j, row) in armor.iter().enumerate()
        {
            manager.addRecipe(ItemStack::withItem(&row[i]), recipePatterns[j], &[('X', object.clone())]);
        }
    }
}
