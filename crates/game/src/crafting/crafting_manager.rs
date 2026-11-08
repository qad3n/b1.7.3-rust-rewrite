use crate::block::Block;
use crate::crafting::{CraftingGrid, IRecipe, ShapedRecipes, ShapelessRecipes};
use crate::item::{Item, ItemStack};
use jvm::array_list::JArrayList;
use std::cell::OnceCell;
use std::collections::HashMap;
use std::rc::Rc;

thread_local! {
    static INSTANCE: OnceCell<Rc<CraftingManager>> = const { OnceCell::new() };
}

#[derive(Clone)]
pub enum Ingredient
{
    Item(Rc<Item>),
    Block(Rc<Block>),
    Stack(Rc<ItemStack>),
}

pub struct CraftingManager
{
    m_recipes: Rc<JArrayList<Rc<dyn IRecipe>>>,
}

fn compareRecipes(recipe: &Rc<dyn IRecipe>, recipe2: &Rc<dyn IRecipe>) -> i32
{
    if recipe.instanceOf("ShapelessRecipes") && recipe2.instanceOf("ShapedRecipes")
    {
        return 1;
    }
    if recipe2.instanceOf("ShapelessRecipes") && recipe.instanceOf("ShapedRecipes")
    {
        return -1;
    }
    if recipe2.getRecipeSize() < recipe.getRecipeSize()
    {
        return -1;
    }
    if recipe2.getRecipeSize() > recipe.getRecipeSize()
    {
        return 1;
    }
    0
}

impl CraftingManager
{
    pub fn getInstance() -> Rc<CraftingManager>
    {
        INSTANCE.with(|instance| Rc::clone(instance.get_or_init(|| Rc::new(CraftingManager::new()))))
    }

    fn new() -> CraftingManager
    {
        let manager: CraftingManager = CraftingManager { m_recipes: Rc::new(JArrayList::new()) };
        super::recipes_tools::addRecipes(&manager);
        super::recipes_weapons::addRecipes(&manager);
        super::recipes_ingots::addRecipes(&manager);
        super::recipes_food::addRecipes(&manager);
        super::recipes_crafting::addRecipes(&manager);
        super::recipes_armor::addRecipes(&manager);
        super::recipes_dyes::addRecipes(&manager);
        manager.addRecipe(
            ItemStack::withItemCount(&Item::paper(), 3),
            &["###"],
            &[('#', Ingredient::Item(Item::reed()))],
        );
        manager.addRecipe(
            ItemStack::withItemCount(&Item::book(), 1),
            &["#", "#", "#"],
            &[('#', Ingredient::Item(Item::paper()))],
        );
        manager.addRecipe(
            ItemStack::withBlockCount(&Block::fence(), 2),
            &["###", "###"],
            &[('#', Ingredient::Item(Item::stick()))],
        );
        manager.addRecipe(
            ItemStack::withBlockCount(&Block::jukebox(), 1),
            &["###", "#X#", "###"],
            &[('#', Ingredient::Block(Block::planks())), ('X', Ingredient::Item(Item::diamond()))],
        );
        manager.addRecipe(
            ItemStack::withBlockCount(&Block::musicBlock(), 1),
            &["###", "#X#", "###"],
            &[('#', Ingredient::Block(Block::planks())), ('X', Ingredient::Item(Item::redstone()))],
        );
        manager.addRecipe(
            ItemStack::withBlockCount(&Block::bookShelf(), 1),
            &["###", "XXX", "###"],
            &[('#', Ingredient::Block(Block::planks())), ('X', Ingredient::Item(Item::book()))],
        );
        manager.addRecipe(
            ItemStack::withBlockCount(&Block::blockSnow(), 1),
            &["##", "##"],
            &[('#', Ingredient::Item(Item::snowball()))],
        );
        manager.addRecipe(
            ItemStack::withBlockCount(&Block::blockClay(), 1),
            &["##", "##"],
            &[('#', Ingredient::Item(Item::clay()))],
        );
        manager.addRecipe(
            ItemStack::withBlockCount(&Block::brick(), 1),
            &["##", "##"],
            &[('#', Ingredient::Item(Item::brick()))],
        );
        manager.addRecipe(
            ItemStack::withBlockCount(&Block::glowStone(), 1),
            &["##", "##"],
            &[('#', Ingredient::Item(Item::lightStoneDust()))],
        );
        manager.addRecipe(
            ItemStack::withBlockCount(&Block::cloth(), 1),
            &["##", "##"],
            &[('#', Ingredient::Item(Item::silk()))],
        );
        manager.addRecipe(
            ItemStack::withBlockCount(&Block::tnt(), 1),
            &["X#X", "#X#", "X#X"],
            &[('X', Ingredient::Item(Item::gunpowder())), ('#', Ingredient::Block(Block::sand()))],
        );
        manager.addRecipe(
            ItemStack::withBlockDamage(&Block::stairSingle(), 3, 3),
            &["###"],
            &[('#', Ingredient::Block(Block::cobblestone()))],
        );
        manager.addRecipe(
            ItemStack::withBlockDamage(&Block::stairSingle(), 3, 0),
            &["###"],
            &[('#', Ingredient::Block(Block::stone()))],
        );
        manager.addRecipe(
            ItemStack::withBlockDamage(&Block::stairSingle(), 3, 1),
            &["###"],
            &[('#', Ingredient::Block(Block::sandStone()))],
        );
        manager.addRecipe(
            ItemStack::withBlockDamage(&Block::stairSingle(), 3, 2),
            &["###"],
            &[('#', Ingredient::Block(Block::planks()))],
        );
        manager.addRecipe(
            ItemStack::withBlockCount(&Block::ladder(), 2),
            &["# #", "###", "# #"],
            &[('#', Ingredient::Item(Item::stick()))],
        );
        manager.addRecipe(
            ItemStack::withItemCount(&Item::doorWood(), 1),
            &["##", "##", "##"],
            &[('#', Ingredient::Block(Block::planks()))],
        );
        manager.addRecipe(
            ItemStack::withBlockCount(&Block::trapdoor(), 2),
            &["###", "###"],
            &[('#', Ingredient::Block(Block::planks()))],
        );
        manager.addRecipe(
            ItemStack::withItemCount(&Item::doorSteel(), 1),
            &["##", "##", "##"],
            &[('#', Ingredient::Item(Item::ingotIron()))],
        );
        manager.addRecipe(
            ItemStack::withItemCount(&Item::sign(), 1),
            &["###", "###", " X "],
            &[('#', Ingredient::Block(Block::planks())), ('X', Ingredient::Item(Item::stick()))],
        );
        manager.addRecipe(
            ItemStack::withItemCount(&Item::cake(), 1),
            &["AAA", "BEB", "CCC"],
            &[
                ('A', Ingredient::Item(Item::bucketMilk())),
                ('B', Ingredient::Item(Item::sugar())),
                ('C', Ingredient::Item(Item::wheat())),
                ('E', Ingredient::Item(Item::egg())),
            ],
        );
        manager.addRecipe(
            ItemStack::withItemCount(&Item::sugar(), 1),
            &["#"],
            &[('#', Ingredient::Item(Item::reed()))],
        );
        manager.addRecipe(
            ItemStack::withBlockCount(&Block::planks(), 4),
            &["#"],
            &[('#', Ingredient::Block(Block::wood()))],
        );
        manager.addRecipe(
            ItemStack::withItemCount(&Item::stick(), 4),
            &["#", "#"],
            &[('#', Ingredient::Block(Block::planks()))],
        );
        manager.addRecipe(
            ItemStack::withBlockCount(&Block::torchWood(), 4),
            &["X", "#"],
            &[('X', Ingredient::Item(Item::coal())), ('#', Ingredient::Item(Item::stick()))],
        );
        manager.addRecipe(
            ItemStack::withBlockCount(&Block::torchWood(), 4),
            &["X", "#"],
            &[
                ('X', Ingredient::Stack(ItemStack::withItemDamage(&Item::coal(), 1, 1))),
                ('#', Ingredient::Item(Item::stick())),
            ],
        );
        manager.addRecipe(
            ItemStack::withItemCount(&Item::bowlEmpty(), 4),
            &["# #", " # "],
            &[('#', Ingredient::Block(Block::planks()))],
        );
        manager.addRecipe(
            ItemStack::withBlockCount(&Block::rail(), 16),
            &["X X", "X#X", "X X"],
            &[('X', Ingredient::Item(Item::ingotIron())), ('#', Ingredient::Item(Item::stick()))],
        );
        manager.addRecipe(
            ItemStack::withBlockCount(&Block::railPowered(), 6),
            &["X X", "X#X", "XRX"],
            &[
                ('X', Ingredient::Item(Item::ingotGold())),
                ('R', Ingredient::Item(Item::redstone())),
                ('#', Ingredient::Item(Item::stick())),
            ],
        );
        manager.addRecipe(
            ItemStack::withBlockCount(&Block::railDetector(), 6),
            &["X X", "X#X", "XRX"],
            &[
                ('X', Ingredient::Item(Item::ingotIron())),
                ('R', Ingredient::Item(Item::redstone())),
                ('#', Ingredient::Block(Block::pressurePlateStone())),
            ],
        );
        manager.addRecipe(
            ItemStack::withItemCount(&Item::minecartEmpty(), 1),
            &["# #", "###"],
            &[('#', Ingredient::Item(Item::ingotIron()))],
        );
        manager.addRecipe(
            ItemStack::withBlockCount(&Block::pumpkinLantern(), 1),
            &["A", "B"],
            &[('A', Ingredient::Block(Block::pumpkin())), ('B', Ingredient::Block(Block::torchWood()))],
        );
        manager.addRecipe(
            ItemStack::withItemCount(&Item::minecartCrate(), 1),
            &["A", "B"],
            &[('A', Ingredient::Block(Block::chest())), ('B', Ingredient::Item(Item::minecartEmpty()))],
        );
        manager.addRecipe(
            ItemStack::withItemCount(&Item::minecartPowered(), 1),
            &["A", "B"],
            &[('A', Ingredient::Block(Block::stoneOvenIdle())), ('B', Ingredient::Item(Item::minecartEmpty()))],
        );
        manager.addRecipe(
            ItemStack::withItemCount(&Item::boat(), 1),
            &["# #", "###"],
            &[('#', Ingredient::Block(Block::planks()))],
        );
        manager.addRecipe(
            ItemStack::withItemCount(&Item::bucketEmpty(), 1),
            &["# #", " # "],
            &[('#', Ingredient::Item(Item::ingotIron()))],
        );
        manager.addRecipe(
            ItemStack::withItemCount(&Item::flintAndSteel(), 1),
            &["A ", " B"],
            &[('A', Ingredient::Item(Item::ingotIron())), ('B', Ingredient::Item(Item::flint()))],
        );
        manager.addRecipe(
            ItemStack::withItemCount(&Item::bread(), 1),
            &["###"],
            &[('#', Ingredient::Item(Item::wheat()))],
        );
        manager.addRecipe(
            ItemStack::withBlockCount(&Block::stairCompactPlanks(), 4),
            &["#  ", "## ", "###"],
            &[('#', Ingredient::Block(Block::planks()))],
        );
        manager.addRecipe(
            ItemStack::withItemCount(&Item::fishingRod(), 1),
            &["  #", " #X", "# X"],
            &[('#', Ingredient::Item(Item::stick())), ('X', Ingredient::Item(Item::silk()))],
        );
        manager.addRecipe(
            ItemStack::withBlockCount(&Block::stairCompactCobblestone(), 4),
            &["#  ", "## ", "###"],
            &[('#', Ingredient::Block(Block::cobblestone()))],
        );
        manager.addRecipe(
            ItemStack::withItemCount(&Item::painting(), 1),
            &["###", "#X#", "###"],
            &[('#', Ingredient::Item(Item::stick())), ('X', Ingredient::Block(Block::cloth()))],
        );
        manager.addRecipe(
            ItemStack::withItemCount(&Item::appleGold(), 1),
            &["###", "#X#", "###"],
            &[('#', Ingredient::Block(Block::blockGold())), ('X', Ingredient::Item(Item::appleRed()))],
        );
        manager.addRecipe(
            ItemStack::withBlockCount(&Block::lever(), 1),
            &["X", "#"],
            &[('#', Ingredient::Block(Block::cobblestone())), ('X', Ingredient::Item(Item::stick()))],
        );
        manager.addRecipe(
            ItemStack::withBlockCount(&Block::torchRedstoneActive(), 1),
            &["X", "#"],
            &[('#', Ingredient::Item(Item::stick())), ('X', Ingredient::Item(Item::redstone()))],
        );
        manager.addRecipe(
            ItemStack::withItemCount(&Item::redstoneRepeater(), 1),
            &["#X#", "III"],
            &[
                ('#', Ingredient::Block(Block::torchRedstoneActive())),
                ('X', Ingredient::Item(Item::redstone())),
                ('I', Ingredient::Block(Block::stone())),
            ],
        );
        manager.addRecipe(
            ItemStack::withItemCount(&Item::pocketSundial(), 1),
            &[" # ", "#X#", " # "],
            &[('#', Ingredient::Item(Item::ingotGold())), ('X', Ingredient::Item(Item::redstone()))],
        );
        manager.addRecipe(
            ItemStack::withItemCount(&Item::compass(), 1),
            &[" # ", "#X#", " # "],
            &[('#', Ingredient::Item(Item::ingotIron())), ('X', Ingredient::Item(Item::redstone()))],
        );
        manager.addRecipe(
            ItemStack::withItemCount(&Item::mapItem(), 1),
            &["###", "#X#", "###"],
            &[('#', Ingredient::Item(Item::paper())), ('X', Ingredient::Item(Item::compass()))],
        );
        manager.addRecipe(
            ItemStack::withBlockCount(&Block::button(), 1),
            &["#", "#"],
            &[('#', Ingredient::Block(Block::stone()))],
        );
        manager.addRecipe(
            ItemStack::withBlockCount(&Block::pressurePlateStone(), 1),
            &["##"],
            &[('#', Ingredient::Block(Block::stone()))],
        );
        manager.addRecipe(
            ItemStack::withBlockCount(&Block::pressurePlatePlanks(), 1),
            &["##"],
            &[('#', Ingredient::Block(Block::planks()))],
        );
        manager.addRecipe(
            ItemStack::withBlockCount(&Block::dispenser(), 1),
            &["###", "#X#", "#R#"],
            &[
                ('#', Ingredient::Block(Block::cobblestone())),
                ('X', Ingredient::Item(Item::bow())),
                ('R', Ingredient::Item(Item::redstone())),
            ],
        );
        manager.addRecipe(
            ItemStack::withBlockCount(&Block::pistonBase(), 1),
            &["TTT", "#X#", "#R#"],
            &[
                ('#', Ingredient::Block(Block::cobblestone())),
                ('X', Ingredient::Item(Item::ingotIron())),
                ('R', Ingredient::Item(Item::redstone())),
                ('T', Ingredient::Block(Block::planks())),
            ],
        );
        manager.addRecipe(
            ItemStack::withBlockCount(&Block::pistonStickyBase(), 1),
            &["S", "P"],
            &[('S', Ingredient::Item(Item::slimeBall())), ('P', Ingredient::Block(Block::pistonBase()))],
        );
        manager.addRecipe(
            ItemStack::withItemCount(&Item::bed(), 1),
            &["###", "XXX"],
            &[('#', Ingredient::Block(Block::cloth())), ('X', Ingredient::Block(Block::planks()))],
        );
        let mut recipes: Vec<Rc<dyn IRecipe>> = manager.m_recipes.toVec();
        jvm::sort::sort(&mut recipes, &mut compareRecipes);
        manager.m_recipes.replaceAll(recipes);
        jvm::system::println(&format!("{} recipes", manager.m_recipes.size()));
        manager
    }

    pub(super) fn addRecipe(&self, result: Rc<ItemStack>, pattern: &[&str], keys: &[(char, Ingredient)])
    {
        let mut s: String = String::new();
        let mut width: i32 = 0;
        let mut height: i32 = 0;
        for line in pattern
        {
            height += 1;
            width = line.len() as i32;
            s.push_str(line);
        }

        let mut hashMap: HashMap<char, Rc<ItemStack>> = HashMap::new();
        for (c, ingredient) in keys
        {
            let stack: Rc<ItemStack> = match ingredient
            {
                Ingredient::Item(item) => ItemStack::withItem(item),
                Ingredient::Block(block) => ItemStack::withBlockDamage(block, 1, -1),
                Ingredient::Stack(stack) => Rc::clone(stack),
            };
            hashMap.insert(*c, stack);
        }

        let mut ingredients: Vec<Option<Rc<ItemStack>>> = Vec::with_capacity((width * height) as usize);
        for k in 0..width * height
        {
            let char1: char = s.chars().nth(k as usize).unwrap_or_else(|| {
                jvm::throwable::throw(&jvm::throwable::STRING_INDEX, &format!("String index out of range: {k}"))
            });
            ingredients.push(hashMap.get(&char1).map(|stack| stack.copy()));
        }

        self.m_recipes.add(Rc::new(ShapedRecipes::new(width, height, ingredients, result)));
    }

    pub(super) fn addShapelessRecipe(&self, result: Rc<ItemStack>, args: &[Ingredient])
    {
        let mut ingredients: Vec<Rc<ItemStack>> = Vec::new();
        for o in args
        {
            let stack: Rc<ItemStack> = match o
            {
                Ingredient::Stack(stack) => stack.copy(),
                Ingredient::Item(item) => ItemStack::withItem(item),
                Ingredient::Block(block) => ItemStack::withBlock(block),
            };
            ingredients.push(stack);
        }

        self.m_recipes.add(Rc::new(ShapelessRecipes::new(result, ingredients)));
    }

    pub fn findMatchingRecipe(&self, inventory: &dyn CraftingGrid) -> Option<Rc<ItemStack>>
    {
        for i in 0..self.m_recipes.size()
        {
            let recipe: Rc<dyn IRecipe> = self.m_recipes.get(i);
            if recipe.matches(inventory)
            {
                return Some(recipe.getCraftingResult(inventory));
            }
        }

        None
    }

    pub fn getRecipeList(&self) -> Rc<JArrayList<Rc<dyn IRecipe>>> { Rc::clone(&self.m_recipes) }
}
