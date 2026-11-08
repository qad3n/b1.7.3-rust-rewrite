use crate::block::Block;
use crate::item::{Item, ItemStack};
use jvm::hash_map::JHashMap;
use std::cell::{OnceCell, RefCell};
use std::rc::Rc;

thread_local! {
    static SMELTING_BASE: OnceCell<Rc<FurnaceRecipes>> = const { OnceCell::new() };
}

pub struct FurnaceRecipes
{
    m_smeltingList: RefCell<JHashMap<i32, Rc<ItemStack>>>,
}

impl FurnaceRecipes
{
    pub fn smelting() -> Rc<FurnaceRecipes>
    {
        SMELTING_BASE.with(|base| Rc::clone(base.get_or_init(|| Rc::new(FurnaceRecipes::new()))))
    }

    fn new() -> FurnaceRecipes
    {
        let recipes: FurnaceRecipes = FurnaceRecipes { m_smeltingList: RefCell::new(JHashMap::new()) };
        recipes.addSmelting(Block::oreIron().m_blockID, ItemStack::withItem(&Item::ingotIron()));
        recipes.addSmelting(Block::oreGold().m_blockID, ItemStack::withItem(&Item::ingotGold()));
        recipes.addSmelting(Block::oreDiamond().m_blockID, ItemStack::withItem(&Item::diamond()));
        recipes.addSmelting(Block::sand().m_blockID, ItemStack::withBlock(&Block::glass()));
        recipes.addSmelting(Item::porkRaw().m_shiftedIndex, ItemStack::withItem(&Item::porkCooked()));
        recipes.addSmelting(Item::fishRaw().m_shiftedIndex, ItemStack::withItem(&Item::fishCooked()));
        recipes.addSmelting(Block::cobblestone().m_blockID, ItemStack::withBlock(&Block::stone()));
        recipes.addSmelting(Item::clay().m_shiftedIndex, ItemStack::withItem(&Item::brick()));
        recipes.addSmelting(Block::cactus().m_blockID, ItemStack::withItemDamage(&Item::dyePowder(), 1, 2));
        recipes.addSmelting(Block::wood().m_blockID, ItemStack::withItemDamage(&Item::coal(), 1, 1));
        recipes
    }

    pub fn addSmelting(&self, input: i32, result: Rc<ItemStack>)
    {
        self.m_smeltingList.borrow_mut().put(input, result);
    }

    pub fn getSmeltingResult(&self, input: i32) -> Option<Rc<ItemStack>> { self.m_smeltingList.borrow().get(&input) }

    pub fn getSmeltingList(&self) -> &RefCell<JHashMap<i32, Rc<ItemStack>>> { &self.m_smeltingList }
}
