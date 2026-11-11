use crate::block::Block;
use crate::entity::Entity;
use crate::item::Item;
use crate::stats::named::ContentStat;
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
    // rewrite: typed per stack data from mods by component identifier, stacks merge only when these match
    pub m_components: std::cell::RefCell<common::nbt::attachments::Attachments>,
}

impl ItemStack
{
    pub fn withBlock(block: &Block) -> Rc<Self> { Self::withBlockCount(block, 1) }

    pub fn withBlockCount(block: &Block, size: i32) -> Rc<Self> { Self::withDamage(block.m_blockID, size, 0) }

    pub fn withBlockDamage(block: &Block, size: i32, metadata: i32) -> Rc<Self>
    {
        Self::withDamage(block.m_blockID, size, metadata)
    }

    pub fn withItem(item: &Item) -> Rc<Self> { Self::withDamage(item.m_shiftedIndex, 1, 0) }

    pub fn withItemCount(item: &Item, size: i32) -> Rc<Self> { Self::withDamage(item.m_shiftedIndex, size, 0) }

    pub fn withItemDamage(item: &Item, size: i32, metadata: i32) -> Rc<Self>
    {
        Self::withDamage(item.m_shiftedIndex, size, metadata)
    }

    pub fn withDamage(id: i32, size: i32, metadata: i32) -> Rc<Self>
    {
        Rc::new(Self
        {
            m_stackSize: Cell::new(size),
            m_animationsToGo: Cell::new(0),
            m_itemID: Cell::new(id),
            m_itemDamage: Cell::new(metadata),
            m_components: std::cell::RefCell::new(common::nbt::attachments::Attachments::new()),
        })
    }

    pub fn withNbt(nbt: &NBTTagCompound) -> Rc<Self>
    {
        let stack: Rc<Self> = Self::withDamage(0, 0, 0);
        stack.readFromNBT(nbt);
        stack
    }

    pub fn splitStack(&self, amount: i32) -> Rc<Self>
    {
        self.m_stackSize.set(self.m_stackSize.get() - amount);
        self.withComponents(Self::withDamage(self.m_itemID.get(), amount, self.m_itemDamage.get()))
    }

    pub fn getItem(&self) -> Rc<Item> { Item::byIdOrNpe(self.m_itemID.get()) }

    // java getItem may return null and this variant keeps it for callers that test the result
    pub fn getItemNullable(&self) -> Option<Rc<Item>> { Item::byId(self.m_itemID.get()) }

    pub fn getIconIndex(self: &Rc<Self>) -> i32 { self.getItem().getIconIndex(self) }

    pub fn useItem(self: &Rc<Self>, player: &Rc<Entity>, world: &Rc<World>, x: i32, y: i32, z: i32, face: i32) -> bool
    {
        // mods may take the use before the item does, a canceled use counts as used so the right click stops here
        if world.playerEdit(||
        {
            crate::hooks::fire(crate::hooks::GameEvent::ItemUseOnBlock { world, player, stack: self, x, y, z, face })
        })
        {
            return true;
        }

        let bl: bool = world.playerEdit(|| self.getItem().onItemUse(self, player, world, x, y, z, face));
        if bl
        {
            crate::stats::named::count(player, ContentStat::Used, self.m_itemID.get(), 1);
        }

        bl
    }

    pub fn getStrVsBlock(self: &Rc<Self>, block: &Rc<Block>) -> f32 { self.getItem().getStrVsBlock(self, block) }

    pub fn useItemRightClick(self: &Rc<Self>, world: &Rc<World>, player: &Rc<Entity>) -> Option<Rc<Self>>
    {
        // mods may take the right click before the item does
        if world.playerEdit(|| crate::hooks::fire(crate::hooks::GameEvent::ItemUse { world, player, stack: self }))
        {
            return Some(Rc::clone(self));
        }

        world.playerEdit(|| self.getItem().onItemRightClick(self, world, player))
    }

    pub fn writeToNBT<'a>(&self, nbt: &'a NBTTagCompound) -> &'a NBTTagCompound
    {
        nbt.setShort("id", self.m_itemID.get() as i16);
        nbt.setByte("Count", self.m_stackSize.get() as i8);
        nbt.setShort("Damage", self.m_itemDamage.get() as i16);
        // rewrite: with mods loaded a stack also saves its item by name and its components
        crate::hooks::writeItemName(self.m_itemID.get(), nbt);
        common::nbt::attachments::writeAt(&self.m_components.borrow(), nbt, common::nbt::attachments::COMPONENTS);
        nbt
    }

    pub fn readFromNBT(&self, nbt: &NBTTagCompound)
    {
        self.m_itemID.set(nbt.getShort("id") as i32);
        self.m_stackSize.set(nbt.getByte("Count") as i32);
        self.m_itemDamage.set(nbt.getShort("Damage") as i32);

        // rewrite: a saved name wins over the numeric id, which only a vanilla game or an unknown item falls back to
        if let Some(id) = crate::hooks::itemByName(nbt)
        {
            self.m_itemID.set(id);
        }
        *self.m_components.borrow_mut() = common::nbt::attachments::readAt(nbt, common::nbt::attachments::COMPONENTS);
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
        // mods observe the stack reaching its last use
        if crate::hooks::active().is_some()
            && self.isItemStackDamageable()
            && self.m_itemDamage.get() + amount > self.getMaxDamage()
        {
            crate::hooks::fire(crate::hooks::GameEvent::ItemDestroyed { player: holder, stack: self });
        }

        if !self.isItemStackDamageable()
        {
            return;
        }

        self.m_itemDamage.set(self.m_itemDamage.get() + amount);

        if self.m_itemDamage.get() > self.getMaxDamage()
        {
            if holder.instanceOf("EntityPlayer")
            {
                crate::stats::named::count(holder, ContentStat::Broken, self.m_itemID.get(), 1);
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
            crate::stats::named::count(attacker, ContentStat::Used, self.m_itemID.get(), 1);
        }
    }

    pub fn onDestroyBlock(self: &Rc<Self>, block: i32, x: i32, y: i32, z: i32, player: &Rc<Entity>)
    {
        let bl: bool = self.getItem().onBlockDestroyed(self, block, x, y, z, player);
        if bl
        {
            crate::stats::named::count(player, ContentStat::Used, self.m_itemID.get(), 1);
        }
    }

    pub fn getDamageVsEntity(&self, target: Option<&Rc<Entity>>) -> i32 { self.getItem().getDamageVsEntity(target) }

    pub fn canHarvestBlock(&self, block: &Rc<Block>) -> bool { self.getItem().canHarvestBlock(block) }

    pub fn onRemoved(&self, _player: &Rc<Entity>) {}

    pub fn useItemOnEntity(self: &Rc<Self>, entity: &Rc<Entity>) { self.getItem().saddleEntity(self, entity); }

    pub fn copy(&self) -> Rc<Self>
    {
        self.withComponents(Self::withDamage(self.m_itemID.get(), self.m_stackSize.get(), self.m_itemDamage.get()))
    }

    fn withComponents(&self, stack: Rc<Self>) -> Rc<Self>
    {
        stack.m_components.borrow_mut().clone_from(&self.m_components.borrow());
        stack
    }

    // rewrite: stacks with different mod components never merge or compare equal
    pub fn sameComponents(&self, other: &Self) -> bool { *self.m_components.borrow() == *other.m_components.borrow() }

    pub fn areItemStacksEqual(stack1: Option<&Rc<Self>>, stack2: Option<&Rc<Self>>) -> bool
    {
        match (stack1, stack2)
        {
            (None, None) => true,
            (Some(first), Some(second)) => first.isItemStackEqual(second),
            _ => false,
        }
    }

    fn isItemStackEqual(&self, other: &Self) -> bool
    {
        if self.m_stackSize.get() != other.m_stackSize.get()
        {
            return false;
        }
        if self.m_itemID.get() != other.m_itemID.get()
        {
            return false;
        }

        self.m_itemDamage.get() == other.m_itemDamage.get() && self.sameComponents(other)
    }

    pub fn isItemEqual(&self, other: &Self) -> bool
    {
        self.m_itemID.get() == other.m_itemID.get()
            && self.m_itemDamage.get() == other.m_itemDamage.get()
            && self.sameComponents(other)
    }

    pub fn getItemName(self: &Rc<Self>) -> Option<String> { self.getItem().getItemNameIS(self) }

    pub fn copyItemStack(stack: Option<&Rc<Self>>) -> Option<Rc<Self>> { stack.map(|stack| stack.copy()) }

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
        crate::stats::named::count(player, ContentStat::Crafted, self.m_itemID.get(), self.m_stackSize.get());
        self.getItem().onCreated(self, world, player);
    }

    pub fn isStackEqual(&self, other: &Self) -> bool
    {
        self.m_itemID.get() == other.m_itemID.get()
            && self.m_stackSize.get() == other.m_stackSize.get()
            && self.m_itemDamage.get() == other.m_itemDamage.get()
            && self.sameComponents(other)
    }
}
