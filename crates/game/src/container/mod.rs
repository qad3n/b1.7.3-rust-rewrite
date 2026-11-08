pub mod container_chest;
pub mod container_dispenser;
pub mod container_furnace;
pub mod container_player;
pub mod container_workbench;
mod i_crafting;
pub mod slot;
pub mod slot_crafting;
pub mod slot_furnace;

use crate::entity::{Entity, entity_player};
use crate::inventory::{IInventory, InventoryPlayer};
use crate::item::ItemStack;
pub use container_player as ContainerPlayer;
pub use i_crafting::ICrafting;
use jvm::array_list::JArrayList;
use jvm::field::Field;
use jvm::hash_map::JHashSet;
use jvm::throwable::OrNpe;
pub use slot::Slot;
use std::any::Any;
use std::cell::Cell;
use std::rc::{Rc, Weak};

const OUTSIDE_WINDOW: i32 = -999;

crate::vtable! {
    ContainerClass for Container {
        fn getStackInSlot(slot: i32) -> Option<Rc<ItemStack>>;
        fn isUsableByPlayer(player: &Rc<Entity>) -> bool;
        fn onCraftGuiOpened(listener: &Rc<dyn ICrafting>);
        fn onCraftGuiClosed(player: &Rc<Entity>);
        fn onCraftMatrixChanged(inventory: &dyn IInventory);
        fn setData(id: i32, value: i32);
        fn updateCraftingResults();
    }
}

fn updateCraftingResults(this: &Rc<Container>)
{
    let mut i: i32 = 0;
    while i < this.m_slots.size()
    {
        let itemstack: Option<Rc<ItemStack>> = this.m_slots.get(i).getStack();
        let mut itemstack1: Option<Rc<ItemStack>> = this.m_items.get(i);
        if !ItemStack::areItemStacksEqual(itemstack1.as_ref(), itemstack.as_ref())
        {
            itemstack1 = itemstack.map(|stack| stack.copy());
            this.m_items.set(i, itemstack1.clone());
            let mut j: i32 = 0;
            while j < this.m_listeners.size()
            {
                this.m_listeners.get(j).onSlotChanged(this, i, itemstack1.clone());
                j += 1;
            }
        }
        i += 1;
    }
}

fn onCraftGuiOpened(this: &Rc<Container>, listener: &Rc<dyn ICrafting>)
{
    if this.m_listeners.contains(listener)
    {
        jvm::throwable::throw(&jvm::throwable::ILLEGAL_ARGUMENT, "Listener already listening");
    }
    this.m_listeners.add(Rc::clone(listener));
    listener.updateCraftingInventory(this, &this.getItems());
    this.updateCraftingResults();
}

fn onCraftGuiClosed(_this: &Rc<Container>, player: &Rc<Entity>)
{
    let inventoryplayer: Rc<InventoryPlayer> = entity_player::inventory(player);
    if inventoryplayer.getItemStack().is_some()
    {
        entity_player::dropPlayerItem(player, inventoryplayer.getItemStack());
        inventoryplayer.setItemStack(None);
    }
}

pub const CONTAINER: ContainerClass = ContainerClass {
    m_name: "Container",
    m_parent: None,
    m_interfaces: &[],
    getStackInSlot: |this, slot| this.m_slots.get(slot).getStack(),
    isUsableByPlayer: |_, _| unreachable!("Container is abstract"),
    onCraftGuiOpened,
    onCraftGuiClosed,
    onCraftMatrixChanged: |this, _| this.updateCraftingResults(),
    setData: |_, _, _| {},
    updateCraftingResults,
};

pub struct Container
{
    pub m_class: &'static ContainerClass,
    pub m_items: JArrayList<Option<Rc<ItemStack>>>,
    pub m_slots: JArrayList<Rc<Slot>>,
    pub m_windowId: Cell<i32>,
    m_nextInteractionId: Cell<i16>,
    m_listeners: JArrayList<Rc<dyn ICrafting>>,
    m_restrictedPlayers: Field<JHashSet<Rc<Entity>>>,
    m_ext: Box<dyn Any>,
}

impl Container
{
    pub fn construct(
        class: &'static ContainerClass,
        ext: impl FnOnce(&Weak<Container>) -> Box<dyn Any>,
    ) -> Rc<Container>
    {
        Rc::new_cyclic(|this| Container {
            m_class: class,
            m_items: JArrayList::new(),
            m_slots: JArrayList::new(),
            m_windowId: Cell::new(0),
            m_nextInteractionId: Cell::new(0),
            m_listeners: JArrayList::new(),
            m_restrictedPlayers: Field::new(JHashSet::new()),
            m_ext: ext(this),
        })
    }

    pub fn ext<T: 'static>(&self) -> &T { self.m_ext.downcast_ref::<T>().expect("container ext matches its class") }

    fn addSlot(&self, slot: Rc<Slot>)
    {
        slot.m_slotNumber.set(self.m_slots.size());
        self.m_slots.add(slot);
        self.m_items.add(None);
    }

    pub fn getItems(&self) -> Vec<Option<Rc<ItemStack>>>
    {
        let mut arraylist: Vec<Option<Rc<ItemStack>>> = Vec::new();
        let mut i: i32 = 0;
        while i < self.m_slots.size()
        {
            arraylist.push(self.m_slots.get(i).getStack());
            i += 1;
        }
        arraylist
    }

    pub fn getSlotFor(&self, inventory: &Rc<dyn IInventory>, slot: i32) -> Option<Rc<Slot>>
    {
        let mut i: i32 = 0;
        while i < self.m_slots.size()
        {
            let slotx: Rc<Slot> = self.m_slots.get(i);
            if slotx.isHere(inventory, slot)
            {
                return Some(slotx);
            }
            i += 1;
        }
        None
    }

    pub fn getSlot(&self, slot: i32) -> Rc<Slot> { self.m_slots.get(slot) }

    pub fn onClickSlot(
        self: &Rc<Self>,
        slot: i32,
        button: i32,
        quickMove: bool,
        player: &Rc<Entity>,
    ) -> Option<Rc<ItemStack>>
    {
        let mut itemstack: Option<Rc<ItemStack>> = None;
        if button == 0 || button == 1
        {
            let inventoryplayer: Rc<InventoryPlayer> = entity_player::inventory(player);
            if slot == OUTSIDE_WINDOW
            {
                if inventoryplayer.getItemStack().is_some() && slot == OUTSIDE_WINDOW
                {
                    if button == 0
                    {
                        entity_player::dropPlayerItem(player, inventoryplayer.getItemStack());
                        inventoryplayer.setItemStack(None);
                    }
                    if button == 1
                    {
                        entity_player::dropPlayerItem(
                            player,
                            Some(inventoryplayer.getItemStack().orNpe().splitStack(1)),
                        );
                        if inventoryplayer.getItemStack().orNpe().m_stackSize.get() == 0
                        {
                            inventoryplayer.setItemStack(None);
                        }
                    }
                }
            }
            else if quickMove
            {
                let itemstack1: Option<Rc<ItemStack>> = self.getStackInSlot(slot);
                if let Some(itemstack1) = itemstack1
                {
                    let i: i32 = itemstack1.m_stackSize.get();
                    itemstack = Some(itemstack1.copy());
                    let slotx: Rc<Slot> = self.m_slots.get(slot);
                    if let Some(current) = slotx.getStack()
                    {
                        let j: i32 = current.m_stackSize.get();
                        if j < i
                        {
                            self.onClickSlot(slot, button, quickMove, player);
                        }
                    }
                }
            }
            else
            {
                let slot1: Rc<Slot> = self.m_slots.get(slot);
                slot1.onSlotChanged();
                let itemstack3: Option<Rc<ItemStack>> = slot1.getStack();
                let itemstack4: Option<Rc<ItemStack>> = inventoryplayer.getItemStack();
                if let Some(itemstack3) = &itemstack3
                {
                    itemstack = Some(itemstack3.copy());
                }
                match (itemstack3, itemstack4)
                {
                    (None, itemstack4) =>
                    {
                        if let Some(itemstack4) = itemstack4
                            && slot1.isItemValid(&itemstack4)
                        {
                            let mut k: i32 = if button == 0
                            {
                                itemstack4.m_stackSize.get()
                            }
                            else
                            {
                                1
                            };
                            if k > slot1.getSlotStackLimit()
                            {
                                k = slot1.getSlotStackLimit();
                            }
                            slot1.putStack(Some(itemstack4.splitStack(k)));
                            if itemstack4.m_stackSize.get() == 0
                            {
                                inventoryplayer.setItemStack(None);
                            }
                        }
                    }
                    (Some(itemstack3), None) =>
                    {
                        let l: i32 = if button == 0
                        {
                            itemstack3.m_stackSize.get()
                        }
                        else
                        {
                            (itemstack3.m_stackSize.get() + 1) / 2
                        };
                        let itemstack2: Option<Rc<ItemStack>> = slot1.decrStackSize(l);
                        inventoryplayer.setItemStack(itemstack2);
                        if itemstack3.m_stackSize.get() == 0
                        {
                            slot1.putStack(None);
                        }
                        slot1.onPickupFromSlot(inventoryplayer.getItemStack());
                    }
                    (Some(itemstack3), Some(itemstack4)) =>
                    {
                        if slot1.isItemValid(&itemstack4)
                        {
                            Self::placeOnStack(&slot1, &inventoryplayer, &itemstack3, &itemstack4, button);
                        }
                        else
                        {
                            Self::pickUpOntoHeld(&slot1, &inventoryplayer, &itemstack3, &itemstack4);
                        }
                    }
                }
            }
        }
        itemstack
    }

    fn placeOnStack(
        slot1: &Rc<Slot>,
        inventoryplayer: &InventoryPlayer,
        itemstack3: &Rc<ItemStack>,
        itemstack4: &Rc<ItemStack>,
        button: i32,
    )
    {
        if itemstack3.m_itemID.get() != itemstack4.m_itemID.get()
            || itemstack3.getHasSubtypes() && itemstack3.getItemDamage() != itemstack4.getItemDamage()
        {
            if itemstack4.m_stackSize.get() <= slot1.getSlotStackLimit()
            {
                let itemstack5: Rc<ItemStack> = Rc::clone(itemstack3);
                slot1.putStack(Some(Rc::clone(itemstack4)));
                inventoryplayer.setItemStack(Some(itemstack5));
            }
        }
        else
        {
            let mut i1: i32 = if button == 0
            {
                itemstack4.m_stackSize.get()
            }
            else
            {
                1
            };
            if i1 > slot1.getSlotStackLimit() - itemstack3.m_stackSize.get()
            {
                i1 = slot1.getSlotStackLimit() - itemstack3.m_stackSize.get();
            }
            if i1 > itemstack4.getMaxStackSize() - itemstack3.m_stackSize.get()
            {
                i1 = itemstack4.getMaxStackSize() - itemstack3.m_stackSize.get();
            }
            itemstack4.splitStack(i1);
            if itemstack4.m_stackSize.get() == 0
            {
                inventoryplayer.setItemStack(None);
            }
            itemstack3.m_stackSize.set(itemstack3.m_stackSize.get().wrapping_add(i1));
        }
    }

    fn pickUpOntoHeld(
        slot1: &Rc<Slot>,
        inventoryplayer: &InventoryPlayer,
        itemstack3: &Rc<ItemStack>,
        itemstack4: &Rc<ItemStack>,
    )
    {
        if itemstack3.m_itemID.get() == itemstack4.m_itemID.get()
            && itemstack4.getMaxStackSize() > 1
            && (!itemstack3.getHasSubtypes() || itemstack3.getItemDamage() == itemstack4.getItemDamage())
        {
            let j1: i32 = itemstack3.m_stackSize.get();
            if j1 > 0 && j1.wrapping_add(itemstack4.m_stackSize.get()) <= itemstack4.getMaxStackSize()
            {
                itemstack4.m_stackSize.set(itemstack4.m_stackSize.get().wrapping_add(j1));
                itemstack3.splitStack(j1);
                if itemstack3.m_stackSize.get() == 0
                {
                    slot1.putStack(None);
                }
                slot1.onPickupFromSlot(inventoryplayer.getItemStack());
            }
        }
    }

    pub fn getCanCraft(&self, player: &Rc<Entity>) -> bool
    {
        !self.m_restrictedPlayers.with(|set| set.contains(player))
    }

    pub fn setCanCraft(&self, player: &Rc<Entity>, synced: bool)
    {
        if synced
        {
            self.m_restrictedPlayers.update(|set| set.remove(player));
        }
        else
        {
            self.m_restrictedPlayers.update(|set| set.add(Rc::clone(player)));
        }
    }

    pub fn putStackInSlot(&self, slot: i32, item: Option<Rc<ItemStack>>) { self.getSlot(slot).putStack(item); }

    pub fn putStacksInSlots(&self, items: &[Option<Rc<ItemStack>>])
    {
        for (i, item) in items.iter().enumerate()
        {
            self.getSlot(i as i32).putStack(item.clone());
        }
    }

    pub fn nextInteractionId(&self, _inventory: &InventoryPlayer) -> i16
    {
        self.m_nextInteractionId.set(self.m_nextInteractionId.get().wrapping_add(1));
        self.m_nextInteractionId.get()
    }

    pub fn deleteBackup(&self, _id: i16) {}

    pub fn rollbackToBackup(&self, _id: i16) {}

    fn moveItem(&self, item: &Rc<ItemStack>, from: i32, to: i32, reverse: bool)
    {
        let mut i: i32 = from;
        if reverse
        {
            i = to - 1;
        }
        if item.isStackable()
        {
            while item.m_stackSize.get() > 0 && (!reverse && i < to || reverse && i >= from)
            {
                let slot: Rc<Slot> = self.m_slots.get(i);
                let itemstack: Option<Rc<ItemStack>> = slot.getStack();
                if let Some(itemstack) = itemstack
                    && itemstack.m_itemID.get() == item.m_itemID.get()
                    && (!item.getHasSubtypes() || item.getItemDamage() == itemstack.getItemDamage())
                {
                    let j: i32 = itemstack.m_stackSize.get().wrapping_add(item.m_stackSize.get());
                    if j <= item.getMaxStackSize()
                    {
                        item.m_stackSize.set(0);
                        itemstack.m_stackSize.set(j);
                        slot.onSlotChanged();
                    }
                    else if itemstack.m_stackSize.get() < item.getMaxStackSize()
                    {
                        item.m_stackSize.set(
                            item.m_stackSize.get().wrapping_sub(item.getMaxStackSize() - itemstack.m_stackSize.get()),
                        );
                        itemstack.m_stackSize.set(item.getMaxStackSize());
                        slot.onSlotChanged();
                    }
                }
                if reverse
                {
                    i -= 1;
                }
                else
                {
                    i += 1;
                }
            }
        }
        if item.m_stackSize.get() > 0
        {
            if reverse
            {
                i = to - 1;
            }
            else
            {
                i = from;
            }
            while !reverse && i < to || reverse && i >= from
            {
                let slot1: Rc<Slot> = self.m_slots.get(i);
                let itemstack1: Option<Rc<ItemStack>> = slot1.getStack();
                if itemstack1.is_none()
                {
                    slot1.putStack(Some(item.copy()));
                    slot1.onSlotChanged();
                    item.m_stackSize.set(0);
                    break;
                }
                if reverse
                {
                    i -= 1;
                }
                else
                {
                    i += 1;
                }
            }
        }
    }

    fn quickMove(&self, slot: i32, moveTo: impl FnOnce(&Rc<ItemStack>)) -> Option<Rc<ItemStack>>
    {
        let mut itemstack: Option<Rc<ItemStack>> = None;
        let slotx: Rc<Slot> = self.m_slots.get(slot);
        if slotx.getHasStack()
        {
            let itemstack1: Rc<ItemStack> = slotx.getStack().orNpe();
            let copy: Rc<ItemStack> = itemstack1.copy();
            itemstack = Some(Rc::clone(&copy));
            moveTo(&itemstack1);
            if itemstack1.m_stackSize.get() == 0
            {
                slotx.putStack(None);
            }
            else
            {
                slotx.onSlotChanged();
            }
            if itemstack1.m_stackSize.get() == copy.m_stackSize.get()
            {
                return None;
            }
            slotx.onPickupFromSlot(Some(itemstack1));
        }
        itemstack
    }
}
