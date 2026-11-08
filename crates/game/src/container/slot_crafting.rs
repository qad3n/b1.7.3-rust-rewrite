use crate::block::Block;
use crate::container::slot::{SLOT, Slot, SlotClass};
use crate::entity::Entity;
use crate::inventory::IInventory;
use crate::item::{Item, ItemStack};
use crate::stats::{AchievementList, StatBase};
use jvm::throwable::OrNpe;
use std::rc::{Rc, Weak};

pub struct CraftingExt
{
    m_craftMatrix: Rc<dyn IInventory>,
    m_thePlayer: Weak<Entity>,
}

struct CraftAchievement
{
    m_itemID: fn() -> i32,
    m_achievement: fn() -> Option<Rc<StatBase>>,
}

const CRAFT_ACHIEVEMENTS: [CraftAchievement; 8] = [
    CraftAchievement { m_itemID: || Block::workbench().m_blockID, m_achievement: AchievementList::buildWorkBench },
    CraftAchievement { m_itemID: || Item::pickaxeWood().m_shiftedIndex, m_achievement: AchievementList::buildPickaxe },
    CraftAchievement { m_itemID: || Block::stoneOvenIdle().m_blockID, m_achievement: AchievementList::buildFurnace },
    CraftAchievement { m_itemID: || Item::hoeWood().m_shiftedIndex, m_achievement: AchievementList::buildHoe },
    CraftAchievement { m_itemID: || Item::bread().m_shiftedIndex, m_achievement: AchievementList::makeBread },
    CraftAchievement { m_itemID: || Item::cake().m_shiftedIndex, m_achievement: AchievementList::bakeCake },
    CraftAchievement {
        m_itemID: || Item::pickaxeStone().m_shiftedIndex,
        m_achievement: AchievementList::buildBetterPickaxe,
    },
    CraftAchievement { m_itemID: || Item::swordWood().m_shiftedIndex, m_achievement: AchievementList::buildSword },
];

fn fields(this: &Slot) -> &CraftingExt { this.ext::<CraftingExt>() }

pub fn player(this: &Slot) -> Rc<Entity> { fields(this).m_thePlayer.upgrade().expect("the player outlives its slots") }

fn onPickupFromSlot(this: &Rc<Slot>, item: Option<Rc<ItemStack>>)
{
    let item: Rc<ItemStack> = item.orNpe();
    let thePlayer: Rc<Entity> = player(this);
    item.onCrafting(&thePlayer.world(), &thePlayer);
    if let Some(row) = CRAFT_ACHIEVEMENTS.iter().find(|row| item.m_itemID.get() == (row.m_itemID)())
    {
        thePlayer.addStat((row.m_achievement)().as_ref(), 1);
    }
    let craftMatrix: &Rc<dyn IInventory> = &fields(this).m_craftMatrix;
    for i in 0..craftMatrix.getSizeInventory()
    {
        let itemstack: Option<Rc<ItemStack>> = craftMatrix.getStackInSlot(i);
        if let Some(itemstack) = itemstack
        {
            craftMatrix.decrStackSize(i, 1);
            if itemstack.getItem().hasContainerItem()
            {
                craftMatrix.setInventorySlotContents(
                    i,
                    Some(ItemStack::withItem(&itemstack.getItem().getContainerItem().orNpe())),
                );
            }
        }
    }
}

pub const CLASS: SlotClass = SlotClass {
    m_name: "SlotCrafting",
    m_parent: Some(&SLOT),
    m_interfaces: &[],
    isItemValid: |_, _| false,
    onPickupFromSlot,
    ..SLOT
};

pub fn create(
    player: &Rc<Entity>,
    craftingInventory: Rc<dyn IInventory>,
    resultInventory: Rc<dyn IInventory>,
    slot: i32,
    x: i32,
    y: i32,
) -> Rc<Slot>
{
    let ext: CraftingExt = CraftingExt { m_craftMatrix: craftingInventory, m_thePlayer: Rc::downgrade(player) };
    Slot::construct(&CLASS, Box::new(ext), resultInventory, slot, x, y)
}
