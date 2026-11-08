use crate::item::ItemStack;
use crate::tileentity::tile_entity_chest::{self, Slots};
use crate::tileentity::{TILE_ENTITY, TileEntity, TileEntityClass};
use jvm::random::Random;
use std::cell::RefCell;
use std::rc::Rc;

pub struct DispenserExt
{
    m_dispenserContents: Slots,
    m_dispenserRandom: Random,
}

fn contents(this: &TileEntity) -> &Slots { &this.ext::<DispenserExt>().m_dispenserContents }

pub fn getRandomStackFromInventory(this: &Rc<TileEntity>) -> Option<Rc<ItemStack>>
{
    let random: &Random = &this.ext::<DispenserExt>().m_dispenserRandom;
    let mut slot: i32 = -1;
    let mut n: i32 = 1;
    let length: i32 = contents(this).borrow().len() as i32;
    for i in 0..length
    {
        if tile_entity_chest::slot(contents(this), i).is_some()
        {
            let bound: i32 = n;
            n += 1;
            if random.nextIntBounded(bound) == 0
            {
                slot = i;
            }
        }
    }
    if slot >= 0
    {
        return this.decrStackSize(slot, 1);
    }
    None
}

pub const CLASS: TileEntityClass = TileEntityClass {
    m_name: "TileEntityDispenser",
    m_parent: Some(&TILE_ENTITY),
    m_interfaces: &["IInventory"],
    getSizeInventory: |_| 9,
    getStackInSlot: |this, index| tile_entity_chest::slot(contents(this), index),
    decrStackSize: |this, index, amount| tile_entity_chest::decrSlot(this, contents(this), index, amount, true),
    setInventorySlotContents: |this, index, item| {
        tile_entity_chest::setSlotContents(this, contents(this), index, item, true)
    },
    getInvName: |_| "Trap",
    readFromNBT: |this, nbt| {
        (TILE_ENTITY.readFromNBT)(this, nbt);
        tile_entity_chest::readSlots(this, contents(this), nbt);
    },
    writeToNBT: |this, nbt| {
        (TILE_ENTITY.writeToNBT)(this, nbt);
        tile_entity_chest::writeSlots(contents(this), nbt);
    },
    getInventoryStackLimit: |_| 64,
    canInteractWith: tile_entity_chest::canInteractWith,
    ..TILE_ENTITY
};

pub fn create() -> Rc<TileEntity>
{
    let ext: DispenserExt =
        DispenserExt { m_dispenserContents: RefCell::new(vec![None; 9]), m_dispenserRandom: Random::new() };
    TileEntity::new(&CLASS, Box::new(ext))
}
