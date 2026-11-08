use crate::block::Block;
use crate::container::slot::{SLOT, Slot, SlotClass};
use crate::container::{CONTAINER, Container, ContainerClass, slot_crafting};
use crate::crafting::CraftingManager;
use crate::entity::{Entity, entity_player};
use crate::inventory::{IInventory, InventoryCraftResult, InventoryCrafting, InventoryPlayer};
use crate::item::{ItemStack, item_armor};
use std::cell::Cell;
use std::rc::Rc;

pub struct PlayerExt
{
    pub m_craftMatrix: Rc<InventoryCrafting>,
    pub m_craftResult: Rc<dyn IInventory>,
    pub m_isSinglePlayer: Cell<bool>,
}

pub fn fields(this: &Container) -> &PlayerExt { this.ext::<PlayerExt>() }

const ARMOR_SLOT: SlotClass = SlotClass {
    m_name: "ContainerPlayer$1",
    m_parent: Some(&SLOT),
    m_interfaces: &[],
    getSlotStackLimit: |_| 1,
    isItemValid: |this, item| {
        let j1: i32 = *this.ext::<i32>();
        if item.getItem().instanceOf("ItemArmor")
        {
            item_armor::fields(&item.getItem()).m_armorType == j1
        }
        else if item.getItem().m_shiftedIndex == Block::pumpkin().m_blockID
        {
            j1 == 0
        }
        else
        {
            false
        }
    },
    ..SLOT
};

fn onCraftMatrixChanged(this: &Rc<Container>, _inventory: &dyn IInventory)
{
    let ext: &PlayerExt = fields(this);
    ext.m_craftResult
        .setInventorySlotContents(0, CraftingManager::getInstance().findMatchingRecipe(&*ext.m_craftMatrix));
}

fn onCraftGuiClosed(this: &Rc<Container>, player: &Rc<Entity>)
{
    (CONTAINER.onCraftGuiClosed)(this, player);
    let craftMatrix: &Rc<InventoryCrafting> = &fields(this).m_craftMatrix;
    for i in 0..4
    {
        let itemstack: Option<Rc<ItemStack>> = craftMatrix.getStackInSlot(i);
        if itemstack.is_some()
        {
            entity_player::dropPlayerItem(player, itemstack);
            craftMatrix.setInventorySlotContents(i, None);
        }
    }
}

fn getStackInSlot(this: &Rc<Container>, slot: i32) -> Option<Rc<ItemStack>>
{
    this.quickMove(slot, |itemstack1| {
        if slot == 0
        {
            this.moveItem(itemstack1, 9, 45, true);
        }
        else if slot >= 9 && slot < 36
        {
            this.moveItem(itemstack1, 36, 45, false);
        }
        else if slot >= 36 && slot < 45
        {
            this.moveItem(itemstack1, 9, 36, false);
        }
        else
        {
            this.moveItem(itemstack1, 9, 45, false);
        }
    })
}

pub const CLASS: ContainerClass = ContainerClass {
    m_name: "ContainerPlayer",
    m_parent: Some(&CONTAINER),
    m_interfaces: &[],
    getStackInSlot,
    isUsableByPlayer: |_, _| true,
    onCraftGuiClosed,
    onCraftMatrixChanged,
    ..CONTAINER
};

pub fn new(inventory: &Rc<InventoryPlayer>, active: bool) -> Rc<Container>
{
    let this: Rc<Container> = Container::construct(&CLASS, |this| {
        Box::new(PlayerExt {
            m_craftMatrix: Rc::new(InventoryCrafting::new(this, 2, 2)),
            m_craftResult: Rc::new(InventoryCraftResult::new()),
            m_isSinglePlayer: Cell::new(false),
        })
    });
    let ext: &PlayerExt = fields(&this);
    let playerInventory: Rc<dyn IInventory> = Rc::clone(inventory) as Rc<dyn IInventory>;
    let craftMatrix: Rc<dyn IInventory> = Rc::clone(&ext.m_craftMatrix) as Rc<dyn IInventory>;
    ext.m_isSinglePlayer.set(active);
    this.addSlot(slot_crafting::create(
        &inventory.player(),
        Rc::clone(&craftMatrix),
        Rc::clone(&ext.m_craftResult),
        0,
        144,
        36,
    ));
    for i in 0..2
    {
        for j in 0..2
        {
            this.addSlot(Slot::new(Rc::clone(&craftMatrix), j + i * 2, 88 + j * 18, 26 + i * 18));
        }
    }
    for k in 0..4
    {
        let j1: i32 = k;
        this.addSlot(Slot::construct(
            &ARMOR_SLOT,
            Box::new(j1),
            Rc::clone(&playerInventory),
            inventory.getSizeInventory() - 1 - k,
            8,
            8 + k * 18,
        ));
    }
    for l in 0..3
    {
        for k1 in 0..9
        {
            this.addSlot(Slot::new(Rc::clone(&playerInventory), k1 + (l + 1) * 9, 8 + k1 * 18, 84 + l * 18));
        }
    }
    for i1 in 0..9
    {
        this.addSlot(Slot::new(Rc::clone(&playerInventory), i1, 8 + i1 * 18, 142));
    }
    this.onCraftMatrixChanged(&*ext.m_craftMatrix);
    this
}
