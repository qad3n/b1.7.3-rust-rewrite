use crate::entity::Entity;
use crate::inventory::IInventory;
use crate::item::ItemStack;
use crate::tileentity::tile_entity_chest::{Slots, setSlot, slot};
use std::cell::RefCell;
use std::rc::Rc;

pub struct InventoryCraftResult
{
    m_stackResult: Slots,
}

impl InventoryCraftResult
{
    pub fn new() -> InventoryCraftResult { InventoryCraftResult { m_stackResult: RefCell::new(vec![None; 1]) } }
}

impl Default for InventoryCraftResult
{
    fn default() -> Self { Self::new() }
}

impl IInventory for InventoryCraftResult
{
    fn getSizeInventory(&self) -> i32 { 1 }

    fn getStackInSlot(&self, index: i32) -> Option<Rc<ItemStack>> { slot(&self.m_stackResult, index) }

    fn decrStackSize(&self, index: i32, _amount: i32) -> Option<Rc<ItemStack>>
    {
        let itemstack: Rc<ItemStack> = slot(&self.m_stackResult, index)?;
        setSlot(&self.m_stackResult, index, None);
        Some(itemstack)
    }

    fn setInventorySlotContents(&self, index: i32, item: Option<Rc<ItemStack>>)
    {
        setSlot(&self.m_stackResult, index, item);
    }

    fn getInvName(&self) -> String { "Result".to_string() }

    fn getInventoryStackLimit(&self) -> i32 { 64 }

    fn onInventoryChanged(&self) {}

    fn canInteractWith(&self, _player: &Rc<Entity>) -> bool { true }
}
