use crate::screens::ScreenType;
use game::container::slot::Slot;
use game::container::{CONTAINER, Container, ContainerClass};
use game::inventory::IInventory;
use game::item::ItemStack;
use std::rc::Rc;

const PLAYER_ROWS: i32 = 3;
const ROW_SLOTS: i32 = 9;
const SLOT_SIZE: i32 = 18;
const HOTBAR_GAP: i32 = 58;

pub struct ModContainer
{
    pub m_inventory: Rc<dyn IInventory>,
    pub m_screen: Rc<ScreenType>,
    pub m_modSlots: i32,
}

pub fn ext(container: &Container) -> &ModContainer { container.ext::<ModContainer>() }

// shift click moves between the mod slots and the player inventory, as vanilla chests do
fn transfer(this: &Rc<Container>, slot: i32) -> Option<Rc<ItemStack>>
{
    let modSlots: i32 = ext(this).m_modSlots;
    let target: Rc<Slot> = this.m_slots.get(slot);
    let stack: Rc<ItemStack> = target.getStack()?;
    let copy: Rc<ItemStack> = stack.copy();
    if slot < modSlots
    {
        this.moveItem(&stack, modSlots, this.m_slots.size(), true);
    }
    else
    {
        this.moveItem(&stack, 0, modSlots, false);
    }
    if stack.m_stackSize.get() == 0
    {
        target.putStack(None)
    }
    else
    {
        target.onSlotChanged()
    }
    Some(copy)
}

pub const MOD_CONTAINER: ContainerClass = ContainerClass {
    m_name: "ModContainer",
    m_parent: Some(&CONTAINER),
    m_interfaces: &[],
    getStackInSlot: transfer,
    isUsableByPlayer: |this, player| ext(this).m_inventory.canInteractWith(player),
    ..CONTAINER
};

pub fn create(
    screen: &Rc<ScreenType>,
    playerInventory: Rc<dyn IInventory>,
    inventory: Rc<dyn IInventory>,
) -> Rc<Container>
{
    let modSlots: i32 = screen.m_spec.slots.len() as i32;
    let this: Rc<Container> = Container::construct(&MOD_CONTAINER, |_| {
        Box::new(ModContainer { m_inventory: Rc::clone(&inventory), m_screen: Rc::clone(screen), m_modSlots: modSlots })
    });
    for slot in &screen.m_spec.slots
    {
        this.addSlot(Slot::new(Rc::clone(&inventory), slot.index, slot.x, slot.y));
    }
    if let Some([x, y]) = screen.m_spec.player_inventory
    {
        for row in 0..PLAYER_ROWS
        {
            for column in 0..ROW_SLOTS
            {
                let index: i32 = column + row * ROW_SLOTS + ROW_SLOTS;
                this.addSlot(Slot::new(
                    Rc::clone(&playerInventory),
                    index,
                    x + column * SLOT_SIZE,
                    y + row * SLOT_SIZE,
                ));
            }
        }
        for column in 0..ROW_SLOTS
        {
            this.addSlot(Slot::new(Rc::clone(&playerInventory), column, x + column * SLOT_SIZE, y + HOTBAR_GAP));
        }
    }
    this
}

// the client side of a server opened container mirrors the slot count with a plain inventory
pub fn slotCount(screen: &ScreenType) -> i32
{
    screen.m_spec.slots.iter().map(|slot| slot.index + 1).max().unwrap_or(0)
}

pub fn tileInventory(world: &game::world::World, pos: mod_protocol::abi::BlockPos) -> Option<Rc<dyn IInventory>>
{
    let tile = world.getBlockTileEntity(pos.x, pos.y, pos.z)?;
    tile.tryExt::<crate::tiles::ModTileExt>()?;
    let inventory: Rc<dyn IInventory> = tile;
    Some(inventory)
}
