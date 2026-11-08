use crate::entity::Entity;
use crate::inventory::IInventory;
use crate::item::ItemStack;
use jvm::throwable::OrNpe;
use std::rc::Rc;

pub struct InventoryLargeChest
{
    m_name: String,
    m_upperChest: Option<Rc<dyn IInventory>>,
    m_lowerChest: Option<Rc<dyn IInventory>>,
}

impl InventoryLargeChest
{
    pub fn new(
        name: &str,
        inventory1: Option<Rc<dyn IInventory>>,
        inventory2: Option<Rc<dyn IInventory>>,
    ) -> Rc<InventoryLargeChest>
    {
        Rc::new(InventoryLargeChest { m_name: name.to_string(), m_upperChest: inventory1, m_lowerChest: inventory2 })
    }

    fn upper(&self) -> &Rc<dyn IInventory> { self.m_upperChest.as_ref().orNpe() }

    fn lower(&self) -> &Rc<dyn IInventory> { self.m_lowerChest.as_ref().orNpe() }
}

impl IInventory for InventoryLargeChest
{
    fn getSizeInventory(&self) -> i32 { self.upper().getSizeInventory() + self.lower().getSizeInventory() }

    fn getInvName(&self) -> String { self.m_name.clone() }

    fn getStackInSlot(&self, slot: i32) -> Option<Rc<ItemStack>>
    {
        if slot >= self.upper().getSizeInventory()
        {
            return self.lower().getStackInSlot(slot - self.upper().getSizeInventory());
        }
        self.upper().getStackInSlot(slot)
    }

    fn decrStackSize(&self, slot: i32, amount: i32) -> Option<Rc<ItemStack>>
    {
        if slot >= self.upper().getSizeInventory()
        {
            return self.lower().decrStackSize(slot - self.upper().getSizeInventory(), amount);
        }
        self.upper().decrStackSize(slot, amount)
    }

    fn setInventorySlotContents(&self, slot: i32, item: Option<Rc<ItemStack>>)
    {
        if slot >= self.upper().getSizeInventory()
        {
            self.lower().setInventorySlotContents(slot - self.upper().getSizeInventory(), item);
        }
        else
        {
            self.upper().setInventorySlotContents(slot, item);
        }
    }

    fn getInventoryStackLimit(&self) -> i32 { self.upper().getInventoryStackLimit() }

    fn onInventoryChanged(&self)
    {
        self.upper().onInventoryChanged();
        self.lower().onInventoryChanged();
    }

    fn canInteractWith(&self, player: &Rc<Entity>) -> bool
    {
        self.upper().canInteractWith(player) && self.lower().canInteractWith(player)
    }
}
