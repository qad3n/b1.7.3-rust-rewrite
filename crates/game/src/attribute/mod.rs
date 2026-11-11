use crate::entity::Entity;
use common::nbt::{NBTBase, NBTTagCompound, NBTTagList};
use jvm::string::JString;
use std::cell::{Ref, RefCell, RefMut};
use std::rc::Rc;

const NBT_KEY: &str = "Attributes";

// a living number a modifier can change, its base is the value beta computes at the read site
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Attribute
{
    MaxHealth,
    MoveSpeed,
    AttackDamage,
    Armor,
    KnockbackResistance,
    SwimSpeed,
}

pub struct AttributeRow
{
    pub m_attribute: Attribute,
    pub m_name: &'static str,
}

pub const ROWS: [AttributeRow; 6] = [
    AttributeRow { m_attribute: Attribute::MaxHealth, m_name: "max_health" },
    AttributeRow { m_attribute: Attribute::MoveSpeed, m_name: "movement_speed" },
    AttributeRow { m_attribute: Attribute::AttackDamage, m_name: "attack_damage" },
    AttributeRow { m_attribute: Attribute::Armor, m_name: "armor" },
    AttributeRow { m_attribute: Attribute::KnockbackResistance, m_name: "knockback_resistance" },
    AttributeRow { m_attribute: Attribute::SwimSpeed, m_name: "swim_speed" },
];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Operation
{
    Add,
    MultiplyBase,
    MultiplyTotal,
}

pub struct OperationRow
{
    pub m_operation: Operation,
    pub m_name: &'static str,
}

pub const OPERATIONS: [OperationRow; 3] = [
    OperationRow { m_operation: Operation::Add, m_name: "add" },
    OperationRow { m_operation: Operation::MultiplyBase, m_name: "multiply_base" },
    OperationRow { m_operation: Operation::MultiplyTotal, m_name: "multiply_total" },
];

impl Attribute
{
    pub fn name(self) -> &'static str { ROWS.iter().find(|row| row.m_attribute == self).map_or("", |row| row.m_name) }

    pub fn byName(name: &str) -> Option<Self> { ROWS.iter().find(|row| row.m_name == name).map(|row| row.m_attribute) }
}

impl Operation
{
    pub fn name(self) -> &'static str
    {
        OPERATIONS.iter().find(|row| row.m_operation == self).map_or("", |row| row.m_name)
    }

    pub fn byName(name: &str) -> Option<Self>
    {
        OPERATIONS.iter().find(|row| row.m_name == name).map(|row| row.m_operation)
    }
}

// a non living entity has no modifiers and keeps its base
pub fn value(entity: &Entity, attribute: Attribute, base: f64) -> f64
{
    match entity.m_ext.living()
    {
        Some(living) => living.m_attributes.apply(attribute, base),
        None => base,
    }
}

// whole number attributes such as damage and armor drop the fraction as beta casts do
pub fn whole(entity: &Entity, attribute: Attribute, base: i32) -> i32 { value(entity, attribute, base as f64) as i32 }

#[derive(Clone, PartialEq, Debug)]
pub struct Modifier
{
    pub m_attribute: Attribute,
    pub m_id: String,
    pub m_amount: f64,
    pub m_operation: Operation,
}

// modifiers of one living entity, empty for every vanilla entity so reads return the beta base untouched
#[derive(Default)]
pub struct Attributes
{
    m_modifiers: RefCell<Vec<Modifier>>,
}

impl Attributes
{
    pub fn empty(&self) -> bool { self.m_modifiers.borrow().is_empty() }

    pub fn modifiers(&self) -> Vec<Modifier> { self.m_modifiers.borrow().clone() }

    // adds, below, base multipliers, then total multipliers, as later versions apply them
    pub fn apply(&self, attribute: Attribute, base: f64) -> f64
    {
        let modifiers: Ref<Vec<Modifier>> = self.m_modifiers.borrow();
        if !modifiers.iter().any(|modifier| modifier.m_attribute == attribute)
        {
            return base;
        }

        let of = |operation: Operation|
        {
            modifiers
                .iter()
                .filter(move |modifier| modifier.m_attribute == attribute && modifier.m_operation == operation)
                .map(|modifier| modifier.m_amount)
        };
        let added: f64 = base + of(Operation::Add).sum::<f64>();
        let scaled: f64 = added + added * of(Operation::MultiplyBase).sum::<f64>();
        of(Operation::MultiplyTotal).fold(scaled, |value, amount| value * (1.0 + amount))
    }

    // a modifier with the same attribute and id replaces the old one
    pub fn set(&self, modifier: Modifier)
    {
        let mut modifiers: RefMut<Vec<Modifier>> = self.m_modifiers.borrow_mut();
        modifiers.retain(|old| !(old.m_attribute == modifier.m_attribute && old.m_id == modifier.m_id));
        modifiers.push(modifier);
    }

    pub fn remove(&self, attribute: Attribute, id: &str)
    {
        self.m_modifiers.borrow_mut().retain(|old| !(old.m_attribute == attribute && old.m_id == id));
    }

    pub fn replace(&self, modifiers: Vec<Modifier>) { *self.m_modifiers.borrow_mut() = modifiers; }

    // written only when a modifier exists, so a vanilla save is unchanged
    pub fn write(&self, nbt: &NBTTagCompound)
    {
        if self.empty()
        {
            return;
        }

        let list: Rc<NBTBase> = NBTTagList::create();

        for modifier in self.m_modifiers.borrow().iter()
        {
            let entry: Rc<NBTBase> = NBTTagCompound::create();
            entry.compound().setString("Name", Some(JString::from(modifier.m_attribute.name())));
            entry.compound().setString("Id", Some(JString::from(modifier.m_id.as_str())));
            entry.compound().setDouble("Amount", modifier.m_amount);
            entry.compound().setString("Operation", Some(JString::from(modifier.m_operation.name())));
            list.list().setTag(entry);
        }

        nbt.setTag(NBT_KEY, &list);
    }

    pub fn read(&self, nbt: &NBTTagCompound)
    {
        if !nbt.hasKey(NBT_KEY)
        {
            return;
        }

        let list: Rc<NBTBase> = nbt.getTagList(NBT_KEY);
        let text =
            |entry: &NBTTagCompound, key: &str| entry.getString(key).map(|value| value.to_string()).unwrap_or_default();
        let mut modifiers: Vec<Modifier> = Vec::new();

        for i in 0..list.list().tagCount()
        {
            let tag: Rc<NBTBase> = list.list().tagAt(i);
            let entry: &NBTTagCompound = tag.compound();
            let (Some(attribute), Some(operation)) =
                (Attribute::byName(&text(entry, "Name")), Operation::byName(&text(entry, "Operation")))
            else
            {
                continue;
            };

            modifiers.push(Modifier
            {
                m_attribute: attribute,
                m_id: text(entry, "Id"),
                m_amount: entry.getDouble("Amount"),
                m_operation: operation,
            });
        }

        self.replace(modifiers);
    }
}
