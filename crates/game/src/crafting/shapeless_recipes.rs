use crate::crafting::{CraftingGrid, IRecipe};
use crate::item::ItemStack;
use std::rc::Rc;

pub struct ShapelessRecipes
{
    m_recipeOutput: Rc<ItemStack>,
    m_recipeItems: Vec<Rc<ItemStack>>,
}

impl ShapelessRecipes
{
    pub fn new(result: Rc<ItemStack>, ingredients: Vec<Rc<ItemStack>>) -> ShapelessRecipes
    {
        ShapelessRecipes { m_recipeOutput: result, m_recipeItems: ingredients }
    }
}

impl IRecipe for ShapelessRecipes
{
    fn matches(&self, inventory: &dyn CraftingGrid) -> bool
    {
        let mut arraylist: Vec<Rc<ItemStack>> = self.m_recipeItems.clone();
        for i in 0..3
        {
            for j in 0..3
            {
                let Some(itemstack) = inventory.getItem(j, i)
                else
                {
                    continue;
                };
                let found: Option<usize> = arraylist.iter().position(|itemstack1| {
                    itemstack.m_itemID.get() == itemstack1.m_itemID.get()
                        && (itemstack1.getItemDamage() == -1 || itemstack.getItemDamage() == itemstack1.getItemDamage())
                });
                let Some(index) = found
                else
                {
                    return false;
                };
                arraylist.remove(index);
            }
        }

        arraylist.is_empty()
    }

    fn getCraftingResult(&self, _inventory: &dyn CraftingGrid) -> Rc<ItemStack> { self.m_recipeOutput.copy() }

    fn getRecipeSize(&self) -> i32 { self.m_recipeItems.len() as i32 }

    fn getRecipeOutput(&self) -> Rc<ItemStack> { Rc::clone(&self.m_recipeOutput) }

    fn instanceOf(&self, name: &str) -> bool { matches!(name, "ShapelessRecipes" | "IRecipe") }
}
