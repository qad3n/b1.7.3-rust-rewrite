use crate::container::slot::Slot;
use crate::container::{CONTAINER, Container, ContainerClass};
use crate::entity::Entity;
use crate::inventory::IInventory;
use crate::item::ItemStack;
use jvm::throwable::OrNpe;
use std::rc::Rc;

pub struct ChestExt
{
    m_inventory: Rc<dyn IInventory>,
    m_rows: i32,
}

fn fields(this: &Container) -> &ChestExt { this.ext::<ChestExt>() }

fn isUsableByPlayer(this: &Rc<Container>, player: &Rc<Entity>) -> bool
{
    fields(this).m_inventory.canInteractWith(player)
}

fn getStackInSlot(this: &Rc<Container>, slot: i32) -> Option<Rc<ItemStack>>
{
    let mut itemStack: Option<Rc<ItemStack>> = None;
    let rows: i32 = fields(this).m_rows;
    let slot2: Rc<Slot> = this.m_slots.get(slot);
    if slot2.getHasStack()
    {
        let itemStack2: Rc<ItemStack> = slot2.getStack().orNpe();
        itemStack = Some(itemStack2.copy());
        if slot < rows * 9
        {
            this.moveItem(&itemStack2, rows * 9, this.m_slots.size(), true);
        }
        else
        {
            this.moveItem(&itemStack2, 0, rows * 9, false);
        }
        if itemStack2.m_stackSize.get() == 0
        {
            slot2.putStack(None);
        }
        else
        {
            slot2.onSlotChanged();
        }
    }
    itemStack
}

pub const CLASS: ContainerClass = ContainerClass {
    m_name: "ContainerChest",
    m_parent: Some(&CONTAINER),
    m_interfaces: &[],
    getStackInSlot,
    isUsableByPlayer,
    ..CONTAINER
};

pub fn create(playerInventory: Rc<dyn IInventory>, inventory: Rc<dyn IInventory>) -> Rc<Container>
{
    let rows: i32 = inventory.getSizeInventory() / 9;
    let this: Rc<Container> =
        Container::construct(&CLASS, |_| Box::new(ChestExt { m_inventory: Rc::clone(&inventory), m_rows: rows }));
    let n3: i32 = (rows - 4) * 18;
    for n2 in 0..rows
    {
        for n in 0..9
        {
            this.addSlot(Slot::new(Rc::clone(&inventory), n + n2 * 9, 8 + n * 18, 18 + n2 * 18));
        }
    }
    for n2 in 0..3
    {
        for n in 0..9
        {
            this.addSlot(Slot::new(Rc::clone(&playerInventory), n + n2 * 9 + 9, 8 + n * 18, 103 + n2 * 18 + n3));
        }
    }
    for n2 in 0..9
    {
        this.addSlot(Slot::new(Rc::clone(&playerInventory), n2, 8 + n2 * 18, 161 + n3));
    }
    this
}
