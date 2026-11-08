use crate::entity::Entity;
use crate::item::ItemStack;
use crate::tileentity::{TILE_ENTITY, TileEntity, TileEntityClass};
use common::nbt::{NBTBase, NBTTagCompound, NBTTagList};
use std::cell::RefCell;
use std::rc::Rc;

pub type Slots = RefCell<Vec<Option<Rc<ItemStack>>>>;

pub struct ChestExt
{
    m_chestContents: Slots,
}

fn contents(this: &TileEntity) -> &Slots { &this.ext::<ChestExt>().m_chestContents }

pub fn slot(slots: &Slots, index: i32) -> Option<Rc<ItemStack>> { slots.borrow()[index as usize].clone() }

pub fn setSlot(slots: &Slots, index: i32, value: Option<Rc<ItemStack>>) { slots.borrow_mut()[index as usize] = value; }

pub fn decrSlot(this: &Rc<TileEntity>, slots: &Slots, index: i32, amount: i32, notify: bool) -> Option<Rc<ItemStack>>
{
    let current: Rc<ItemStack> = slot(slots, index)?;
    if current.m_stackSize.get() <= amount
    {
        let itemstack1: Rc<ItemStack> = current;
        setSlot(slots, index, None);
        if notify
        {
            this.onInventoryChanged();
        }
        return Some(itemstack1);
    }
    let itemstack: Rc<ItemStack> = current.splitStack(amount);
    if current.m_stackSize.get() == 0
    {
        setSlot(slots, index, None);
    }
    if notify
    {
        this.onInventoryChanged();
    }
    Some(itemstack)
}

pub fn setSlotContents(this: &Rc<TileEntity>, slots: &Slots, index: i32, item: Option<Rc<ItemStack>>, notify: bool)
{
    setSlot(slots, index, item.clone());
    if let Some(item) = item
        && item.m_stackSize.get() > this.getInventoryStackLimit()
    {
        item.m_stackSize.set(this.getInventoryStackLimit());
    }
    if notify
    {
        this.onInventoryChanged();
    }
}

pub fn readSlots(this: &Rc<TileEntity>, slots: &Slots, nbt: &NBTTagCompound)
{
    let nbttaglist: Rc<NBTBase> = nbt.getTagList("Items");
    *slots.borrow_mut() = vec![None; this.getSizeInventory() as usize];
    for i in 0..nbttaglist.list().tagCount()
    {
        let tag: Rc<NBTBase> = nbttaglist.list().tagAt(i);
        let nbttagcompound: &NBTTagCompound = tag.compound();
        let j: i32 = nbttagcompound.getByte("Slot") as i32 & 255;
        if j >= 0 && j < slots.borrow().len() as i32
        {
            setSlot(slots, j, Some(ItemStack::withNbt(nbttagcompound)));
        }
    }
}

pub fn writeSlots(slots: &Slots, nbt: &NBTTagCompound)
{
    let nbttaglist: Rc<NBTBase> = NBTTagList::create();
    let length: i32 = slots.borrow().len() as i32;
    for i in 0..length
    {
        if let Some(stack) = slot(slots, i)
        {
            let nbttagcompound: Rc<NBTBase> = NBTTagCompound::create();
            nbttagcompound.compound().setByte("Slot", i as i8);
            stack.writeToNBT(nbttagcompound.compound());
            nbttaglist.list().setTag(nbttagcompound);
        }
    }
    nbt.setTag("Items", &nbttaglist);
}

pub fn canInteractWith(this: &Rc<TileEntity>, player: &Rc<Entity>) -> bool
{
    let here: Option<Rc<TileEntity>> =
        this.world().getBlockTileEntity(this.m_xCoord.get(), this.m_yCoord.get(), this.m_zCoord.get());
    if !here.is_some_and(|entity| Rc::ptr_eq(&entity, this))
    {
        return false;
    }
    !(player.getDistanceSq(
        this.m_xCoord.get() as f64 + 0.5,
        this.m_yCoord.get() as f64 + 0.5,
        this.m_zCoord.get() as f64 + 0.5,
    ) > 64.0)
}

pub const CLASS: TileEntityClass = TileEntityClass {
    m_name: "TileEntityChest",
    m_parent: Some(&TILE_ENTITY),
    m_interfaces: &["IInventory"],
    getSizeInventory: |_| 27,
    getStackInSlot: |this, index| slot(contents(this), index),
    decrStackSize: |this, index, amount| decrSlot(this, contents(this), index, amount, true),
    setInventorySlotContents: |this, index, item| setSlotContents(this, contents(this), index, item, true),
    getInvName: |_| "Chest",
    readFromNBT: |this, nbt| {
        (TILE_ENTITY.readFromNBT)(this, nbt);
        readSlots(this, contents(this), nbt);
    },
    writeToNBT: |this, nbt| {
        (TILE_ENTITY.writeToNBT)(this, nbt);
        writeSlots(contents(this), nbt);
    },
    getInventoryStackLimit: |_| 64,
    canInteractWith,
    ..TILE_ENTITY
};

pub fn create() -> Rc<TileEntity>
{
    TileEntity::new(&CLASS, Box::new(ChestExt { m_chestContents: RefCell::new(vec![None; 36]) }))
}
