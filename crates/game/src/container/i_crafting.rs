use crate::container::Container;
use crate::entity::Entity;
use crate::item::ItemStack;
use jvm::object::JavaObject;
use std::rc::Rc;

pub trait ICrafting
{
    fn updateCraftingInventory(&self, container: &Rc<Container>, items: &[Option<Rc<ItemStack>>]);

    fn onSlotChanged(&self, container: &Rc<Container>, slot: i32, item: Option<Rc<ItemStack>>);

    fn onDataChanged(&self, container: &Rc<Container>, id: i32, value: i32);

    fn entity(&self) -> Rc<Entity>;
}

impl JavaObject for dyn ICrafting
{
    fn javaEquals(&self, other: &Self) -> bool { self.entity().javaEquals(&other.entity()) }

    fn javaHash(&self) -> i32 { self.entity().javaHash() }
}
