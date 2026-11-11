use crate::content::recipes;
use crate::host::context::stackData;
use crate::registry::{EntityEntry, ItemEntry, Registries, Row, UNASSIGNED};
use game::crafting::{CraftingManager, FurnaceRecipes, IRecipe};
use game::item::{Item, ItemStack};
use game::locale::StatCollector;
use mod_protocol::Identifier;
use mod_protocol::abi::{RecipeShape, RecipeView, StackData};
use std::rc::Rc;

pub mod loot;
pub mod machines;

// block items nobody holds: flowing liquids, fire, technical blocks and the blocks a separate item places
const HIDDEN: [&str; 22] = [
    "flowing_water",
    "flowing_lava",
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

struct Variants
{
    m_name: &'static str,
    m_count: i32,
    // one name per damage where beta shares one name or has no lang line, empty where vanilla names each damage
    m_labels: &'static [&'static str],
}

// vanilla items whose damage values are distinct items or whose name beta never translated
const VARIANTS: [Variants; 9] = [
    Variants { m_name: "sapling", m_count: 3, m_labels: &["Oak Sapling", "Spruce Sapling", "Birch Sapling"] },
    Variants { m_name: "log", m_count: 3, m_labels: &["Oak Wood", "Spruce Wood", "Birch Wood"] },
    Variants { m_name: "leaves", m_count: 3, m_labels: &["Oak Leaves", "Spruce Leaves", "Birch Leaves"] },
    Variants { m_name: "tallgrass", m_count: 3, m_labels: &["Shrub", "Tall Grass", "Fern"] },
    Variants { m_name: "deadbush", m_count: 1, m_labels: &["Dead Bush"] },
    Variants { m_name: "wool", m_count: 16, m_labels: &[] },
    Variants { m_name: "stone_slab", m_count: 4, m_labels: &[] },
    Variants { m_name: "coal", m_count: 2, m_labels: &[] },
    Variants { m_name: "dye", m_count: 16, m_labels: &[] },
];

const ANY_DAMAGE: i32 = -1;

// the vanilla spawner keys of living entities, the rest of EntityList are projectiles, drops and vehicles
const VANILLA_MOBS: [&str; 15] = [
    "Monster",
    "Creeper",
    "Skeleton",
    "Spider",
    "Giant",
    "Zombie",
    "Slime",
    "Ghast",
    "PigZombie",
    "Pig",
    "Sheep",
    "Cow",
    "Chicken",
    "Squid",
    "Wolf",
];

fn variants(id: &Identifier) -> Option<&'static Variants>
{
    VARIANTS.iter().find(|row| id.isVanilla() && id.path() == row.m_name)
}

fn subtypes(id: &Identifier) -> i32 { variants(id).map_or(1, |row| row.m_count) }

// a registry path such as dead_bush read as Dead Bush
fn humanize(path: &str) -> String
{
    let words: Vec<String> = path
        .split('_')
        .filter(|word| !word.is_empty())
        .map(|word|
        {
            let mut chars: std::str::Chars = word.chars();
            chars.next().map_or_else(String::new, |first| first.to_uppercase().chain(chars).collect())
        })
        .collect();
    words.join(" ")
}

// the name every damage shows, beta's shared or missing names come from the variants table or the registry path
pub fn displayName(stack: &Rc<ItemStack>) -> String
{
    let id: Option<Identifier> = crate::registry::with(|registries|
    {
        registries.m_items.byNumeric(stack.m_itemID.get()).map(|row| row.m_id.clone())
    });

    let damage: Option<usize> = usize::try_from(stack.getItemDamage()).ok();
    let label: Option<&str> =
        id.as_ref().and_then(variants).zip(damage).and_then(|(row, damage)| row.m_labels.get(damage).copied());

    if let Some(label) = label
    {
        return label.to_owned();
    }

    let Some(name) = stack.getItemName()
    else
    {
        return String::new();
    };

    let key: String = format!("{name}.name");
    let translated: String = StatCollector::translateToLocal(&key);

    match id
    {
        Some(id) if translated == key => humanize(id.path()),
        _ => translated,
    }
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
    crate::registry::with(|registries|
    {
        let mut rows: Vec<&Row<ItemEntry>> = registries.m_items.rows().filter(|row| listed(registries, row)).collect();
        rows.sort_by_key(|row| row.m_numeric);
        rows.into_iter()
            .flat_map(|row|
            {
                let count: i32 = Item::byId(row.m_numeric).map_or(1, |item| item.getItemStackLimit());
                (0..subtypes(&row.m_id)).map(move |damage| StackData { id: row.m_numeric, count, damage })
            })
            .collect()
    })
}

fn mob(row: &Row<EntityEntry>) -> bool
{
    match &row.m_value.m_spec
    {
        Some(spec) => crate::entities::living(spec),
        None => VANILLA_MOBS.contains(&row.m_value.m_saveKey.as_str()),
    }
}

pub fn mobs() -> Vec<Identifier>
{
    crate::registry::with(|registries|
    {
        let mut rows: Vec<&Row<EntityEntry>> =
            registries.m_entities.rows().filter(|row| row.m_numeric != UNASSIGNED && mob(row)).collect();
        rows.sort_by_key(|row| row.m_numeric);
        rows.into_iter().map(|row| row.m_id.clone()).collect()
    })
}

fn crafting(recipe: &Rc<dyn IRecipe>) -> RecipeView
{
    let output: StackData = stackData(&recipe.getRecipeOutput());
    let (width, inputs): (i32, Vec<Option<Rc<ItemStack>>>) = recipe.getRecipeGrid();
    let shape: RecipeShape = match width
    {
        0 => RecipeShape::Shapeless { inputs: inputs.iter().flatten().map(|stack| stackData(stack)).collect() },
        _ => RecipeShape::Shaped
        {
            width,
            inputs: inputs.iter().map(|stack| stack.as_ref().map(|stack| stackData(stack))).collect(),
        },
    };
    RecipeView { id: recipes::craftingName(recipe), output, shape }
}

// a mod recipe for the input wins over the vanilla one it replaced in the furnace table
fn smeltingName(input: i32) -> Option<Identifier>
{
    recipes::smeltingName(input).or_else(||
    {
        crate::registry::with(|registries|
        {
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

    recipes.extend(smelting.into_iter().map(|(input, result)| RecipeView
    {
        id: smeltingName(input),
        output: stackData(&result),
        shape: RecipeShape::Smelting { input: StackData { id: input, count: 1, damage: ANY_DAMAGE } },
    }));

    recipes
}
