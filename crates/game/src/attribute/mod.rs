use crate::entity::Entity;
use common::nbt::{NBTBase, NBTTagCompound, NBTTagList, Unread};
use jvm::string::JString;
use std::cell::{Ref, RefCell, RefMut};
use std::rc::Rc;

const NBT_KEY: &str = "Attributes";

// a living number a modifier can change, an index into the attribute registry where vanilla rows come first
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Attribute(usize);

pub struct AttributeType
{
    pub m_name: String,
    // what a read with no base of its own starts from, the vanilla rows use the beta value of their usual read site
    pub m_default: f64,
    pub m_min: f64,
    pub m_max: f64,
}

struct VanillaRow
{
    m_name: &'static str,
    m_default: f64,
}

const NAMESPACE: &str = "minecraft:";
pub const VANILLA_MAX_HEALTH: i32 = 20;
pub const BLOCK_REACH: f64 = 4.0;
pub const ENTITY_REACH: f64 = 3.0;

// order matches the Attribute consts
const VANILLA: [VanillaRow; 8] = [
    VanillaRow { m_name: "minecraft:max_health", m_default: 20.0 },
    VanillaRow { m_name: "minecraft:movement_speed", m_default: 0.1 },
    VanillaRow { m_name: "minecraft:attack_damage", m_default: 0.0 },
    VanillaRow { m_name: "minecraft:armor", m_default: 0.0 },
    VanillaRow { m_name: "minecraft:knockback_resistance", m_default: 0.0 },
    VanillaRow { m_name: "minecraft:swim_speed", m_default: 0.02 },
    // beta's survival reach for blocks and for the entities it points at
    VanillaRow { m_name: "minecraft:block_interaction_range", m_default: BLOCK_REACH },
    VanillaRow { m_name: "minecraft:entity_interaction_range", m_default: ENTITY_REACH },
];

thread_local!
{
    static TYPES: RefCell<Vec<AttributeType>> = RefCell::new(
        VANILLA
            .iter()
            .map(|row| AttributeType
            {
                m_name: row.m_name.to_owned(),
                m_default: row.m_default,
                m_min: f64::MIN,
                m_max: f64::MAX,
            })
            .collect(),
    );
}

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
    pub const MAX_HEALTH: Self = Self(0);
    pub const MOVE_SPEED: Self = Self(1);
    pub const ATTACK_DAMAGE: Self = Self(2);
    pub const ARMOR: Self = Self(3);
    pub const KNOCKBACK_RESISTANCE: Self = Self(4);
    pub const SWIM_SPEED: Self = Self(5);
    pub const BLOCK_REACH: Self = Self(6);
    pub const ENTITY_REACH: Self = Self(7);

    pub fn name(self) -> String { TYPES.with(|types| types.borrow()[self.0].m_name.clone()) }

    pub fn default(self) -> f64 { TYPES.with(|types| types.borrow()[self.0].m_default) }

    // saves from before the registry name vanilla attributes without their namespace
    pub fn byName(name: &str) -> Option<Self>
    {
        let full: String = if name.contains(':')
        {
            name.to_owned()
        }
        else
        {
            format!("{NAMESPACE}{name}")
        };
        TYPES.with(|types| types.borrow().iter().position(|kind| kind.m_name == full).map(Self))
    }

    fn clamp(self, value: f64) -> f64
    {
        TYPES.with(|types|
        {
            let kind: &AttributeType = &types.borrow()[self.0];
            value.clamp(kind.m_min, kind.m_max)
        })
    }
}

// registering a name again replaces its row and keeps its index, so modifiers already set stay on it
pub fn register(kind: AttributeType) -> Attribute
{
    TYPES.with(|types|
    {
        let mut types: RefMut<Vec<AttributeType>> = types.borrow_mut();
        match types.iter().position(|old| old.m_name == kind.m_name)
        {
            Some(index) =>
            {
                types[index] = kind;
                Attribute(index)
            }

            None =>
            {
                types.push(kind);
                Attribute(types.len() - 1)
            }
        }
    })
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

// the health an entity heals to, vanilla caps every living entity at 20
pub fn maxHealth(entity: &Entity) -> i32 { whole(entity, Attribute::MAX_HEALTH, VANILLA_MAX_HEALTH) }

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
    m_unread: Unread,
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
        let total: f64 = of(Operation::MultiplyTotal).fold(scaled, |value, amount| value * (1.0 + amount));
        attribute.clamp(total)
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
        if self.empty() && self.m_unread.empty()
        {
            return;
        }

        let list: Rc<NBTBase> = NBTTagList::create();

        for modifier in self.m_modifiers.borrow().iter()
        {
            let entry: Rc<NBTBase> = NBTTagCompound::create();
            entry.compound().setString("Name", Some(JString::from(modifier.m_attribute.name().as_str())));
            entry.compound().setString("Id", Some(JString::from(modifier.m_id.as_str())));
            entry.compound().setDouble("Amount", modifier.m_amount);
            entry.compound().setString("Operation", Some(JString::from(modifier.m_operation.name())));
            list.list().setTag(entry);
        }

        self.m_unread.writeInto(&list);
        nbt.setTag(NBT_KEY, &list);
    }

    // modifiers of an attribute no mod registers any more are kept as saved
    pub fn read(&self, nbt: &NBTTagCompound)
    {
        self.m_unread.clear();

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
            let name: String = text(entry, "Name");
            let (Some(attribute), Some(operation)) =
                (Attribute::byName(&name), Operation::byName(&text(entry, "Operation")))
            else
            {
                self.m_unread.keep(tag);
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
