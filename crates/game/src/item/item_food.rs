use crate::item::{ITEM, Item, ItemClass, ItemStack};
use std::any::Any;
use std::rc::Rc;

pub struct FoodExt
{
    m_healAmount: i32,
    m_isWolfsFavoriteMeat: bool,
}

pub fn getHealAmount(this: &Item) -> i32 { this.ext::<FoodExt>().m_healAmount }

pub fn getIsWolfsFavoriteMeat(this: &Item) -> bool { this.ext::<FoodExt>().m_isWolfsFavoriteMeat }

pub fn onItemRightClick(
    this: &Rc<Item>,
    stack: &Rc<ItemStack>,
    _world: &Rc<crate::world::World>,
    player: &Rc<crate::entity::Entity>,
) -> Option<Rc<ItemStack>>
{
    stack.m_stackSize.set(stack.m_stackSize.get() - 1);
    player.heal(getHealAmount(this));
    Some(Rc::clone(stack))
}

pub const CLASS: ItemClass = ItemClass { m_name: "ItemFood", m_parent: Some(&ITEM), onItemRightClick, ..ITEM };

pub fn construct(class: &'static ItemClass, id: i32, hungerPoints: i32, canBeCooked: bool) -> Rc<Item>
{
    let ext: Box<dyn Any> = Box::new(FoodExt { m_healAmount: hungerPoints, m_isWolfsFavoriteMeat: canBeCooked });
    let item: Rc<Item> = Item::construct(class, ext, id);
    item.m_maxStackSize.set(1);
    item
}

pub fn create(id: i32, hungerPoints: i32, canBeCooked: bool) -> Rc<Item>
{
    construct(&CLASS, id, hungerPoints, canBeCooked)
}
