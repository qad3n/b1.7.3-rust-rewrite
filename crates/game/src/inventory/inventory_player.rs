use crate::block::Block;
use crate::entity::{Entity, entity_player};
use crate::inventory::IInventory;
use crate::item::{ItemStack, item_armor};
use common::nbt::{NBTBase, NBTTagCompound};
use jvm::field::Field;
use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};

pub struct InventoryPlayer
{
    pub m_mainInventory: RefCell<Vec<Option<Rc<ItemStack>>>>,
    pub m_armorInventory: RefCell<Vec<Option<Rc<ItemStack>>>>,
    pub m_currentItem: Cell<i32>,
    m_player: Weak<Entity>,
    m_itemStack: Field<Option<Rc<ItemStack>>>,
    pub m_inventoryChanged: Cell<bool>,
}

fn at(slots: &RefCell<Vec<Option<Rc<ItemStack>>>>, index: i32) -> Option<Rc<ItemStack>>
{
    slots.borrow()[index as usize].clone()
}

fn put(slots: &RefCell<Vec<Option<Rc<ItemStack>>>>, index: i32, value: Option<Rc<ItemStack>>)
{
    slots.borrow_mut()[index as usize] = value;
}

impl InventoryPlayer
{
    pub fn getHotbarSize() -> i32 { 9 }

    pub fn new(player: &Weak<Entity>) -> InventoryPlayer
    {
        InventoryPlayer {
            m_mainInventory: RefCell::new(vec![None; 36]),
            m_armorInventory: RefCell::new(vec![None; 4]),
            m_currentItem: Cell::new(0),
            m_player: Weak::clone(player),
            m_itemStack: Field::new(None),
            m_inventoryChanged: Cell::new(false),
        }
    }

    pub fn player(&self) -> Rc<Entity> { self.m_player.upgrade().expect("the player owns its inventory") }

    fn mainLength(&self) -> i32 { self.m_mainInventory.borrow().len() as i32 }

    fn armorLength(&self) -> i32 { self.m_armorInventory.borrow().len() as i32 }

    pub fn getCurrentItem(&self) -> Option<Rc<ItemStack>>
    {
        let current: i32 = self.m_currentItem.get();
        if current < 9 && current >= 0
        {
            return at(&self.m_mainInventory, current);
        }
        None
    }

    fn getInventorySlotContainItem(&self, item: i32) -> i32
    {
        for i in 0..self.mainLength()
        {
            if at(&self.m_mainInventory, i).is_some_and(|stack| stack.m_itemID.get() == item)
            {
                return i;
            }
        }
        -1
    }

    fn storeItemStack(&self, item: &ItemStack) -> i32
    {
        for i in 0..self.mainLength()
        {
            if let Some(stack) = at(&self.m_mainInventory, i)
                && stack.m_itemID.get() == item.m_itemID.get()
                && stack.isStackable()
                && stack.m_stackSize.get() < stack.getMaxStackSize()
                && stack.m_stackSize.get() < self.getInventoryStackLimit()
                && (!stack.getHasSubtypes() || stack.getItemDamage() == item.getItemDamage())
            {
                return i;
            }
        }
        -1
    }

    fn getFirstEmptyStack(&self) -> i32
    {
        for i in 0..self.mainLength()
        {
            if at(&self.m_mainInventory, i).is_none()
            {
                return i;
            }
        }
        -1
    }

    pub fn setCurrentItem(&self, block: i32, _creative: bool)
    {
        let n: i32 = self.getInventorySlotContainItem(block);
        if (0..9).contains(&n)
        {
            self.m_currentItem.set(n);
        }
    }

    pub fn changeCurrentItem(&self, mut amount: i32)
    {
        if amount > 0
        {
            amount = 1;
        }
        if amount < 0
        {
            amount = -1;
        }
        self.m_currentItem.set(self.m_currentItem.get() - amount);
        while self.m_currentItem.get() < 0
        {
            self.m_currentItem.set(self.m_currentItem.get() + 9);
        }
        while self.m_currentItem.get() >= 9
        {
            self.m_currentItem.set(self.m_currentItem.get() - 9);
        }
    }

    fn storePartialItemStack(&self, item: &ItemStack) -> i32
    {
        let i: i32 = item.m_itemID.get();
        let mut j: i32 = item.m_stackSize.get();
        let mut k: i32 = self.storeItemStack(item);
        if k < 0
        {
            k = self.getFirstEmptyStack();
        }
        if k < 0
        {
            return j;
        }
        if at(&self.m_mainInventory, k).is_none()
        {
            put(&self.m_mainInventory, k, Some(ItemStack::withDamage(i, 0, item.getItemDamage())));
        }
        let stack: Rc<ItemStack> = at(&self.m_mainInventory, k).expect("slot filled above");
        let mut l: i32 = j;
        if l > stack.getMaxStackSize() - stack.m_stackSize.get()
        {
            l = stack.getMaxStackSize() - stack.m_stackSize.get();
        }
        if l > self.getInventoryStackLimit() - stack.m_stackSize.get()
        {
            l = self.getInventoryStackLimit() - stack.m_stackSize.get();
        }
        if l == 0
        {
            return j;
        }
        j -= l;
        stack.m_stackSize.set(stack.m_stackSize.get() + l);
        stack.m_animationsToGo.set(5);
        j
    }

    pub fn decrementAnimations(&self)
    {
        let player: Rc<Entity> = self.player();
        for i in 0..self.mainLength()
        {
            if let Some(stack) = at(&self.m_mainInventory, i)
            {
                stack.updateAnimation(&player.world(), &player, i, self.m_currentItem.get() == i);
            }
        }
    }

    pub fn consumeInventoryItem(&self, item: i32) -> bool
    {
        let n: i32 = self.getInventorySlotContainItem(item);
        if n < 0
        {
            return false;
        }
        let stack: Rc<ItemStack> = at(&self.m_mainInventory, n).expect("slot holds the item");
        stack.m_stackSize.set(stack.m_stackSize.get() - 1);
        if stack.m_stackSize.get() <= 0
        {
            put(&self.m_mainInventory, n, None);
        }
        true
    }

    pub fn addItemStackToInventory(&self, item: &Rc<ItemStack>) -> bool
    {
        if !item.isItemDamaged()
        {
            let mut stackSize: i32;
            loop
            {
                stackSize = item.m_stackSize.get();
                item.m_stackSize.set(self.storePartialItemStack(item));
                if !(item.m_stackSize.get() > 0 && item.m_stackSize.get() < stackSize)
                {
                    break;
                }
            }
            return item.m_stackSize.get() < stackSize;
        }
        let firstEmptyStack: i32 = self.getFirstEmptyStack();
        if firstEmptyStack >= 0
        {
            let copy: Option<Rc<ItemStack>> = ItemStack::copyItemStack(Some(item));
            put(&self.m_mainInventory, firstEmptyStack, copy.clone());
            if let Some(copy) = copy
            {
                copy.m_animationsToGo.set(5);
            }
            item.m_stackSize.set(0);
            return true;
        }
        false
    }

    pub fn getStrVsBlock(&self, block: &Rc<Block>) -> f32
    {
        let mut n: f32 = 1.0;
        if let Some(stack) = at(&self.m_mainInventory, self.m_currentItem.get())
        {
            n *= stack.getStrVsBlock(block);
        }
        n
    }

    pub fn writeToNBT(&self, list: &Rc<NBTBase>) -> Rc<NBTBase>
    {
        for i in 0..self.mainLength()
        {
            if let Some(stack) = at(&self.m_mainInventory, i)
            {
                let nbtTagCompound: Rc<NBTBase> = NBTTagCompound::create();
                nbtTagCompound.compound().setByte("Slot", i as i8);
                stack.writeToNBT(nbtTagCompound.compound());
                list.list().setTag(nbtTagCompound);
            }
        }
        for j in 0..self.armorLength()
        {
            if let Some(stack) = at(&self.m_armorInventory, j)
            {
                let nbtTagCompound2: Rc<NBTBase> = NBTTagCompound::create();
                nbtTagCompound2.compound().setByte("Slot", (j + 100) as i8);
                stack.writeToNBT(nbtTagCompound2.compound());
                list.list().setTag(nbtTagCompound2);
            }
        }
        Rc::clone(list)
    }

    pub fn readFromNBT(&self, list: &Rc<NBTBase>)
    {
        *self.m_mainInventory.borrow_mut() = vec![None; 36];
        *self.m_armorInventory.borrow_mut() = vec![None; 4];
        for i in 0..list.list().tagCount()
        {
            let tag: Rc<NBTBase> = list.list().tagAt(i);
            let nbt: &NBTTagCompound = tag.compound();
            let n: i32 = nbt.getByte("Slot") as i32 & 0xFF;
            let itemStack: Rc<ItemStack> = ItemStack::withNbt(nbt);
            if crate::item::Item::byId(itemStack.m_itemID.get()).is_some()
            {
                if n >= 0 && n < self.mainLength()
                {
                    put(&self.m_mainInventory, n, Some(Rc::clone(&itemStack)));
                }
                if n >= 100 && n < self.armorLength() + 100
                {
                    put(&self.m_armorInventory, n - 100, Some(Rc::clone(&itemStack)));
                }
            }
        }
    }

    pub fn getDamageVsEntity(&self, target: &Rc<Entity>) -> i32
    {
        match self.getStackInSlot(self.m_currentItem.get())
        {
            Some(stack) => stack.getDamageVsEntity(Some(target)),
            None => 1,
        }
    }

    pub fn canHarvestBlock(&self, block: &Rc<Block>) -> bool
    {
        if block.m_blockMaterial.getIsHarvestable()
        {
            return true;
        }
        match self.getStackInSlot(self.m_currentItem.get())
        {
            Some(itemStack) => itemStack.canHarvestBlock(block),
            None => false,
        }
    }

    pub fn armorItemInSlot(&self, slot: i32) -> Option<Rc<ItemStack>> { at(&self.m_armorInventory, slot) }

    pub fn getTotalArmorValue(&self) -> i32
    {
        let mut n: i32 = 0;
        let mut n2: i32 = 0;
        let mut n3: i32 = 0;
        for i in 0..self.armorLength()
        {
            if let Some(stack) = at(&self.m_armorInventory, i)
                && stack.getItem().instanceOf("ItemArmor")
            {
                let maxDamage: i32 = stack.getMaxDamage();
                n2 += maxDamage - stack.getItemDamageForDisplay();
                n3 += maxDamage;
                n += item_armor::fields(&stack.getItem()).m_damageReduceAmount;
            }
        }
        if n3 == 0
        {
            return 0;
        }
        (n - 1) * n2 / n3 + 1
    }

    pub fn damageArmor(&self, damage: i32)
    {
        let player: Rc<Entity> = self.player();
        for i in 0..self.armorLength()
        {
            if let Some(stack) = at(&self.m_armorInventory, i)
                && stack.getItem().instanceOf("ItemArmor")
            {
                stack.damageItem(damage, &player);
                if stack.m_stackSize.get() == 0
                {
                    stack.onRemoved(&player);
                    put(&self.m_armorInventory, i, None);
                }
            }
        }
    }

    pub fn dropAllItems(&self)
    {
        let player: Rc<Entity> = self.player();
        for i in 0..self.mainLength()
        {
            if let Some(stack) = at(&self.m_mainInventory, i)
            {
                entity_player::dropPlayerItemWithRandomChoice(&player, Some(stack), true);
                put(&self.m_mainInventory, i, None);
            }
        }
        for j in 0..self.armorLength()
        {
            if let Some(stack) = at(&self.m_armorInventory, j)
            {
                entity_player::dropPlayerItemWithRandomChoice(&player, Some(stack), true);
                put(&self.m_armorInventory, j, None);
            }
        }
    }

    pub fn setItemStack(&self, item: Option<Rc<ItemStack>>)
    {
        self.m_itemStack.set(item.clone());
        self.player().onItemStackChanged(item);
    }

    pub fn getItemStack(&self) -> Option<Rc<ItemStack>> { self.m_itemStack.get() }

    pub fn contains(&self, item: &ItemStack) -> bool
    {
        for i in 0..self.armorLength()
        {
            if at(&self.m_armorInventory, i).is_some_and(|stack| stack.isStackEqual(item))
            {
                return true;
            }
        }
        for j in 0..self.mainLength()
        {
            if at(&self.m_mainInventory, j).is_some_and(|stack| stack.isStackEqual(item))
            {
                return true;
            }
        }
        false
    }

    fn slots(&self, slot: i32) -> (&RefCell<Vec<Option<Rc<ItemStack>>>>, i32)
    {
        if slot >= self.mainLength()
        {
            return (&self.m_armorInventory, slot - self.mainLength());
        }
        (&self.m_mainInventory, slot)
    }
}

impl IInventory for InventoryPlayer
{
    fn getSizeInventory(&self) -> i32 { self.mainLength() + 4 }

    fn getStackInSlot(&self, slot: i32) -> Option<Rc<ItemStack>>
    {
        let (array, index): (&RefCell<Vec<Option<Rc<ItemStack>>>>, i32) = self.slots(slot);
        at(array, index)
    }

    fn decrStackSize(&self, slot: i32, amount: i32) -> Option<Rc<ItemStack>>
    {
        let (array, index): (&RefCell<Vec<Option<Rc<ItemStack>>>>, i32) = self.slots(slot);
        let stack: Rc<ItemStack> = at(array, index)?;
        if stack.m_stackSize.get() <= amount
        {
            put(array, index, None);
            return Some(stack);
        }
        let itemStack: Rc<ItemStack> = stack.splitStack(amount);
        if stack.m_stackSize.get() == 0
        {
            put(array, index, None);
        }
        Some(itemStack)
    }

    fn setInventorySlotContents(&self, slot: i32, item: Option<Rc<ItemStack>>)
    {
        let (array, index): (&RefCell<Vec<Option<Rc<ItemStack>>>>, i32) = self.slots(slot);
        put(array, index, item);
    }

    fn getInvName(&self) -> String { "Inventory".to_string() }

    fn getInventoryStackLimit(&self) -> i32 { 64 }

    fn onInventoryChanged(&self) { self.m_inventoryChanged.set(true); }

    fn canInteractWith(&self, player: &Rc<Entity>) -> bool
    {
        let owner: Rc<Entity> = self.player();
        if owner.m_isDead.get()
        {
            return false;
        }
        !(player.getDistanceSqToEntity(&owner) > 64.0)
    }
}
