use crate::inventory::IInventory;
use crate::item::ItemStack;
use std::any::Any;
use std::cell::Cell;
use std::rc::Rc;

crate::vtable! {
    SlotClass for Slot {
        fn getSlotStackLimit() -> i32;
        fn isItemValid(item: &Rc<ItemStack>) -> bool;
        fn onPickupFromSlot(item: Option<Rc<ItemStack>>);
    }
}

pub const SLOT: SlotClass = SlotClass {
    m_name: "Slot",
    m_parent: None,
    m_interfaces: &[],
    getSlotStackLimit: |this| this.m_inventory.getInventoryStackLimit(),
    isItemValid: |_, _| true,
    onPickupFromSlot: |this, _| this.onSlotChanged(),
};

pub struct Slot
{
    pub m_class: &'static SlotClass,
    m_ext: Box<dyn Any>,
    m_slotIndex: i32,
    m_inventory: Rc<dyn IInventory>,
    pub m_slotNumber: Cell<i32>,
    pub m_xDisplayPosition: Cell<i32>,
    pub m_yDisplayPosition: Cell<i32>,
}

impl Slot
{
    pub fn new(inventory: Rc<dyn IInventory>, id: i32, x: i32, y: i32) -> Rc<Slot>
    {
        Slot::construct(&SLOT, Box::new(()), inventory, id, x, y)
    }

    pub fn construct(
        class: &'static SlotClass,
        ext: Box<dyn Any>,
        inventory: Rc<dyn IInventory>,
        id: i32,
        x: i32,
        y: i32,
    ) -> Rc<Slot>
    {
        Rc::new(Slot {
            m_class: class,
            m_ext: ext,
            m_slotIndex: id,
            m_inventory: inventory,
            m_slotNumber: Cell::new(0),
            m_xDisplayPosition: Cell::new(x),
            m_yDisplayPosition: Cell::new(y),
        })
    }

    pub fn ext<T: 'static>(&self) -> &T { self.m_ext.downcast_ref::<T>().expect("slot ext matches its class") }

    pub fn isHere(&self, inventory: &Rc<dyn IInventory>, slot: i32) -> bool
    {
        std::ptr::addr_eq(Rc::as_ptr(inventory), Rc::as_ptr(&self.m_inventory)) && slot == self.m_slotIndex
    }

    pub fn getStack(&self) -> Option<Rc<ItemStack>> { self.m_inventory.getStackInSlot(self.m_slotIndex) }

    pub fn getHasStack(&self) -> bool { self.getStack().is_some() }

    pub fn putStack(&self, item: Option<Rc<ItemStack>>)
    {
        self.m_inventory.setInventorySlotContents(self.m_slotIndex, item);
        self.onSlotChanged();
    }

    pub fn onSlotChanged(&self) { self.m_inventory.onInventoryChanged(); }

    pub fn getBackgroundIconIndex(&self) -> i32 { -1 }

    pub fn decrStackSize(&self, amount: i32) -> Option<Rc<ItemStack>>
    {
        self.m_inventory.decrStackSize(self.m_slotIndex, amount)
    }
}
