use crate::container::slot::Slot;
use crate::container::{CONTAINER, Container, ContainerClass, ICrafting, slot_furnace};
use crate::entity::Entity;
use crate::inventory::{IInventory, InventoryPlayer};
use crate::item::ItemStack;
use crate::tileentity::{TileEntity, tile_entity_furnace};
use std::cell::Cell;
use std::rc::Rc;

pub struct FurnaceExt
{
    m_furnace: Rc<TileEntity>,
    m_cookTime: Cell<i32>,
    m_burnTime: Cell<i32>,
    m_itemBurnTime: Cell<i32>,
}

fn fields(this: &Container) -> &FurnaceExt { this.ext::<FurnaceExt>() }

fn onCraftGuiOpened(this: &Rc<Container>, listener: &Rc<dyn ICrafting>)
{
    (CONTAINER.onCraftGuiOpened)(this, listener);
    let furnace: &tile_entity_furnace::FurnaceExt = tile_entity_furnace::fields(&fields(this).m_furnace);
    listener.onDataChanged(this, 0, furnace.m_furnaceCookTime.get());
    listener.onDataChanged(this, 1, furnace.m_furnaceBurnTime.get());
    listener.onDataChanged(this, 2, furnace.m_currentItemBurnTime.get());
}

fn updateCraftingResults(this: &Rc<Container>)
{
    (CONTAINER.updateCraftingResults)(this);
    let ext: &FurnaceExt = fields(this);
    let furnace: &tile_entity_furnace::FurnaceExt = tile_entity_furnace::fields(&ext.m_furnace);
    let mut i: i32 = 0;
    while i < this.m_listeners.size()
    {
        let icrafting: Rc<dyn ICrafting> = this.m_listeners.get(i);
        if ext.m_cookTime.get() != furnace.m_furnaceCookTime.get()
        {
            icrafting.onDataChanged(this, 0, furnace.m_furnaceCookTime.get());
        }
        if ext.m_burnTime.get() != furnace.m_furnaceBurnTime.get()
        {
            icrafting.onDataChanged(this, 1, furnace.m_furnaceBurnTime.get());
        }
        if ext.m_itemBurnTime.get() != furnace.m_currentItemBurnTime.get()
        {
            icrafting.onDataChanged(this, 2, furnace.m_currentItemBurnTime.get());
        }
        i += 1;
    }
    ext.m_cookTime.set(furnace.m_furnaceCookTime.get());
    ext.m_burnTime.set(furnace.m_furnaceBurnTime.get());
    ext.m_itemBurnTime.set(furnace.m_currentItemBurnTime.get());
}

fn setData(this: &Rc<Container>, id: i32, value: i32)
{
    let furnace: &tile_entity_furnace::FurnaceExt = tile_entity_furnace::fields(&fields(this).m_furnace);
    if id == 0
    {
        furnace.m_furnaceCookTime.set(value);
    }
    if id == 1
    {
        furnace.m_furnaceBurnTime.set(value);
    }
    if id == 2
    {
        furnace.m_currentItemBurnTime.set(value);
    }
}

fn isUsableByPlayer(this: &Rc<Container>, player: &Rc<Entity>) -> bool
{
    fields(this).m_furnace.canInteractWith(player)
}

fn getStackInSlot(this: &Rc<Container>, slot: i32) -> Option<Rc<ItemStack>>
{
    this.quickMove(slot, |itemStack2| {
        if slot == 2
        {
            this.moveItem(itemStack2, 3, 39, true);
        }
        else if slot >= 3 && slot < 30
        {
            this.moveItem(itemStack2, 30, 39, false);
        }
        else if slot >= 30 && slot < 39
        {
            this.moveItem(itemStack2, 3, 30, false);
        }
        else
        {
            this.moveItem(itemStack2, 3, 39, false);
        }
    })
}

pub const CLASS: ContainerClass = ContainerClass {
    m_name: "ContainerFurnace",
    m_parent: Some(&CONTAINER),
    m_interfaces: &[],
    getStackInSlot,
    isUsableByPlayer,
    onCraftGuiOpened,
    setData,
    updateCraftingResults,
    ..CONTAINER
};

pub fn create(playerInventory: &Rc<InventoryPlayer>, furnace: &Rc<TileEntity>) -> Rc<Container>
{
    let this: Rc<Container> = Container::construct(&CLASS, |_| {
        Box::new(FurnaceExt {
            m_furnace: Rc::clone(furnace),
            m_cookTime: Cell::new(0),
            m_burnTime: Cell::new(0),
            m_itemBurnTime: Cell::new(0),
        })
    });
    let furnaceInventory: Rc<dyn IInventory> = Rc::clone(furnace) as Rc<dyn IInventory>;
    let inventory: Rc<dyn IInventory> = Rc::clone(playerInventory) as Rc<dyn IInventory>;
    this.addSlot(Slot::new(Rc::clone(&furnaceInventory), 0, 56, 17));
    this.addSlot(Slot::new(Rc::clone(&furnaceInventory), 1, 56, 53));
    this.addSlot(slot_furnace::create(&playerInventory.player(), furnaceInventory, 2, 116, 35));
    for n in 0..3
    {
        for i in 0..9
        {
            this.addSlot(Slot::new(Rc::clone(&inventory), i + n * 9 + 9, 8 + i * 18, 84 + n * 18));
        }
    }
    for n in 0..9
    {
        this.addSlot(Slot::new(Rc::clone(&inventory), n, 8 + n * 18, 142));
    }
    this
}
