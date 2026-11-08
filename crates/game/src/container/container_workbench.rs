use crate::block::Block;
use crate::container::slot::Slot;
use crate::container::{CONTAINER, Container, ContainerClass, slot_crafting};
use crate::crafting::CraftingManager;
use crate::entity::{Entity, entity_player};
use crate::inventory::{IInventory, InventoryCraftResult, InventoryCrafting, InventoryPlayer};
use crate::item::ItemStack;
use crate::world::World;
use std::rc::Rc;

pub struct WorkbenchExt
{
    pub m_craftMatrix: Rc<InventoryCrafting>,
    pub m_craftResult: Rc<dyn IInventory>,
    m_world: Rc<World>,
    m_x: i32,
    m_y: i32,
    m_z: i32,
}

pub fn fields(this: &Container) -> &WorkbenchExt { this.ext::<WorkbenchExt>() }

fn onCraftMatrixChanged(this: &Rc<Container>, _inventory: &dyn IInventory)
{
    let ext: &WorkbenchExt = fields(this);
    ext.m_craftResult
        .setInventorySlotContents(0, CraftingManager::getInstance().findMatchingRecipe(&*ext.m_craftMatrix));
}

fn onCraftGuiClosed(this: &Rc<Container>, player: &Rc<Entity>)
{
    (CONTAINER.onCraftGuiClosed)(this, player);
    let ext: &WorkbenchExt = fields(this);
    if ext.m_world.m_multiplayerWorld.get()
    {
        return;
    }
    for i in 0..9
    {
        let stackInSlot: Option<Rc<ItemStack>> = ext.m_craftMatrix.getStackInSlot(i);
        if stackInSlot.is_some()
        {
            entity_player::dropPlayerItem(player, stackInSlot);
        }
    }
}

fn isUsableByPlayer(this: &Rc<Container>, player: &Rc<Entity>) -> bool
{
    let ext: &WorkbenchExt = fields(this);
    if ext.m_world.getBlockId(ext.m_x, ext.m_y, ext.m_z) != Block::workbench().m_blockID
    {
        return false;
    }
    !(player.getDistanceSq(ext.m_x as f64 + 0.5, ext.m_y as f64 + 0.5, ext.m_z as f64 + 0.5) > 64.0)
}

fn getStackInSlot(this: &Rc<Container>, slot: i32) -> Option<Rc<ItemStack>>
{
    this.quickMove(slot, |itemStack2| {
        if slot == 0
        {
            this.moveItem(itemStack2, 10, 46, true);
        }
        else if slot >= 10 && slot < 37
        {
            this.moveItem(itemStack2, 37, 46, false);
        }
        else if slot >= 37 && slot < 46
        {
            this.moveItem(itemStack2, 10, 37, false);
        }
        else
        {
            this.moveItem(itemStack2, 10, 46, false);
        }
    })
}

pub const CLASS: ContainerClass = ContainerClass {
    m_name: "ContainerWorkbench",
    m_parent: Some(&CONTAINER),
    m_interfaces: &[],
    getStackInSlot,
    isUsableByPlayer,
    onCraftGuiClosed,
    onCraftMatrixChanged,
    ..CONTAINER
};

pub fn create(playerInventory: &Rc<InventoryPlayer>, world: &Rc<World>, x: i32, y: i32, z: i32) -> Rc<Container>
{
    let this: Rc<Container> = Container::construct(&CLASS, |this| {
        Box::new(WorkbenchExt {
            m_craftMatrix: Rc::new(InventoryCrafting::new(this, 3, 3)),
            m_craftResult: Rc::new(InventoryCraftResult::new()),
            m_world: Rc::clone(world),
            m_x: x,
            m_y: y,
            m_z: z,
        })
    });
    let ext: &WorkbenchExt = fields(&this);
    let inventory: Rc<dyn IInventory> = Rc::clone(playerInventory) as Rc<dyn IInventory>;
    let craftMatrix: Rc<dyn IInventory> = Rc::clone(&ext.m_craftMatrix) as Rc<dyn IInventory>;
    this.addSlot(slot_crafting::create(
        &playerInventory.player(),
        Rc::clone(&craftMatrix),
        Rc::clone(&ext.m_craftResult),
        0,
        124,
        35,
    ));
    for n2 in 0..3
    {
        for n in 0..3
        {
            this.addSlot(Slot::new(Rc::clone(&craftMatrix), n + n2 * 3, 30 + n * 18, 17 + n2 * 18));
        }
    }
    for n2 in 0..3
    {
        for n in 0..9
        {
            this.addSlot(Slot::new(Rc::clone(&inventory), n + n2 * 9 + 9, 8 + n * 18, 84 + n2 * 18));
        }
    }
    for n2 in 0..9
    {
        this.addSlot(Slot::new(Rc::clone(&inventory), n2, 8 + n2 * 18, 142));
    }
    this.onCraftMatrixChanged(&*ext.m_craftMatrix);
    this
}
