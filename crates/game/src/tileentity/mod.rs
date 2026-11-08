pub mod tile_entity_chest;
pub mod tile_entity_dispenser;
pub mod tile_entity_furnace;
pub mod tile_entity_mob_spawner;
pub mod tile_entity_note;
pub mod tile_entity_piston;
pub mod tile_entity_sign;

use crate::block::{Block, block_juke_box};
use crate::entity::Entity;
use crate::inventory::IInventory;
use crate::item::ItemStack;
use crate::network::packet::Packet;
use crate::world::World;
use common::nbt::NBTTagCompound;
use jvm::field::Field;
use jvm::string::JString;
use jvm::throwable::OrNpe;
use std::any::Any;
use std::cell::Cell;
use std::rc::{Rc, Weak};

crate::vtable! {
    TileEntityClass for TileEntity {
        fn canInteractWith(player: &Rc<Entity>) -> bool;
        fn decrStackSize(slot: i32, amount: i32) -> Option<Rc<ItemStack>>;
        fn getBlockMetadata() -> i32;
        fn getDescriptionPacket() -> Option<Box<dyn Packet>>;
        fn getInvName() -> &'static str;
        fn getInventoryStackLimit() -> i32;
        fn getSizeInventory() -> i32;
        fn getStackInSlot(slot: i32) -> Option<Rc<ItemStack>>;
        fn readFromNBT(nbt: &NBTTagCompound);
        fn setInventorySlotContents(slot: i32, item: Option<Rc<ItemStack>>);
        fn updateEntity();
        fn writeToNBT(nbt: &NBTTagCompound);
    }
}

fn notInventory(name: &str) -> !
{
    jvm::throwable::throw(&jvm::throwable::ERROR, &format!("java.lang.AbstractMethodError: {name}"))
}

pub const TILE_ENTITY: TileEntityClass = TileEntityClass {
    m_name: "TileEntity",
    m_parent: None,
    m_interfaces: &[],
    canInteractWith: |_, _| notInventory("canInteractWith"),
    decrStackSize: |_, _, _| notInventory("decrStackSize"),
    getDescriptionPacket: |_| None,
    getBlockMetadata: |this| {
        this.world().getBlockMetadata(this.m_xCoord.get(), this.m_yCoord.get(), this.m_zCoord.get())
    },
    getInvName: |_| notInventory("getInvName"),
    getInventoryStackLimit: |_| notInventory("getInventoryStackLimit"),
    getSizeInventory: |_| notInventory("getSizeInventory"),
    getStackInSlot: |_, _| notInventory("getStackInSlot"),
    readFromNBT: |this, nbt| {
        this.m_xCoord.set(nbt.getInteger("x"));
        this.m_yCoord.set(nbt.getInteger("y"));
        this.m_zCoord.set(nbt.getInteger("z"));
    },
    setInventorySlotContents: |_, _, _| notInventory("setInventorySlotContents"),
    updateEntity: |_| {},
    writeToNBT: |this, nbt| {
        let Some(kind) = TileEntityKind::forClass(this.className())
        else
        {
            jvm::throwable::throw(
                &jvm::throwable::RUNTIME,
                &format!("class net.minecraft.src.{} is missing a mapping! This is a bug!", this.className()),
            );
        };
        nbt.setString("id", Some(JString::from(kind.row().m_id)));
        nbt.setInteger("x", this.m_xCoord.get());
        nbt.setInteger("y", this.m_yCoord.get());
        nbt.setInteger("z", this.m_zCoord.get());
    },
};

pub struct TileEntity
{
    pub m_class: &'static TileEntityClass,
    m_ext: Box<dyn Any>,
    m_this: Weak<TileEntity>,
    pub m_worldObj: Field<Option<Rc<World>>>,
    pub m_xCoord: Cell<i32>,
    pub m_yCoord: Cell<i32>,
    pub m_zCoord: Cell<i32>,
    pub m_removed: Cell<bool>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TileEntityKind
{
    Furnace,
    Chest,
    RecordPlayer,
    Dispenser,
    Sign,
    MobSpawner,
    Note,
    Piston,
}

pub struct TileEntityRow
{
    pub m_kind: TileEntityKind,
    pub m_className: &'static str,
    pub m_id: &'static str,
    m_create: fn() -> Rc<TileEntity>,
}

const MAPPINGS: [TileEntityRow; 8] = [
    TileEntityRow {
        m_kind: TileEntityKind::Furnace,
        m_className: "TileEntityFurnace",
        m_id: "Furnace",
        m_create: tile_entity_furnace::create,
    },
    TileEntityRow {
        m_kind: TileEntityKind::Chest,
        m_className: "TileEntityChest",
        m_id: "Chest",
        m_create: tile_entity_chest::create,
    },
    TileEntityRow {
        m_kind: TileEntityKind::RecordPlayer,
        m_className: "BlockJukeBox$TileEntityRecordPlayer",
        m_id: "RecordPlayer",
        m_create: block_juke_box::newRecordPlayer,
    },
    TileEntityRow {
        m_kind: TileEntityKind::Dispenser,
        m_className: "TileEntityDispenser",
        m_id: "Trap",
        m_create: tile_entity_dispenser::create,
    },
    TileEntityRow {
        m_kind: TileEntityKind::Sign,
        m_className: "TileEntitySign",
        m_id: "Sign",
        m_create: tile_entity_sign::create,
    },
    TileEntityRow {
        m_kind: TileEntityKind::MobSpawner,
        m_className: "TileEntityMobSpawner",
        m_id: "MobSpawner",
        m_create: tile_entity_mob_spawner::create,
    },
    TileEntityRow {
        m_kind: TileEntityKind::Note,
        m_className: "TileEntityNote",
        m_id: "Music",
        m_create: tile_entity_note::create,
    },
    TileEntityRow {
        m_kind: TileEntityKind::Piston,
        m_className: "TileEntityPiston",
        m_id: "Piston",
        m_create: tile_entity_piston::createEmpty,
    },
];

impl TileEntityKind
{
    pub fn row(self) -> &'static TileEntityRow
    {
        match MAPPINGS.iter().find(|row| row.m_kind == self)
        {
            Some(row) => row,
            None => unreachable!("every kind has a mapping row"),
        }
    }

    pub fn forClass(className: &str) -> Option<TileEntityKind>
    {
        MAPPINGS.iter().find(|row| row.m_className == className).map(|row| row.m_kind)
    }

    pub fn forId(id: &JString) -> Option<TileEntityKind>
    {
        MAPPINGS.iter().find(|row| JString::from(row.m_id) == *id).map(|row| row.m_kind)
    }

    pub fn newInstance(self) -> Rc<TileEntity> { (self.row().m_create)() }
}

impl TileEntity
{
    pub fn new(class: &'static TileEntityClass, ext: Box<dyn Any>) -> Rc<TileEntity>
    {
        Rc::new_cyclic(|this| TileEntity {
            m_class: class,
            m_ext: ext,
            m_this: Weak::clone(this),
            m_worldObj: Field::new(None),
            m_xCoord: Cell::new(0),
            m_yCoord: Cell::new(0),
            m_zCoord: Cell::new(0),
            m_removed: Cell::new(false),
        })
    }

    pub fn ext<T: 'static>(&self) -> &T { self.m_ext.downcast_ref::<T>().expect("tile entity ext matches its class") }

    pub fn world(&self) -> Rc<World> { self.m_worldObj.get().orNpe() }

    fn this(&self) -> Rc<TileEntity> { self.m_this.upgrade().expect("a live tile entity is owned by an Rc") }

    pub fn castNullable(tileEntity: Option<Rc<TileEntity>>, className: &str) -> Option<Rc<TileEntity>>
    {
        // forwards to port castNullable until item_sign calls port directly
        crate::port::tile_entity::castNullable(tileEntity, className)
    }

    pub fn createAndLoadEntity(nbt: &NBTTagCompound) -> Option<Rc<TileEntity>>
    {
        let id: Option<JString> = nbt.getString("id");
        let tileEntity: Option<Rc<TileEntity>> =
            id.as_ref().and_then(TileEntityKind::forId).map(TileEntityKind::newInstance);
        match &tileEntity
        {
            Some(entity) => entity.readFromNBT(nbt),
            None => jvm::system::println(&format!("Skipping TileEntity with id {}", JString::display(id.as_ref()))),
        }
        tileEntity
    }

    pub fn onInventoryChanged(self: &Rc<Self>)
    {
        if let Some(world) = self.m_worldObj.get()
        {
            world.notifyBlockEntityChanged(self.m_xCoord.get(), self.m_yCoord.get(), self.m_zCoord.get(), self);
        }
    }

    pub fn getDistanceFrom(&self, x: f64, y: f64, z: f64) -> f64
    {
        let d: f64 = self.m_xCoord.get() as f64 + 0.5 - x;
        let d2: f64 = self.m_yCoord.get() as f64 + 0.5 - y;
        let d3: f64 = self.m_zCoord.get() as f64 + 0.5 - z;
        d * d + d2 * d2 + d3 * d3
    }

    pub fn getBlockType(&self) -> Option<Rc<Block>>
    {
        Block::byId(self.world().getBlockId(self.m_xCoord.get(), self.m_yCoord.get(), self.m_zCoord.get()))
    }

    pub fn isRemoved(&self) -> bool { self.m_removed.get() }

    pub fn markRemoved(&self) { self.m_removed.set(true); }

    pub fn cancelRemoval(&self) { self.m_removed.set(false); }
}

impl IInventory for TileEntity
{
    fn getSizeInventory(&self) -> i32 { self.this().getSizeInventory() }

    fn getStackInSlot(&self, slot: i32) -> Option<Rc<ItemStack>> { self.this().getStackInSlot(slot) }

    fn decrStackSize(&self, slot: i32, amount: i32) -> Option<Rc<ItemStack>> { self.this().decrStackSize(slot, amount) }

    fn setInventorySlotContents(&self, slot: i32, item: Option<Rc<ItemStack>>)
    {
        self.this().setInventorySlotContents(slot, item);
    }

    fn getInvName(&self) -> String { self.this().getInvName().to_string() }

    fn getInventoryStackLimit(&self) -> i32 { self.this().getInventoryStackLimit() }

    fn onInventoryChanged(&self) { self.this().onInventoryChanged(); }

    fn canInteractWith(&self, player: &Rc<Entity>) -> bool { self.this().canInteractWith(player) }
}
