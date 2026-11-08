use crate::block::{Block, Material, block_furnace};
use crate::crafting::FurnaceRecipes;
use crate::item::{Item, ItemStack};
use crate::tileentity::tile_entity_chest::{self, Slots};
use crate::tileentity::{TILE_ENTITY, TileEntity, TileEntityClass};
use crate::world::World;
use common::nbt::{NBTBase, NBTTagCompound, NBTTagList};
use jvm::throwable::OrNpe;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

pub struct FurnaceExt
{
    m_furnaceItemStacks: Slots,
    pub m_furnaceBurnTime: Cell<i32>,
    pub m_currentItemBurnTime: Cell<i32>,
    pub m_furnaceCookTime: Cell<i32>,
}

pub fn fields(this: &TileEntity) -> &FurnaceExt { this.ext::<FurnaceExt>() }

fn stacks(this: &TileEntity) -> &Slots { &fields(this).m_furnaceItemStacks }

fn stack(this: &TileEntity, index: i32) -> Option<Rc<ItemStack>> { tile_entity_chest::slot(stacks(this), index) }

fn readFromNBT(this: &Rc<TileEntity>, nbt: &NBTTagCompound)
{
    (TILE_ENTITY.readFromNBT)(this, nbt);
    let nbttaglist: Rc<NBTBase> = nbt.getTagList("Items");
    *stacks(this).borrow_mut() = vec![None; this.getSizeInventory() as usize];
    for i in 0..nbttaglist.list().tagCount()
    {
        let tag: Rc<NBTBase> = nbttaglist.list().tagAt(i);
        let nbttagcompound: &NBTTagCompound = tag.compound();
        let b0: i8 = nbttagcompound.getByte("Slot");
        if b0 >= 0 && (b0 as usize) < stacks(this).borrow().len()
        {
            tile_entity_chest::setSlot(stacks(this), b0 as i32, Some(ItemStack::withNbt(nbttagcompound)));
        }
    }
    let furnace: &FurnaceExt = fields(this);
    furnace.m_furnaceBurnTime.set(nbt.getShort("BurnTime") as i32);
    furnace.m_furnaceCookTime.set(nbt.getShort("CookTime") as i32);
    furnace.m_currentItemBurnTime.set(getItemBurnTime(stack(this, 1)));
}

fn writeToNBT(this: &Rc<TileEntity>, nbt: &NBTTagCompound)
{
    (TILE_ENTITY.writeToNBT)(this, nbt);
    nbt.setShort("BurnTime", fields(this).m_furnaceBurnTime.get() as i16);
    nbt.setShort("CookTime", fields(this).m_furnaceCookTime.get() as i16);
    let nbttaglist: Rc<NBTBase> = NBTTagList::create();
    let length: i32 = stacks(this).borrow().len() as i32;
    for i in 0..length
    {
        if let Some(item) = stack(this, i)
        {
            let nbttagcompound: Rc<NBTBase> = NBTTagCompound::create();
            nbttagcompound.compound().setByte("Slot", i as i8);
            item.writeToNBT(nbttagcompound.compound());
            nbttaglist.list().setTag(nbttagcompound);
        }
    }
    nbt.setTag("Items", &nbttaglist);
}

pub fn getCookProgressScaled(this: &TileEntity, range: i32) -> i32
{
    fields(this).m_furnaceCookTime.get() * range / 200
}

pub fn getBurnTimeRemainingScaled(this: &TileEntity, range: i32) -> i32
{
    let furnace: &FurnaceExt = fields(this);
    if furnace.m_currentItemBurnTime.get() == 0
    {
        furnace.m_currentItemBurnTime.set(200);
    }
    furnace.m_furnaceBurnTime.get() * range / furnace.m_currentItemBurnTime.get()
}

pub fn isBurning(this: &TileEntity) -> bool { fields(this).m_furnaceBurnTime.get() > 0 }

fn updateEntity(this: &Rc<TileEntity>)
{
    let furnace: &FurnaceExt = fields(this);
    let bl: bool = furnace.m_furnaceBurnTime.get() > 0;
    let mut bl2: bool = false;
    if furnace.m_furnaceBurnTime.get() > 0
    {
        furnace.m_furnaceBurnTime.set(furnace.m_furnaceBurnTime.get() - 1);
    }
    let world: Rc<World> = this.world();
    if !world.m_multiplayerWorld.get()
    {
        if furnace.m_furnaceBurnTime.get() == 0 && canSmelt(this)
        {
            let burnTime: i32 = getItemBurnTime(stack(this, 1));
            furnace.m_furnaceBurnTime.set(burnTime);
            furnace.m_currentItemBurnTime.set(burnTime);
            if furnace.m_furnaceBurnTime.get() > 0
            {
                bl2 = true;
                if let Some(fuel) = stack(this, 1)
                {
                    fuel.m_stackSize.set(fuel.m_stackSize.get() - 1);
                    if fuel.m_stackSize.get() == 0
                    {
                        tile_entity_chest::setSlot(stacks(this), 1, None);
                    }
                }
            }
        }
        if isBurning(this) && canSmelt(this)
        {
            furnace.m_furnaceCookTime.set(furnace.m_furnaceCookTime.get() + 1);
            if furnace.m_furnaceCookTime.get() == 200
            {
                furnace.m_furnaceCookTime.set(0);
                smeltItem(this);
                bl2 = true;
            }
        }
        else
        {
            furnace.m_furnaceCookTime.set(0);
        }
        if bl != (furnace.m_furnaceBurnTime.get() > 0)
        {
            bl2 = true;
            block_furnace::updateFurnaceBlockState(
                furnace.m_furnaceBurnTime.get() > 0,
                &world,
                this.m_xCoord.get(),
                this.m_yCoord.get(),
                this.m_zCoord.get(),
            );
        }
    }
    if bl2
    {
        this.onInventoryChanged();
    }
}

fn canSmelt(this: &Rc<TileEntity>) -> bool
{
    let Some(input) = stack(this, 0)
    else
    {
        return false;
    };
    let Some(itemStack) = FurnaceRecipes::smelting().getSmeltingResult(input.getItem().m_shiftedIndex)
    else
    {
        return false;
    };
    let Some(output) = stack(this, 2)
    else
    {
        return true;
    };
    if !output.isItemEqual(&itemStack)
    {
        return false;
    }
    if output.m_stackSize.get() < this.getInventoryStackLimit() && output.m_stackSize.get() < output.getMaxStackSize()
    {
        return true;
    }
    output.m_stackSize.get() < itemStack.getMaxStackSize()
}

pub fn smeltItem(this: &Rc<TileEntity>)
{
    if !canSmelt(this)
    {
        return;
    }
    let input: Rc<ItemStack> = stack(this, 0).orNpe();
    let itemStack: Rc<ItemStack> = FurnaceRecipes::smelting().getSmeltingResult(input.getItem().m_shiftedIndex).orNpe();
    match stack(this, 2)
    {
        None => tile_entity_chest::setSlot(stacks(this), 2, Some(itemStack.copy())),
        Some(output) =>
        {
            if output.m_itemID.get() == itemStack.m_itemID.get()
            {
                output.m_stackSize.set(output.m_stackSize.get() + 1);
            }
        }
    }
    input.m_stackSize.set(input.m_stackSize.get() - 1);
    if input.m_stackSize.get() <= 0
    {
        tile_entity_chest::setSlot(stacks(this), 0, None);
    }
}

fn getItemBurnTime(item: Option<Rc<ItemStack>>) -> i32
{
    let Some(item) = item
    else
    {
        return 0;
    };
    let n: i32 = item.getItem().m_shiftedIndex;
    if n < 256 && Rc::ptr_eq(&Block::byId(n).orNpe().m_blockMaterial, &Material::wood())
    {
        return 300;
    }
    if n == Item::stick().m_shiftedIndex
    {
        return 100;
    }
    if n == Item::coal().m_shiftedIndex
    {
        return 1600;
    }
    if n == Item::bucketLava().m_shiftedIndex
    {
        return 20000;
    }
    if n == Block::sapling().m_blockID
    {
        return 100;
    }
    0
}

pub const CLASS: TileEntityClass = TileEntityClass {
    m_name: "TileEntityFurnace",
    m_parent: Some(&TILE_ENTITY),
    m_interfaces: &["IInventory"],
    getSizeInventory: |this| stacks(this).borrow().len() as i32,
    getStackInSlot: |this, index| stack(this, index),
    decrStackSize: |this, index, amount| tile_entity_chest::decrSlot(this, stacks(this), index, amount, false),
    setInventorySlotContents: |this, index, item| {
        tile_entity_chest::setSlotContents(this, stacks(this), index, item, false)
    },
    getInvName: |_| "Furnace",
    readFromNBT,
    writeToNBT,
    getInventoryStackLimit: |_| 64,
    updateEntity,
    canInteractWith: tile_entity_chest::canInteractWith,
    ..TILE_ENTITY
};

pub fn create() -> Rc<TileEntity>
{
    let ext: FurnaceExt = FurnaceExt {
        m_furnaceItemStacks: RefCell::new(vec![None; 3]),
        m_furnaceBurnTime: Cell::new(0),
        m_currentItemBurnTime: Cell::new(0),
        m_furnaceCookTime: Cell::new(0),
    };
    TileEntity::new(&CLASS, Box::new(ext))
}
