use crate::entity::Entity;
use crate::item::ItemStack;
use std::rc::Rc;

pub trait IInventory
{
    fn getSizeInventory(&self) -> i32;

    fn getStackInSlot(&self, slot: i32) -> Option<Rc<ItemStack>>;

    fn decrStackSize(&self, slot: i32, amount: i32) -> Option<Rc<ItemStack>>;

    fn setInventorySlotContents(&self, slot: i32, item: Option<Rc<ItemStack>>);

    fn getInvName(&self) -> String;

    fn getInventoryStackLimit(&self) -> i32;

    fn onInventoryChanged(&self);

    fn canInteractWith(&self, player: &Rc<Entity>) -> bool;
}
