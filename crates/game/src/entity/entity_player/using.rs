use super::{fields, inventory};
use crate::entity::Entity;
use crate::item::ItemStack;
use crate::world::World;
use std::rc::Rc;

// rewrite: 1.7's EntityPlayer item in use, held while the use key is down until it runs out or is let go

pub fn isUsingItem(this: &Entity) -> bool { fields(this).m_itemInUse.with(Option::is_some) }

// ticks the item has been held in use, as getItemInUseDuration
pub fn itemInUseDuration(this: &Entity) -> i32
{
    fields(this)
        .m_itemInUse
        .get()
        .map_or(0, |stack| stack.getItem().getMaxItemUseDuration(&stack) - fields(this).m_itemInUseCount.get())
}

pub fn setItemInUse(this: &Entity, stack: &Rc<ItemStack>, ticks: i32)
{
    let player = fields(this);
    if ticks <= 0
    {
        clearItemInUse(this);
        return;
    }
    player.m_itemInUse.set(Some(Rc::clone(stack)));
    player.m_itemInUseCount.set(ticks);
}

pub fn clearItemInUse(this: &Entity)
{
    let player = fields(this);
    player.m_itemInUse.set(None);
    player.m_itemInUseCount.set(0);
}

// the use key let go early, the item hears how many ticks were left
pub fn stopUsingItem(this: &Rc<Entity>)
{
    if let Some(stack) = fields(this).m_itemInUse.get()
    {
        let left: i32 = fields(this).m_itemInUseCount.get();
        stack.getItem().onPlayerStoppedUsing(&stack, &this.world(), this, left);
    }
    clearItemInUse(this);
}

// EntityPlayer.onUpdate's item in use, dropped when the held stack changes, finished when its ticks run out
pub fn tick(this: &Rc<Entity>)
{
    let Some(stack) = fields(this).m_itemInUse.get()
    else
    {
        return;
    };
    let held: Option<Rc<ItemStack>> = inventory(this).getCurrentItem();
    if !held.is_some_and(|held| Rc::ptr_eq(&held, &stack))
    {
        clearItemInUse(this);
        return;
    }
    // Forge's onUsingTick hears the count before this tick takes one off
    let count: i32 = fields(this).m_itemInUseCount.get();
    stack.getItem().onUsingItemTick(&stack, this, count);
    let left: i32 = count - 1;
    fields(this).m_itemInUseCount.set(left);
    if left > 0
    {
        return;
    }
    // the side that owns the world finishes the use, a multiplayer client only lets go and waits for its slot
    if this.world().m_multiplayerWorld.get()
    {
        clearItemInUse(this);
        return;
    }
    finish(this, &stack);
}

// EntityPlayer.onItemUseFinish, what the item gives back takes the stack's slot
fn finish(this: &Rc<Entity>, stack: &Rc<ItemStack>)
{
    let world: Rc<World> = this.world();
    let size: i32 = stack.m_stackSize.get();
    let result: Option<Rc<ItemStack>> = stack.getItem().onItemUseFinish(stack, &world, this);
    let changed: bool =
        !result.as_ref().is_some_and(|result| Rc::ptr_eq(result, stack) && result.m_stackSize.get() == size);
    if changed
    {
        let player = inventory(this);
        let slot: usize = player.m_currentItem.get() as usize;
        let kept: Option<Rc<ItemStack>> = result.filter(|result| result.m_stackSize.get() > 0);
        player.m_mainInventory.borrow_mut()[slot] = kept;
    }
    clearItemInUse(this);
}
