use crate::entity::Entity;
use crate::inventory::{IInvBasic, IInventory};
use crate::item::ItemStack;
use crate::tileentity::tile_entity_chest::{Slots, setSlot, slot};
use jvm::array_list::JArrayList;
use jvm::string::JString;
use std::cell::RefCell;
use std::rc::Rc;

pub struct InventoryBasic
{
    m_inventoryTitle: JString,
    m_slotsCount: i32,
    m_inventoryContents: Slots,
    m_listeners: Option<JArrayList<Rc<dyn IInvBasic>>>,
}

impl InventoryBasic
{
    pub fn new(name: JString, size: i32) -> InventoryBasic
    {
        if size < 0
        {
            jvm::throwable::throwBare(&jvm::throwable::NEGATIVE_ARRAY_SIZE);
        }
        InventoryBasic {
            m_inventoryTitle: name,
            m_slotsCount: size,
            m_inventoryContents: RefCell::new(vec![None; size as usize]),
            m_listeners: None,
        }
    }
}

impl IInventory for InventoryBasic
{
    fn getSizeInventory(&self) -> i32 { self.m_slotsCount }

    fn getStackInSlot(&self, index: i32) -> Option<Rc<ItemStack>> { slot(&self.m_inventoryContents, index) }

    fn decrStackSize(&self, index: i32, amount: i32) -> Option<Rc<ItemStack>>
    {
        let current: Rc<ItemStack> = slot(&self.m_inventoryContents, index)?;
        if current.m_stackSize.get() <= amount
        {
            setSlot(&self.m_inventoryContents, index, None);
            self.onInventoryChanged();
            return Some(current);
        }
        let itemStack: Rc<ItemStack> = current.splitStack(amount);
        if current.m_stackSize.get() == 0
        {
            setSlot(&self.m_inventoryContents, index, None);
        }
        self.onInventoryChanged();
        Some(itemStack)
    }

    fn setInventorySlotContents(&self, index: i32, item: Option<Rc<ItemStack>>)
    {
        setSlot(&self.m_inventoryContents, index, item.clone());
        if let Some(item) = item
            && item.m_stackSize.get() > self.getInventoryStackLimit()
        {
            item.m_stackSize.set(self.getInventoryStackLimit());
        }
        self.onInventoryChanged();
    }

    fn getInvName(&self) -> String { self.m_inventoryTitle.to_string() }

    fn getInventoryStackLimit(&self) -> i32 { 64 }

    fn onInventoryChanged(&self)
    {
        if let Some(listeners) = &self.m_listeners
        {
            let mut i: i32 = 0;
            while i < listeners.size()
            {
                listeners.get(i).onInventoryChanged(self);
                i += 1;
            }
        }
    }

    fn canInteractWith(&self, _player: &Rc<Entity>) -> bool { true }
}
