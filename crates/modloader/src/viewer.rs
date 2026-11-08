use crate::content::recipes;
use crate::host::context::stackData;
use crate::registry::{ItemEntry, Registries, Row};
use game::crafting::{CraftingManager, FurnaceRecipes, IRecipe};
use game::item::{Item, ItemStack};
use mod_protocol::Identifier;
use mod_protocol::abi::{RecipeShape, RecipeView, StackData};
use std::rc::Rc;

// block items nobody holds: liquids, fire, technical blocks and the blocks a separate item places
const HIDDEN: [&str; 24] = [
    "flowing_water",
    "water",
    "flowing_lava",
    "lava",
    "bed_block",
    "piston_head",
    "piston_extension",
    "double_stone_slab",
    "fire",
    "redstone_wire",
    "wheat_block",
    "lit_furnace",
    "standing_sign",
    "wooden_door_block",
    "wall_sign",
    "iron_door_block",
    "lit_redstone_ore",
    "unlit_redstone_torch",
    "reeds_block",
    "portal",
    "cake_block",
    "unpowered_repeater",
    "powered_repeater",
    "locked_chest",
];

struct Subtypes
{
    m_name: &'static str,
    m_count: i32,
}

// vanilla items whose damage values are distinct items
const SUBTYPES: [Subtypes; 8] = [
    Subtypes { m_name: "sapling", m_count: 3 },
    Subtypes { m_name: "log", m_count: 3 },
    Subtypes { m_name: "leaves", m_count: 3 },
    Subtypes { m_name: "tallgrass", m_count: 3 },
    Subtypes { m_name: "wool", m_count: 16 },
    Subtypes { m_name: "stone_slab", m_count: 4 },
    Subtypes { m_name: "coal", m_count: 2 },
    Subtypes { m_name: "dye", m_count: 16 },
];

const ANY_DAMAGE: i32 = -1;

fn subtypes(id: &Identifier) -> i32
{
    let row: Option<&Subtypes> = SUBTYPES.iter().find(|row| id.isVanilla() && id.path() == row.m_name);
    row.map_or(1, |row| row.m_count)
}

fn listed(registries: &Registries, row: &Row<ItemEntry>) -> bool
{
    if Item::byId(row.m_numeric).is_none() || (row.m_id.isVanilla() && HIDDEN.contains(&row.m_id.path()))
    {
        return false;
    }
    let paged: Option<bool> = row
        .m_value
        .m_block
        .as_ref()
        .and_then(|block| registries.m_blocks.get(block))
        .map(|block| block.m_value.m_page.is_some());
    !paged.unwrap_or(false)
}

pub fn itemCatalog() -> Vec<StackData>
{
    crate::registry::with(|registries| {
        let mut rows: Vec<&Row<ItemEntry>> = registries.m_items.rows().filter(|row| listed(registries, row)).collect();
        rows.sort_by_key(|row| row.m_numeric);
        rows.into_iter()
            .flat_map(|row| {
                let count: i32 = Item::byId(row.m_numeric).map_or(1, |item| item.getItemStackLimit());
                (0..subtypes(&row.m_id)).map(move |damage| StackData { id: row.m_numeric, count, damage })
            })
            .collect()
    })
}

fn crafting(recipe: &Rc<dyn IRecipe>) -> RecipeView
{
    let output: StackData = stackData(&recipe.getRecipeOutput());
    let (width, inputs): (i32, Vec<Option<Rc<ItemStack>>>) = recipe.getRecipeGrid();
    let shape: RecipeShape = match width
    {
        0 => RecipeShape::Shapeless { inputs: inputs.iter().flatten().map(|stack| stackData(stack)).collect() },
        _ => RecipeShape::Shaped {
            width,
            inputs: inputs.iter().map(|stack| stack.as_ref().map(|stack| stackData(stack))).collect(),
        },
    };
    RecipeView { id: recipes::craftingName(recipe), output, shape }
}

// a mod recipe for the input wins over the vanilla one it replaced in the furnace table
fn smeltingName(input: i32) -> Option<Identifier>
{
    recipes::smeltingName(input).or_else(|| {
        crate::registry::with(|registries| {
            registries.m_smelting.byNumeric(input).filter(|row| row.m_id.isVanilla()).map(|row| row.m_id.clone())
        })
    })
}

pub fn recipeList() -> Vec<RecipeView>
{
    let mut recipes: Vec<RecipeView> =
        CraftingManager::getInstance().getRecipeList().toVec().iter().map(crafting).collect();
    let mut smelting: Vec<(i32, Rc<ItemStack>)> = FurnaceRecipes::smelting().getSmeltingList().borrow().entries();
    smelting.sort_by_key(|(input, _)| *input);
    recipes.extend(smelting.into_iter().map(|(input, result)| RecipeView {
        id: smeltingName(input),
        output: stackData(&result),
        shape: RecipeShape::Smelting { input: StackData { id: input, count: 1, damage: ANY_DAMAGE } },
    }));
    recipes
}
