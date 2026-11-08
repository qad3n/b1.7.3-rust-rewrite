use crate::item::ItemStack;
use crate::network::packet;
use crate::world::ChunkCoordinates;
use jvm::field::Field;
use jvm::hash_map::JHashMap;
use jvm::io::{DataInputStream, DataOutputStream, InputStream, IoResult, OutputStream};
use jvm::string::JString;
use jvm::throwable::{self, OrNpe};
use std::cell::Cell;
use std::rc::Rc;

#[derive(Clone)]
pub enum WatchedValue
{
    Byte(i8),
    Short(i16),
    Int(i32),
    Float(f32),
    String(JString),
    ItemStack(Rc<ItemStack>),
    ChunkCoordinates(ChunkCoordinates),
}

impl WatchedValue
{
    fn typeId(&self) -> i32
    {
        match self
        {
            Self::Byte(_) => 0,
            Self::Short(_) => 1,
            Self::Int(_) => 2,
            Self::Float(_) => 3,
            Self::String(_) => 4,
            Self::ItemStack(_) => 5,
            Self::ChunkCoordinates(_) => 6,
        }
    }

    fn javaEquals(&self, other: &Self) -> bool
    {
        match (self, other)
        {
            (Self::Byte(a), Self::Byte(b)) => a == b,
            (Self::Short(a), Self::Short(b)) => a == b,
            (Self::Int(a), Self::Int(b)) => a == b,
            (Self::Float(a), Self::Float(b)) => jvm::io::floatToIntBits(*a) == jvm::io::floatToIntBits(*b),
            (Self::String(a), Self::String(b)) => a == b,
            (Self::ItemStack(a), Self::ItemStack(b)) => Rc::ptr_eq(a, b),
            (Self::ChunkCoordinates(a), Self::ChunkCoordinates(b)) => a.equals(b),
            _ => false,
        }
    }
}

pub struct WatchableObject
{
    m_objectType: i32,
    m_dataValueId: i32,
    m_watchedObject: Field<WatchedValue>,
    m_isWatching: Cell<bool>,
}

impl WatchableObject
{
    pub fn new(objectType: i32, id: i32, value: WatchedValue) -> Self
    {
        Self {
            m_objectType: objectType,
            m_dataValueId: id,
            m_watchedObject: Field::new(value),
            m_isWatching: Cell::new(true),
        }
    }

    pub fn getDataValueId(&self) -> i32 { self.m_dataValueId }

    pub fn setObject(&self, value: WatchedValue) { self.m_watchedObject.set(value); }

    pub fn getObject(&self) -> WatchedValue { self.m_watchedObject.get() }

    pub fn getObjectType(&self) -> i32 { self.m_objectType }

    pub fn getWatching(&self) -> bool { self.m_isWatching.get() }

    pub fn setWatching(&self, dirty: bool) { self.m_isWatching.set(dirty); }
}

pub struct DataWatcher
{
    m_watchedObjects: Field<JHashMap<i32, Rc<WatchableObject>>>,
    m_objectChanged: Cell<bool>,
}

impl DataWatcher
{
    pub fn new() -> Self { Self { m_watchedObjects: Field::new(JHashMap::new()), m_objectChanged: Cell::new(false) } }

    pub fn addObject(&self, id: i32, value: WatchedValue)
    {
        let integer: i32 = value.typeId();
        if id > 31
        {
            throwable::throw(&throwable::ILLEGAL_ARGUMENT, &format!("Data value id is too big with {id}! (Max is 31)"));
        }
        if self.m_watchedObjects.with(|map| map.containsKey(&id))
        {
            throwable::throw(&throwable::ILLEGAL_ARGUMENT, &format!("Duplicate id value for {id}!"));
        }
        let object: Rc<WatchableObject> = Rc::new(WatchableObject::new(integer, id, value));
        self.m_watchedObjects.update(|map| map.put(id, object));
    }

    fn object(&self, id: i32) -> WatchedValue { self.m_watchedObjects.with(|map| map.get(&id)).orNpe().getObject() }

    pub fn getWatchableObjectByte(&self, id: i32) -> i8
    {
        match self.object(id)
        {
            WatchedValue::Byte(value) => value,
            // a wrong boxed type throws the ClassCastException of the java cast
            other => crate::port::watchedCastError(&other, "java.lang.Byte"),
        }
    }

    pub fn getWatchableObjectInt(&self, id: i32) -> i32
    {
        match self.object(id)
        {
            WatchedValue::Int(value) => value,
            // a wrong boxed type throws the ClassCastException of the java cast
            other => crate::port::watchedCastError(&other, "java.lang.Integer"),
        }
    }

    pub fn getWatchableObjectString(&self, id: i32) -> JString
    {
        match self.object(id)
        {
            WatchedValue::String(value) => value,
            // a wrong boxed type throws the ClassCastException of the java cast
            other => crate::port::watchedCastError(&other, "java.lang.String"),
        }
    }

    pub fn updateObject(&self, id: i32, value: WatchedValue)
    {
        let object: Rc<WatchableObject> = self.m_watchedObjects.with(|map| map.get(&id)).orNpe();
        if !value.javaEquals(&object.getObject())
        {
            object.setObject(value);
            object.setWatching(true);
            self.m_objectChanged.set(true);
        }
    }

    pub fn getAllWatched(&self) -> Vec<Rc<WatchableObject>> { self.m_watchedObjects.with(JHashMap::values) }

    pub fn hasObjectChanged(&self) -> bool { self.m_objectChanged.get() }

    pub fn writeObjectsInListToStream<S: OutputStream>(
        entries: Option<&[Rc<WatchableObject>]>,
        output: &mut DataOutputStream<S>,
    ) -> IoResult<()>
    {
        if let Some(entries) = entries
        {
            for entry in entries
            {
                Self::writeWatchableObject(output, entry)?;
            }
        }
        output.writeByte(127)
    }

    pub fn getChangedObjects(&self) -> Option<Vec<Rc<WatchableObject>>>
    {
        let mut arraylist: Option<Vec<Rc<WatchableObject>>> = None;
        if self.m_objectChanged.get()
        {
            for datawatcher_watchableobject in self.m_watchedObjects.with(JHashMap::values)
            {
                if datawatcher_watchableobject.getWatching()
                {
                    datawatcher_watchableobject.setWatching(false);
                    arraylist.get_or_insert_with(Vec::new).push(datawatcher_watchableobject);
                }
            }
        }
        self.m_objectChanged.set(false);
        arraylist
    }

    pub fn writeWatchableObjects<S: OutputStream>(&self, output: &mut DataOutputStream<S>) -> IoResult<()>
    {
        for entry in self.m_watchedObjects.with(JHashMap::values)
        {
            Self::writeWatchableObject(output, &entry)?;
        }
        output.writeByte(127)
    }

    fn writeWatchableObject<S: OutputStream>(output: &mut DataOutputStream<S>, entry: &WatchableObject)
    -> IoResult<()>
    {
        let i: i32 = (entry.getObjectType() << 5 | entry.getDataValueId() & 31) & 0xFF;
        output.writeByte(i)?;
        match entry.getObject()
        {
            WatchedValue::Byte(value) => output.writeByte(value as i32),
            WatchedValue::Short(value) => output.writeShort(value as i32),
            WatchedValue::Int(value) => output.writeInt(value),
            WatchedValue::Float(value) => output.writeFloat(value),
            WatchedValue::String(value) => packet::writeString(&value, output),
            WatchedValue::ItemStack(itemstack) =>
            {
                output.writeShort(itemstack.getItem().m_shiftedIndex)?;
                output.writeByte(itemstack.m_stackSize.get())?;
                output.writeShort(itemstack.getItemDamage())
            }
            WatchedValue::ChunkCoordinates(coordinates) =>
            {
                output.writeInt(coordinates.m_x)?;
                output.writeInt(coordinates.m_y)?;
                output.writeInt(coordinates.m_z)
            }
        }
    }

    pub fn readWatchableObjects<S: InputStream>(
        input: &mut DataInputStream<S>,
    ) -> IoResult<Option<Vec<Option<Rc<WatchableObject>>>>>
    {
        let mut arraylist: Option<Vec<Option<Rc<WatchableObject>>>> = None;
        let mut b0: i8 = input.readByte()?;
        while b0 != 127
        {
            let list: &mut Vec<Option<Rc<WatchableObject>>> = arraylist.get_or_insert_with(Vec::new);
            let i: i32 = (b0 as i32 & 224) >> 5;
            let j: i32 = b0 as i32 & 31;
            let value: Option<WatchedValue> = match i
            {
                0 => Some(WatchedValue::Byte(input.readByte()?)),
                1 => Some(WatchedValue::Short(input.readShort()?)),
                2 => Some(WatchedValue::Int(input.readInt()?)),
                3 => Some(WatchedValue::Float(input.readFloat()?)),
                4 => Some(WatchedValue::String(packet::readString(input, 64)?)),
                5 =>
                {
                    let short1: i16 = input.readShort()?;
                    let b1: i8 = input.readByte()?;
                    let short2: i16 = input.readShort()?;
                    Some(WatchedValue::ItemStack(ItemStack::withDamage(short1 as i32, b1 as i32, short2 as i32)))
                }
                6 =>
                {
                    let k: i32 = input.readInt()?;
                    let l: i32 = input.readInt()?;
                    let i1: i32 = input.readInt()?;
                    Some(WatchedValue::ChunkCoordinates(ChunkCoordinates::new(k, l, i1)))
                }
                _ => None,
            };
            list.push(value.map(|value| Rc::new(WatchableObject::new(i, j, value))));
            b0 = input.readByte()?;
        }
        Ok(arraylist)
    }

    pub fn updateWatchedObjectsFromList(&self, entries: &[Rc<WatchableObject>])
    {
        for entry in entries
        {
            if let Some(existing) = self.m_watchedObjects.with(|map| map.get(&entry.getDataValueId()))
            {
                existing.setObject(entry.getObject());
            }
        }
    }
}

impl Default for DataWatcher
{
    fn default() -> Self { Self::new() }
}
