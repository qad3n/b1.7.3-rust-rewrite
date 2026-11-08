use crate::container::slot::{SLOT, Slot, SlotClass};
use crate::entity::Entity;
use crate::inventory::IInventory;
use crate::item::{Item, ItemStack};
use crate::stats::AchievementList;
use jvm::throwable::OrNpe;
use std::rc::{Rc, Weak};

pub struct FurnaceExt
{
    m_thePlayer: Weak<Entity>,
}

fn player(this: &Slot) -> Rc<Entity>
{
    this.ext::<FurnaceExt>().m_thePlayer.upgrade().expect("the player outlives its slots")
}

fn onPickupFromSlot(this: &Rc<Slot>, item: Option<Rc<ItemStack>>)
{
    let stack: &Rc<ItemStack> = item.as_ref().orNpe();
    let thePlayer: Rc<Entity> = player(this);
    stack.onCrafting(&thePlayer.world(), &thePlayer);
    if stack.m_itemID.get() == Item::ingotIron().m_shiftedIndex
    {
        thePlayer.addStat(AchievementList::acquireIron().as_ref(), 1);
    }
    if stack.m_itemID.get() == Item::fishCooked().m_shiftedIndex
    {
        thePlayer.addStat(AchievementList::cookFish().as_ref(), 1);
    }
    (SLOT.onPickupFromSlot)(this, item);
}

pub const CLASS: SlotClass = SlotClass {
    m_name: "SlotFurnace",
    m_parent: Some(&SLOT),
    m_interfaces: &[],
    isItemValid: |_, _| false,
    onPickupFromSlot,
    ..SLOT
};

pub fn create(player: &Rc<Entity>, resultInventory: Rc<dyn IInventory>, slot: i32, x: i32, y: i32) -> Rc<Slot>
{
    let ext: FurnaceExt = FurnaceExt { m_thePlayer: Rc::downgrade(player) };
    Slot::construct(&CLASS, Box::new(ext), resultInventory, slot, x, y)
}
