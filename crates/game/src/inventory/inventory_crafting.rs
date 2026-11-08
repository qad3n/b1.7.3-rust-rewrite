use crate::container::Container;
use crate::crafting::CraftingGrid;
use crate::entity::Entity;
use crate::inventory::IInventory;
use crate::item::ItemStack;
use crate::tileentity::tile_entity_chest::{Slots, setSlot, slot};
use std::cell::RefCell;
use std::rc::{Rc, Weak};

pub struct InventoryCrafting
{
    m_stackList: Slots,
    m_size: i32,
    m_eventHandler: Weak<Container>,
}

impl InventoryCrafting
{
    pub fn new(menu: &Weak<Container>, width: i32, height: i32) -> InventoryCrafting
    {
        let n: i32 = width * height;
        InventoryCrafting {
            m_stackList: RefCell::new(vec![None; n as usize]),
            m_size: width,
            m_eventHandler: Weak::clone(menu),
        }
    }

    fn eventHandler(&self) -> Rc<Container>
    {
        self.m_eventHandler.upgrade().expect("the container owns its crafting matrix")
    }
}

impl CraftingGrid for InventoryCrafting
{
    fn getItem(&self, column: i32, row: i32) -> Option<Rc<ItemStack>>
    {
        if column < 0 || column >= self.m_size
        {
            return None;
        }
        let n: i32 = column + row * self.m_size;
        self.getStackInSlot(n)
    }
}

impl IInventory for InventoryCrafting
{
    fn getSizeInventory(&self) -> i32 { self.m_stackList.borrow().len() as i32 }

    fn getStackInSlot(&self, index: i32) -> Option<Rc<ItemStack>>
    {
        if index >= self.getSizeInventory()
        {
            return None;
        }
        slot(&self.m_stackList, index)
    }

    fn decrStackSize(&self, index: i32, amount: i32) -> Option<Rc<ItemStack>>
    {
        let current: Rc<ItemStack> = slot(&self.m_stackList, index)?;
        if current.m_stackSize.get() <= amount
        {
            setSlot(&self.m_stackList, index, None);
            self.eventHandler().onCraftMatrixChanged(self);
            return Some(current);
        }
        let itemstack: Rc<ItemStack> = current.splitStack(amount);
        if current.m_stackSize.get() == 0
        {
            setSlot(&self.m_stackList, index, None);
        }
        self.eventHandler().onCraftMatrixChanged(self);
        Some(itemstack)
    }

    fn setInventorySlotContents(&self, index: i32, item: Option<Rc<ItemStack>>)
    {
        setSlot(&self.m_stackList, index, item);
        self.eventHandler().onCraftMatrixChanged(self);
    }

    fn getInvName(&self) -> String { "Crafting".to_string() }

    fn getInventoryStackLimit(&self) -> i32 { 64 }

    fn onInventoryChanged(&self) {}

    fn canInteractWith(&self, _player: &Rc<Entity>) -> bool { true }
}
