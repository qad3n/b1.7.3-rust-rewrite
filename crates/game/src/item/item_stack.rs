use crate::block::Block;
use crate::entity::Entity;
use crate::item::Item;
use crate::stats::StatList;
use crate::world::World;
use common::nbt::NBTTagCompound;
use std::cell::Cell;
use std::rc::Rc;

pub struct ItemStack
{
    pub m_stackSize: Cell<i32>,
    pub m_animationsToGo: Cell<i32>,
    pub m_itemID: Cell<i32>,
    m_itemDamage: Cell<i32>,
}

impl ItemStack
{
    pub fn withBlock(block: &Block) -> Rc<ItemStack> { ItemStack::withBlockCount(block, 1) }

    pub fn withBlockCount(block: &Block, size: i32) -> Rc<ItemStack> { ItemStack::withDamage(block.m_blockID, size, 0) }

    pub fn withBlockDamage(block: &Block, size: i32, metadata: i32) -> Rc<ItemStack>
    {
        ItemStack::withDamage(block.m_blockID, size, metadata)
    }

    pub fn withItem(item: &Item) -> Rc<ItemStack> { ItemStack::withDamage(item.m_shiftedIndex, 1, 0) }

    pub fn withItemCount(item: &Item, size: i32) -> Rc<ItemStack>
    {
        ItemStack::withDamage(item.m_shiftedIndex, size, 0)
    }

    pub fn withItemDamage(item: &Item, size: i32, metadata: i32) -> Rc<ItemStack>
    {
        ItemStack::withDamage(item.m_shiftedIndex, size, metadata)
    }

    pub fn withDamage(id: i32, size: i32, metadata: i32) -> Rc<ItemStack>
    {
        Rc::new(ItemStack {
            m_stackSize: Cell::new(size),
            m_animationsToGo: Cell::new(0),
            m_itemID: Cell::new(id),
            m_itemDamage: Cell::new(metadata),
        })
    }

    pub fn withNbt(nbt: &NBTTagCompound) -> Rc<ItemStack>
    {
        let stack: Rc<ItemStack> = ItemStack::withDamage(0, 0, 0);
        stack.readFromNBT(nbt);
        stack
    }

    pub fn splitStack(&self, amount: i32) -> Rc<ItemStack>
    {
        self.m_stackSize.set(self.m_stackSize.get() - amount);
        ItemStack::withDamage(self.m_itemID.get(), amount, self.m_itemDamage.get())
    }

    pub fn getItem(&self) -> Rc<Item> { Item::byIdOrNpe(self.m_itemID.get()) }

    // java getItem may return null and this variant keeps it for callers that test the result
    pub fn getItemNullable(&self) -> Option<Rc<Item>> { Item::byId(self.m_itemID.get()) }

    pub fn getIconIndex(self: &Rc<Self>) -> i32 { self.getItem().getIconIndex(self) }

    pub fn useItem(self: &Rc<Self>, player: &Rc<Entity>, world: &Rc<World>, x: i32, y: i32, z: i32, face: i32) -> bool
    {
        let bl: bool = self.getItem().onItemUse(self, player, world, x, y, z, face);
        if bl
        {
            player.addStat(StatList::itemsUsed(self.m_itemID.get()).as_ref(), 1);
        }
        bl
    }

    pub fn getStrVsBlock(self: &Rc<Self>, block: &Rc<Block>) -> f32 { self.getItem().getStrVsBlock(self, block) }

    pub fn useItemRightClick(self: &Rc<Self>, world: &Rc<World>, player: &Rc<Entity>) -> Option<Rc<ItemStack>>
    {
        self.getItem().onItemRightClick(self, world, player)
    }

    pub fn writeToNBT<'a>(&self, nbt: &'a NBTTagCompound) -> &'a NBTTagCompound
    {
        nbt.setShort("id", self.m_itemID.get() as i16);
        nbt.setByte("Count", self.m_stackSize.get() as i8);
        nbt.setShort("Damage", self.m_itemDamage.get() as i16);
        nbt
    }

    pub fn readFromNBT(&self, nbt: &NBTTagCompound)
    {
        self.m_itemID.set(nbt.getShort("id") as i32);
        self.m_stackSize.set(nbt.getByte("Count") as i32);
        self.m_itemDamage.set(nbt.getShort("Damage") as i32);
    }

    pub fn getMaxStackSize(&self) -> i32 { self.getItem().getItemStackLimit() }

    pub fn isStackable(&self) -> bool
    {
        self.getMaxStackSize() > 1 && (!self.isItemStackDamageable() || !self.isItemDamaged())
    }

    pub fn isItemStackDamageable(&self) -> bool { self.getItem().getMaxDamage() > 0 }

    pub fn getHasSubtypes(&self) -> bool { self.getItem().getHasSubtypes() }

    pub fn isItemDamaged(&self) -> bool { self.isItemStackDamageable() && self.m_itemDamage.get() > 0 }

    pub fn getItemDamageForDisplay(&self) -> i32 { self.m_itemDamage.get() }

    pub fn getItemDamage(&self) -> i32 { self.m_itemDamage.get() }

    pub fn setItemDamage(&self, damage: i32) { self.m_itemDamage.set(damage); }

    pub fn getMaxDamage(&self) -> i32 { self.getItem().getMaxDamage() }

    pub fn damageItem(&self, amount: i32, holder: &Rc<Entity>)
    {
        if !self.isItemStackDamageable()
        {
            return;
        }
        self.m_itemDamage.set(self.m_itemDamage.get() + amount);
        if self.m_itemDamage.get() > self.getMaxDamage()
        {
            if holder.instanceOf("EntityPlayer")
            {
                holder.addStat(StatList::itemsBroken(self.m_itemID.get()).as_ref(), 1);
            }
            self.m_stackSize.set(self.m_stackSize.get() - 1);
            if self.m_stackSize.get() < 0
            {
                self.m_stackSize.set(0);
            }
            self.m_itemDamage.set(0);
        }
    }

    pub fn hitEntity(self: &Rc<Self>, target: &Rc<Entity>, attacker: &Rc<Entity>)
    {
        let bl: bool = self.getItem().hitEntity(self, target, attacker);
        if bl
        {
            attacker.addStat(StatList::itemsUsed(self.m_itemID.get()).as_ref(), 1);
        }
    }

    pub fn onDestroyBlock(self: &Rc<Self>, block: i32, x: i32, y: i32, z: i32, player: &Rc<Entity>)
    {
        let bl: bool = self.getItem().onBlockDestroyed(self, block, x, y, z, player);
        if bl
        {
            player.addStat(StatList::itemsUsed(self.m_itemID.get()).as_ref(), 1);
        }
    }

    pub fn getDamageVsEntity(&self, target: Option<&Rc<Entity>>) -> i32 { self.getItem().getDamageVsEntity(target) }

    pub fn canHarvestBlock(&self, block: &Rc<Block>) -> bool { self.getItem().canHarvestBlock(block) }

    pub fn onRemoved(&self, _player: &Rc<Entity>) {}

    pub fn useItemOnEntity(self: &Rc<Self>, entity: &Rc<Entity>) { self.getItem().saddleEntity(self, entity); }

    pub fn copy(&self) -> Rc<ItemStack>
    {
        ItemStack::withDamage(self.m_itemID.get(), self.m_stackSize.get(), self.m_itemDamage.get())
    }

    pub fn areItemStacksEqual(stack1: Option<&Rc<ItemStack>>, stack2: Option<&Rc<ItemStack>>) -> bool
    {
        match (stack1, stack2)
        {
            (None, None) => true,
            (Some(first), Some(second)) => first.isItemStackEqual(second),
            _ => false,
        }
    }

    fn isItemStackEqual(&self, other: &ItemStack) -> bool
    {
        if self.m_stackSize.get() != other.m_stackSize.get()
        {
            return false;
        }
        if self.m_itemID.get() != other.m_itemID.get()
        {
            return false;
        }
        self.m_itemDamage.get() == other.m_itemDamage.get()
    }

    pub fn isItemEqual(&self, other: &ItemStack) -> bool
    {
        self.m_itemID.get() == other.m_itemID.get() && self.m_itemDamage.get() == other.m_itemDamage.get()
    }

    pub fn getItemName(self: &Rc<Self>) -> Option<String> { self.getItem().getItemNameIS(self) }

    pub fn copyItemStack(stack: Option<&Rc<ItemStack>>) -> Option<Rc<ItemStack>> { stack.map(|stack| stack.copy()) }

    pub fn toString(&self) -> String
    {
        let name: String = self.getItem().getItemName().unwrap_or_else(|| "null".to_string());
        format!("{}x{}@{}", self.m_stackSize.get(), name, self.m_itemDamage.get())
    }

    pub fn updateAnimation(self: &Rc<Self>, world: &Rc<World>, entity: &Rc<Entity>, slot: i32, selected: bool)
    {
        if self.m_animationsToGo.get() > 0
        {
            self.m_animationsToGo.set(self.m_animationsToGo.get() - 1);
        }
        self.getItem().onUpdate(self, world, entity, slot, selected);
    }

    pub fn onCrafting(self: &Rc<Self>, world: &Rc<World>, player: &Rc<Entity>)
    {
        player.addStat(StatList::itemsCrafted(self.m_itemID.get()).as_ref(), self.m_stackSize.get());
        self.getItem().onCreated(self, world, player);
    }

    pub fn isStackEqual(&self, other: &ItemStack) -> bool
    {
        self.m_itemID.get() == other.m_itemID.get()
            && self.m_stackSize.get() == other.m_stackSize.get()
            && self.m_itemDamage.get() == other.m_itemDamage.get()
    }
}
