use crate::block::{Block, block_chest};
use crate::inventory::IInventory;
use crate::item::ItemStack;
use crate::tileentity::tile_entity_furnace;
use crate::world::World;
use std::rc::Rc;

// furnace slots by beta's layout, and the faces that reach them as later versions do
mod furnace
{
    pub const INPUT: i32 = 0;
    pub const FUEL: i32 = 1;
    pub const OUTPUT: i32 = 2;
    pub const BOTTOM: i32 = 0;
    pub const TOP: i32 = 1;
}

// the inventory a block exposes, a chest joined with its neighbour, beta has no hoppers so only mods move items
pub fn inventoryAt(world: &Rc<World>, x: i32, y: i32, z: i32) -> Option<Rc<dyn IInventory>>
{
    let tile: Rc<crate::tileentity::TileEntity> = world.getBlockTileEntity(x, y, z)?;
    if !tile.instanceOf("IInventory")
    {
        return None;
    }
    if world.getBlockId(x, y, z) == Block::CHEST
    {
        return block_chest::inventory(world, x, y, z);
    }

    Some(tile as Rc<dyn IInventory>)
}

fn furnaceAt(world: &Rc<World>, x: i32, y: i32, z: i32) -> bool
{
    world.getBlockTileEntity(x, y, z).is_some_and(|tile| tile.instanceOf("TileEntityFurnace"))
}

// the slots a face reaches, a furnace takes ore on top, fuel on the sides and gives smelted items below
fn slots(world: &Rc<World>, x: i32, y: i32, z: i32, face: i32, inventory: &dyn IInventory) -> Vec<i32>
{
    if !furnaceAt(world, x, y, z)
    {
        return (0..inventory.getSizeInventory()).collect();
    }

    match face
    {
        furnace::BOTTOM => vec![furnace::OUTPUT],
        furnace::TOP => vec![furnace::INPUT],
        _ => vec![furnace::FUEL],
    }
}

fn accepts(world: &Rc<World>, x: i32, y: i32, z: i32, slot: i32, stack: &Rc<ItemStack>) -> bool
{
    if !furnaceAt(world, x, y, z)
    {
        return true;
    }

    match slot
    {
        furnace::FUEL => tile_entity_furnace::getItemBurnTime(Some(Rc::clone(stack))) > 0,
        furnace::OUTPUT => false,
        _ => true,
    }
}

// puts as much of the stack in as fits, joining equal stacks first, and gives back how many are left over
pub fn insert(world: &Rc<World>, x: i32, y: i32, z: i32, face: i32, stack: &Rc<ItemStack>, simulate: bool) -> i32
{
    let Some(inventory) = inventoryAt(world, x, y, z)
    else
    {
        return stack.m_stackSize.get();
    };

    let limit: i32 = stack.getMaxStackSize().min(inventory.getInventoryStackLimit());
    let targets: Vec<i32> = slots(world, x, y, z, face, &*inventory)
        .into_iter()
        .filter(|slot| accepts(world, x, y, z, *slot, stack))
        .collect();
    let mut left: i32 = stack.m_stackSize.get();

    for joining in [true, false]
    {
        for slot in &targets
        {
            if left <= 0
            {
                break;
            }

            let held: Option<Rc<ItemStack>> = inventory.getStackInSlot(*slot);
            let moved: i32 = match (&held, joining)
            {
                (Some(held), true) if held.isItemEqual(stack) && stack.isStackable() =>
                {
                    let moved: i32 = (limit - held.m_stackSize.get()).clamp(0, left);
                    if !simulate && moved > 0
                    {
                        held.m_stackSize.set(held.m_stackSize.get() + moved);
                    }

                    moved
                }

                (None, false) =>
                {
                    let moved: i32 = limit.min(left);

                    if !simulate
                    {
                        let placed: Rc<ItemStack> = stack.copy();
                        placed.m_stackSize.set(moved);
                        inventory.setInventorySlotContents(*slot, Some(placed));
                    }

                    moved
                }

                _ => 0,
            };
            left -= moved;
        }
    }

    if !simulate && left != stack.m_stackSize.get()
    {
        inventory.onInventoryChanged();
    }

    left
}

// takes up to max items from the first filled slot the face reaches
pub fn extract(world: &Rc<World>, x: i32, y: i32, z: i32, face: i32, max: i32, simulate: bool)
-> Option<Rc<ItemStack>>
{
    let inventory: Rc<dyn IInventory> = inventoryAt(world, x, y, z)?;
    for slot in slots(world, x, y, z, face, &*inventory)
    {
        let Some(held) = inventory.getStackInSlot(slot)
        else
        {
            continue;
        };

        let count: i32 = held.m_stackSize.get().min(max);
        if count <= 0
        {
            continue;
        }

        if simulate
        {
            let copy: Rc<ItemStack> = held.copy();
            copy.m_stackSize.set(count);
            return Some(copy);
        }

        let taken: Option<Rc<ItemStack>> = inventory.decrStackSize(slot, count);

        inventory.onInventoryChanged();
        return taken;
    }

    None
}
