mod class;
mod registry;
mod tool_material;

use crate::item::ItemStack;
use crate::locale::StatCollector;
pub use class::{ITEM, ItemClass};
use jvm::field::Field;
use std::any::Any;
use std::cell::Cell;
use std::rc::Rc;
pub use tool_material::EnumToolMaterial;

pub struct Item
{
    pub m_class: &'static ItemClass,
    m_ext: Box<dyn Any>,
    pub m_shiftedIndex: i32,
    pub m_maxStackSize: Cell<i32>,
    m_maxDamage: Cell<i32>,
    pub m_iconIndex: Cell<i32>,
    pub m_bFull3D: Cell<bool>,
    pub m_hasSubtypes: Cell<bool>,
    m_containerItem: Field<Option<Rc<Item>>>,
    m_itemName: Field<Option<String>>,
}

impl Item
{
    pub fn construct(class: &'static ItemClass, ext: Box<dyn Any>, id: i32) -> Rc<Item>
    {
        let item: Rc<Item> = Rc::new(Item {
            m_class: class,
            m_ext: ext,
            m_shiftedIndex: 256 + id,
            m_maxStackSize: Cell::new(64),
            m_maxDamage: Cell::new(0),
            m_iconIndex: Cell::new(0),
            m_bFull3D: Cell::new(false),
            m_hasSubtypes: Cell::new(false),
            m_containerItem: Field::new(None),
            m_itemName: Field::new(None),
        });
        if Item::byId(256 + id).is_some()
        {
            jvm::system::println(&format!("CONFLICT @ {id}"));
        }
        registry::register(256 + id, &item);
        item
    }

    pub fn ext<T: 'static>(&self) -> &T { self.m_ext.downcast_ref::<T>().expect("item ext matches its class") }

    pub fn setIconIndex(self: Rc<Self>, sprite: i32) -> Rc<Self>
    {
        self.m_iconIndex.set(sprite);
        self
    }

    pub fn setMaxStackSize(self: Rc<Self>, size: i32) -> Rc<Self>
    {
        self.m_maxStackSize.set(size);
        self
    }

    pub fn setIconCoord(self: Rc<Self>, u: i32, v: i32) -> Rc<Self>
    {
        self.m_iconIndex.set(u + v * 16);
        self
    }

    pub fn getIconIndex(self: &Rc<Self>, stack: &Rc<ItemStack>) -> i32 { self.getIconFromDamage(stack.getItemDamage()) }

    pub fn getItemStackLimit(&self) -> i32 { self.m_maxStackSize.get() }

    pub fn getHasSubtypes(&self) -> bool { self.m_hasSubtypes.get() }

    pub fn setHasSubtypes(self: Rc<Self>, hasCustomData: bool) -> Rc<Self>
    {
        self.m_hasSubtypes.set(hasCustomData);
        self
    }

    pub fn getMaxDamage(&self) -> i32 { self.m_maxDamage.get() }

    pub fn setMaxDamage(self: Rc<Self>, damage: i32) -> Rc<Self>
    {
        self.m_maxDamage.set(damage);
        self
    }

    pub fn isDamagable(&self) -> bool { self.m_maxDamage.get() > 0 && !self.m_hasSubtypes.get() }

    pub fn setFull3D(self: Rc<Self>) -> Rc<Self>
    {
        self.m_bFull3D.set(true);
        self
    }

    pub fn setItemName(self: Rc<Self>, key: &str) -> Rc<Self>
    {
        self.m_itemName.set(Some(format!("item.{key}")));
        self
    }

    pub fn setContainerItem(self: Rc<Self>, item: Option<Rc<Item>>) -> Rc<Self>
    {
        if self.m_maxStackSize.get() > 1
        {
            jvm::throwable::throw(
                &jvm::throwable::ILLEGAL_ARGUMENT,
                "Max stack size must be 1 for items with crafting results",
            );
        }
        self.m_containerItem.set(item);
        self
    }

    pub fn getContainerItem(&self) -> Option<Rc<Item>> { self.m_containerItem.get() }

    pub fn hasContainerItem(&self) -> bool { self.m_containerItem.with(Option::is_some) }

    pub fn getStatName(self: &Rc<Self>) -> String
    {
        let name: String = self.getItemName().unwrap_or_else(|| "null".to_string());
        StatCollector::translateToLocal(&format!("{name}.name"))
    }
}
