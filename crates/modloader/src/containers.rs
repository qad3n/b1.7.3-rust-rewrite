use crate::screens::ScreenType;
use game::container::slot::Slot;
use game::container::{CONTAINER, Container, ContainerClass};
use game::crafting::{CraftingGrid, CraftingManager};
use game::entity::{Entity, entity_player};
use game::inventory::IInventory;
use game::item::ItemStack;
use mod_protocol::Identifier;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

const PLAYER_ROWS: i32 = 3;
const ROW_SLOTS: i32 = 9;
const SLOT_SIZE: i32 = 18;
const HOTBAR_GAP: i32 = 58;

const GRID_SIDE: i32 = 3;
const GRID_SLOTS: i32 = 9;
const RESULT_LIMIT: i32 = 64;

// a crafting result slot a screen asked for, over nine of its own slots
#[derive(Clone, Copy)]
pub struct CraftSlot
{
    pub m_first: i32,
    pub m_x: i32,
    pub m_y: i32,
}

thread_local!
{
    static CRAFT_SLOTS: RefCell<HashMap<Identifier, CraftSlot>> = RefCell::new(HashMap::new());
}

pub fn setCraftSlot(screen: Identifier, slot: CraftSlot)
{
    CRAFT_SLOTS.with(|slots| slots.borrow_mut().insert(screen, slot));
}

// nine slots of a mod inventory seen as a crafting grid, so SlotCrafting takes from them as from a workbench
struct Grid
{
    m_inner: Rc<dyn IInventory>,
    m_first: i32,
}

impl IInventory for Grid
{
    fn getSizeInventory(&self) -> i32 { GRID_SLOTS }

    fn getStackInSlot(&self, slot: i32) -> Option<Rc<ItemStack>> { self.m_inner.getStackInSlot(self.m_first + slot) }

    fn decrStackSize(&self, slot: i32, amount: i32) -> Option<Rc<ItemStack>>
    {
        self.m_inner.decrStackSize(self.m_first + slot, amount)
    }

    fn setInventorySlotContents(&self, slot: i32, item: Option<Rc<ItemStack>>)
    {
        self.m_inner.setInventorySlotContents(self.m_first + slot, item);
    }

    fn getInvName(&self) -> String { self.m_inner.getInvName() }

    fn getInventoryStackLimit(&self) -> i32 { self.m_inner.getInventoryStackLimit() }

    fn onInventoryChanged(&self) { self.m_inner.onInventoryChanged(); }

    fn canInteractWith(&self, player: &Rc<Entity>) -> bool { self.m_inner.canInteractWith(player) }
}

impl CraftingGrid for Grid
{
    fn getItem(&self, column: i32, row: i32) -> Option<Rc<ItemStack>>
    {
        let inside: bool = (0..GRID_SIDE).contains(&column) && (0..GRID_SIDE).contains(&row);
        if inside
        {
            self.getStackInSlot(row * GRID_SIDE + column)
        }
        else
        {
            None
        }
    }
}

// InventoryCraftResult that always holds what the grid makes right now
struct Output
{
    m_grid: Rc<Grid>,
}

impl Output
{
    fn result(&self) -> Option<Rc<ItemStack>> { CraftingManager::getInstance().findMatchingRecipe(&*self.m_grid) }
}

impl IInventory for Output
{
    fn getSizeInventory(&self) -> i32 { 1 }

    fn getStackInSlot(&self, _slot: i32) -> Option<Rc<ItemStack>> { self.result() }

    fn decrStackSize(&self, _slot: i32, _amount: i32) -> Option<Rc<ItemStack>> { self.result() }

    fn setInventorySlotContents(&self, _slot: i32, _item: Option<Rc<ItemStack>>) {}

    fn getInvName(&self) -> String { "Result".to_owned() }

    fn getInventoryStackLimit(&self) -> i32 { RESULT_LIMIT }

    fn onInventoryChanged(&self) {}

    fn canInteractWith(&self, _player: &Rc<Entity>) -> bool { true }
}

pub struct ModContainer
{
    pub m_inventory: Rc<dyn IInventory>,
    pub m_screen: Rc<ScreenType>,
    pub m_modSlots: i32,
    // slots opened without a block, emptied at the player on close
    pub m_scratch: bool,
    // the crafting result slot's index, after the mod slots
    pub m_craft: Option<i32>,
}

pub fn ext(container: &Container) -> &ModContainer { container.ext::<ModContainer>() }

// shift click moves between the mod slots and the player inventory, as vanilla chests do
fn transfer(this: &Rc<Container>, slot: i32) -> Option<Rc<ItemStack>>
{
    let modSlots: i32 = ext(this).m_modSlots;
    let target: Rc<Slot> = this.m_slots.get(slot);
    let stack: Rc<ItemStack> = target.getStack()?;
    let copy: Rc<ItemStack> = stack.copy();

    // a crafting result moves out whole and is taken from the grid, as ContainerWorkbench does with its slot 0
    if ext(this).m_craft == Some(slot)
    {
        this.moveItem(&stack, modSlots, this.m_slots.size(), true);
        if stack.m_stackSize.get() == copy.m_stackSize.get()
        {
            return None;
        }

        target.onPickupFromSlot(Some(stack));
        return Some(copy);
    }

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

pub const MOD_CONTAINER: ContainerClass = ContainerClass
{
    m_name: "ModContainer",
    m_parent: Some(&CONTAINER),
    m_interfaces: &[],
    getStackInSlot: transfer,
    isUsableByPlayer: |this, player| ext(this).m_inventory.canInteractWith(player),
    onCraftGuiClosed: |this, player|
    {
        (CONTAINER.onCraftGuiClosed)(this, player);

        let container: &ModContainer = ext(this);
        if !container.m_scratch || player.world().m_multiplayerWorld.get()
        {
            return;
        }

        for slot in 0..container.m_inventory.getSizeInventory()
        {
            if let Some(stack) = container.m_inventory.getStackInSlot(slot)
            {
                container.m_inventory.setInventorySlotContents(slot, None);
                entity_player::dropPlayerItem(player, Some(stack));
            }
        }
    },
    ..CONTAINER
};

pub fn create(
    screen: &Rc<ScreenType>,
    player: &Rc<Entity>,
    playerInventory: Rc<dyn IInventory>,
    inventory: Rc<dyn IInventory>,
    scratch: bool,
) -> Rc<Container>
{
    let craft: Option<CraftSlot> = CRAFT_SLOTS.with(|slots| slots.borrow().get(&screen.m_id).copied());
    let ownSlots: i32 = screen.m_spec.slots.len() as i32;
    let modSlots: i32 = ownSlots + i32::from(craft.is_some());

    let this: Rc<Container> = Container::construct(&MOD_CONTAINER, |_|
    {
        Box::new(ModContainer
        {
            m_inventory: Rc::clone(&inventory),
            m_screen: Rc::clone(screen),
            m_modSlots: modSlots,
            m_scratch: scratch,
            m_craft: craft.map(|_| ownSlots),
        })
    });

    for slot in &screen.m_spec.slots
    {
        this.addSlot(Slot::new(Rc::clone(&inventory), slot.index, slot.x, slot.y));
    }

    if let Some(craft) = craft
    {
        let grid: Rc<Grid> = Rc::new(Grid { m_inner: Rc::clone(&inventory), m_first: craft.m_first });
        let output: Rc<dyn IInventory> = Rc::new(Output { m_grid: Rc::clone(&grid) });
        this.addSlot(game::container::slot_crafting::create(player, grid, output, 0, craft.m_x, craft.m_y));
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

// a block-less container screen's slots, held only while it is open
pub fn scratchInventory(screen: &ScreenType) -> Rc<dyn IInventory>
{
    Rc::new(game::inventory::InventoryBasic::new(
        jvm::string::JString::from(screen.m_spec.title.as_str()),
        slotCount(screen),
    ))
}

pub fn tileInventory(world: &game::world::World, pos: mod_protocol::abi::BlockPos) -> Option<Rc<dyn IInventory>>
{
    let tile: Rc<game::tileentity::TileEntity> = world.getBlockTileEntity(pos.x, pos.y, pos.z)?;
    tile.tryExt::<crate::tiles::ModTileExt>()?;

    let inventory: Rc<dyn IInventory> = tile;
    Some(inventory)
}
