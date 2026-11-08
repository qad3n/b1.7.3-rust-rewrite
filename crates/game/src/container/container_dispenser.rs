use crate::container::slot::Slot;
use crate::container::{CONTAINER, Container, ContainerClass};
use crate::inventory::IInventory;
use crate::tileentity::TileEntity;
use std::rc::Rc;

pub struct DispenserExt
{
    m_dispenser: Rc<TileEntity>,
}

pub const CLASS: ContainerClass = ContainerClass {
    m_name: "ContainerDispenser",
    m_parent: Some(&CONTAINER),
    m_interfaces: &[],
    isUsableByPlayer: |this, player| this.ext::<DispenserExt>().m_dispenser.canInteractWith(player),
    ..CONTAINER
};

pub fn create(playerInventory: Rc<dyn IInventory>, dispenser: &Rc<TileEntity>) -> Rc<Container>
{
    let this: Rc<Container> =
        Container::construct(&CLASS, |_| Box::new(DispenserExt { m_dispenser: Rc::clone(dispenser) }));
    let inventory: Rc<dyn IInventory> = Rc::clone(dispenser) as Rc<dyn IInventory>;
    for n2 in 0..3
    {
        for n in 0..3
        {
            this.addSlot(Slot::new(Rc::clone(&inventory), n + n2 * 3, 62 + n * 18, 17 + n2 * 18));
        }
    }
    for n2 in 0..3
    {
        for n in 0..9
        {
            this.addSlot(Slot::new(Rc::clone(&playerInventory), n + n2 * 9 + 9, 8 + n * 18, 84 + n2 * 18));
        }
    }
    for n2 in 0..9
    {
        this.addSlot(Slot::new(Rc::clone(&playerInventory), n2, 8 + n2 * 18, 142));
    }
    this
}
