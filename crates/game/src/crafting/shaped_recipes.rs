use crate::crafting::{CraftingGrid, IRecipe};
use crate::item::ItemStack;
use std::rc::Rc;

pub struct ShapedRecipes
{
    m_recipeWidth: i32,
    m_recipeHeight: i32,
    m_recipeItems: Vec<Option<Rc<ItemStack>>>,
    m_recipeOutput: Rc<ItemStack>,
    pub m_recipeOutputItemID: i32,
}

impl ShapedRecipes
{
    pub fn new(width: i32, height: i32, ingredients: Vec<Option<Rc<ItemStack>>>, result: Rc<ItemStack>)
    -> ShapedRecipes
    {
        ShapedRecipes {
            m_recipeOutputItemID: result.m_itemID.get(),
            m_recipeWidth: width,
            m_recipeHeight: height,
            m_recipeItems: ingredients,
            m_recipeOutput: result,
        }
    }

    fn matchesAt(&self, inventory: &dyn CraftingGrid, x: i32, y: i32, rotate: bool) -> bool
    {
        for i in 0..3
        {
            for j in 0..3
            {
                let k: i32 = i - x;
                let l: i32 = j - y;
                let mut itemstack: Option<Rc<ItemStack>> = None;
                if k >= 0 && l >= 0 && k < self.m_recipeWidth && l < self.m_recipeHeight
                {
                    let index: i32 = if rotate
                    {
                        self.m_recipeWidth - k - 1 + l * self.m_recipeWidth
                    }
                    else
                    {
                        k + l * self.m_recipeWidth
                    };
                    itemstack = self.m_recipeItems[index as usize].clone();
                }

                let itemstack1: Option<Rc<ItemStack>> = inventory.getItem(i, j);
                match (itemstack, itemstack1)
                {
                    (None, None) =>
                    {}
                    (Some(itemstack), Some(itemstack1)) =>
                    {
                        if itemstack.m_itemID.get() != itemstack1.m_itemID.get()
                        {
                            return false;
                        }

                        if itemstack.getItemDamage() != -1 && itemstack.getItemDamage() != itemstack1.getItemDamage()
                        {
                            return false;
                        }
                    }
                    _ => return false,
                }
            }
        }

        true
    }
}

impl IRecipe for ShapedRecipes
{
    fn matches(&self, inventory: &dyn CraftingGrid) -> bool
    {
        for i in 0..=3 - self.m_recipeWidth
        {
            for j in 0..=3 - self.m_recipeHeight
            {
                if self.matchesAt(inventory, i, j, true)
                {
                    return true;
                }

                if self.matchesAt(inventory, i, j, false)
                {
                    return true;
                }
            }
        }

        false
    }

    fn getCraftingResult(&self, _inventory: &dyn CraftingGrid) -> Rc<ItemStack>
    {
        ItemStack::withDamage(
            self.m_recipeOutput.m_itemID.get(),
            self.m_recipeOutput.m_stackSize.get(),
            self.m_recipeOutput.getItemDamage(),
        )
    }

    fn getRecipeSize(&self) -> i32 { self.m_recipeWidth * self.m_recipeHeight }

    fn getRecipeOutput(&self) -> Rc<ItemStack> { Rc::clone(&self.m_recipeOutput) }

    fn instanceOf(&self, name: &str) -> bool { matches!(name, "ShapedRecipes" | "IRecipe") }
}
