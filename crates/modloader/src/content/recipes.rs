use game::crafting::{CraftingManager, FurnaceRecipes, IRecipe, Ingredient};
use game::item::ItemStack;
use mod_protocol::Identifier;
use mod_protocol::spec::{RecipeSpec, StackSpec};
use std::cell::RefCell;
use std::rc::Rc;

thread_local! {
    // recipes and smelting inputs the loader added, removed again before a rebuild under new ids
    static ADDED: RefCell<Vec<Rc<dyn IRecipe>>> = const { RefCell::new(Vec::new()) };
    static SMELTED: RefCell<Vec<i32>> = const { RefCell::new(Vec::new()) };
}

fn trackLast()
{
    let list = CraftingManager::getInstance().getRecipeList();
    let last: Rc<dyn IRecipe> = list.get(list.size() - 1);
    ADDED.with(|added| added.borrow_mut().push(last));
}

pub fn removeModded()
{
    let list = CraftingManager::getInstance().getRecipeList();
    for recipe in ADDED.with(|added| std::mem::take(&mut *added.borrow_mut()))
    {
        list.removeIdentity(&recipe);
    }
    for input in SMELTED.with(|smelted| std::mem::take(&mut *smelted.borrow_mut()))
    {
        FurnaceRecipes::smelting().getSmeltingList().borrow_mut().remove(&input);
    }
}

pub fn itemId(id: &Identifier) -> Result<i32, String>
{
    crate::registry::with(|registries| registries.itemId(id)).ok_or_else(|| format!("{id} is not a registered item"))
}

pub fn stackOf(spec: &StackSpec) -> Result<Rc<ItemStack>, String>
{
    Ok(ItemStack::withDamage(itemId(&spec.item)?, spec.count, spec.meta))
}

pub fn validate(spec: &RecipeSpec) -> Result<(), String>
{
    if let RecipeSpec::Shaped(shaped) = spec
    {
        let width: usize = shaped.pattern.first().map_or(0, String::len);
        if shaped.pattern.is_empty() || shaped.pattern.len() > 3 || width > 3
        {
            return Err("a shaped pattern has one to three rows of one to three characters".to_owned());
        }
        if shaped.pattern.iter().any(|row| row.len() != width)
        {
            return Err("every row of a shaped pattern has the same width".to_owned());
        }
        for symbol in shaped.pattern.iter().flat_map(|row| row.chars()).filter(|symbol| *symbol != ' ')
        {
            if !shaped.key.contains_key(&symbol.to_string())
            {
                return Err(format!("pattern symbol '{symbol}' has no key"));
            }
        }
    }
    Ok(())
}

pub fn apply(spec: &RecipeSpec) -> Result<(), String>
{
    match spec
    {
        RecipeSpec::Shaped(shaped) =>
        {
            let pattern: Vec<&str> = shaped.pattern.iter().map(String::as_str).collect();
            let mut keys: Vec<(char, Ingredient)> = Vec::new();
            for (symbol, stack) in &shaped.key
            {
                let symbol: char = symbol.chars().next().ok_or("a recipe key is one character")?;
                keys.push((symbol, Ingredient::Stack(stackOf(stack)?)));
            }
            CraftingManager::getInstance().addRecipe(stackOf(&shaped.result)?, &pattern, &keys);
            trackLast();
        }
        RecipeSpec::Shapeless(shapeless) =>
        {
            let ingredients: Vec<Ingredient> = shapeless
                .ingredients
                .iter()
                .map(|stack| stackOf(stack).map(Ingredient::Stack))
                .collect::<Result<_, _>>()?;
            CraftingManager::getInstance().addShapelessRecipe(stackOf(&shapeless.result)?, &ingredients);
            trackLast();
        }
        RecipeSpec::Smelting(smelting) =>
        {
            let input: i32 = itemId(&smelting.input)?;
            if FurnaceRecipes::smelting().getSmeltingList().borrow().containsKey(&input)
            {
                crate::log::warn(
                    crate::log::LOADER,
                    &format!("smelting {} already has a result, the later recipe replaces it", smelting.input),
                );
            }
            FurnaceRecipes::smelting().addSmelting(input, stackOf(&smelting.result)?);
            SMELTED.with(|smelted| smelted.borrow_mut().push(input));
        }
        RecipeSpec::Fuel(fuel) => super::putFuel(itemId(&fuel.item)?, fuel.ticks),
    }
    Ok(())
}

// removals use the vanilla list order the registry was seeded from, so they run before any mod recipe is added
pub fn removeVanilla(indices: &mut [i32])
{
    indices.sort_unstable();
    let manager: Rc<CraftingManager> = CraftingManager::getInstance();
    for index in indices.iter().rev()
    {
        if *index < manager.getRecipeList().size()
        {
            manager.getRecipeList().removeAt(*index);
        }
    }
}

pub fn finish() { CraftingManager::getInstance().sortRecipes(); }
