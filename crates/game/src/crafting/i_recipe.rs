use crate::item::ItemStack;
use std::rc::Rc;

pub trait CraftingGrid
{
    fn getItem(&self, column: i32, row: i32) -> Option<Rc<ItemStack>>;
}

pub trait IRecipe
{
    fn matches(&self, inventory: &dyn CraftingGrid) -> bool;

    fn getCraftingResult(&self, inventory: &dyn CraftingGrid) -> Rc<ItemStack>;

    fn getRecipeSize(&self) -> i32;

    fn getRecipeOutput(&self) -> Rc<ItemStack>;

    fn instanceOf(&self, name: &str) -> bool;

    // rewrite: recipe viewers read the ingredients, row by row with the width, width 0 for shapeless
    fn getRecipeGrid(&self) -> (i32, Vec<Option<Rc<ItemStack>>>);
}
