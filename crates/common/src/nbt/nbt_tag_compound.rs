use crate::nbt::*;
use jvm::field::Field;
use jvm::hash_map::JHashMap;
use jvm::io::{DataInputStream, DataOutputStream, InputStream, IoResult, OutputStream};
use jvm::string::JString;
use std::rc::Rc;

pub struct NBTTagCompound
{
    m_tagMap: Field<JHashMap<JString, Rc<NBTBase>>>,
}

impl NBTTagCompound
{
    pub fn create() -> Rc<NBTBase> { NBTBase::wrap(NBTTag::Compound(Self { m_tagMap: Field::new(JHashMap::new()) })) }

    pub fn writeTagContents<S: OutputStream>(&self, output: &mut DataOutputStream<S>) -> IoResult<()>
    {
        for nbtBase in self.getValues()
        {
            NBTBase::writeTag(&nbtBase, output)?;
        }
        output.writeByte(0)
    }

    pub fn readTagContents<S: InputStream>(&self, input: &mut DataInputStream<S>) -> IoResult<()>
    {
        self.m_tagMap.update(JHashMap::clear);
        loop
        {
            let nbtBase: Rc<NBTBase> = NBTBase::readTag(input)?;
            if nbtBase.getType() == 0
            {
                break;
            }
            self.m_tagMap.update(|map| map.put(nbtBase.getKey(), nbtBase));
        }
        Ok(())
    }

    pub fn getValues(&self) -> Vec<Rc<NBTBase>> { self.m_tagMap.with(JHashMap::values) }

    fn put(&self, key: &str, tag: Rc<NBTBase>)
    {
        let key: JString = JString::from(key);
        let tag: Rc<NBTBase> = tag.setKey(Some(key.clone()));
        self.m_tagMap.update(|map| map.put(key, tag));
    }

    fn lookup(&self, key: &str) -> Option<Rc<NBTBase>> { self.m_tagMap.with(|map| map.get(&JString::from(key))) }

    pub fn setTag(&self, key: &str, element: &Rc<NBTBase>) { self.put(key, Rc::clone(element)); }

    pub fn setByte(&self, key: &str, value: i8) { self.put(key, NBTTagByte::create(value)); }

    pub fn setShort(&self, key: &str, value: i16) { self.put(key, NBTTagShort::create(value)); }

    pub fn setInteger(&self, key: &str, value: i32) { self.put(key, NBTTagInt::create(value)); }

    pub fn setLong(&self, key: &str, value: i64) { self.put(key, NBTTagLong::create(value)); }

    pub fn setFloat(&self, key: &str, value: f32) { self.put(key, NBTTagFloat::create(value)); }

    pub fn setDouble(&self, key: &str, value: f64) { self.put(key, NBTTagDouble::create(value)); }

    pub fn setString(&self, key: &str, value: Option<JString>) { self.put(key, NBTTagString::create(value)); }

    pub fn setByteArray(&self, key: &str, value: Rc<Vec<i8>>) { self.put(key, NBTTagByteArray::create(value)); }

    pub fn setCompoundTag(&self, key: &str, nbt: &Rc<NBTBase>) { self.put(key, Rc::clone(nbt)); }

    pub fn setBoolean(&self, key: &str, value: bool) { self.setByte(key, if value { 1 } else { 0 }); }

    pub fn hasKey(&self, key: &str) -> bool { self.m_tagMap.with(|map| map.containsKey(&JString::from(key))) }

    pub fn getByte(&self, key: &str) -> i8 { self.lookup(key).map_or(0, |tag| tag.byteTag().m_byteValue.get()) }

    pub fn getShort(&self, key: &str) -> i16 { self.lookup(key).map_or(0, |tag| tag.shortTag().m_shortValue.get()) }

    pub fn getInteger(&self, key: &str) -> i32 { self.lookup(key).map_or(0, |tag| tag.intTag().m_intValue.get()) }

    pub fn getLong(&self, key: &str) -> i64 { self.lookup(key).map_or(0, |tag| tag.longTag().m_longValue.get()) }

    pub fn getFloat(&self, key: &str) -> f32 { self.lookup(key).map_or(0.0, |tag| tag.floatTag().m_floatValue.get()) }

    pub fn getDouble(&self, key: &str) -> f64
    {
        self.lookup(key).map_or(0.0, |tag| tag.doubleTag().m_doubleValue.get())
    }

    pub fn getString(&self, key: &str) -> Option<JString>
    {
        match self.lookup(key)
        {
            None => Some(JString::new()),
            Some(tag) => tag.stringTag().m_stringValue.get(),
        }
    }

    pub fn getByteArray(&self, key: &str) -> Option<Rc<Vec<i8>>>
    {
        match self.lookup(key)
        {
            None => Some(Rc::new(Vec::new())),
            Some(tag) => tag.byteArrayTag().m_byteArray.get(),
        }
    }

    pub fn getCompoundTag(&self, key: &str) -> Rc<NBTBase>
    {
        match self.lookup(key)
        {
            None => NBTTagCompound::create(),
            Some(tag) =>
            {
                tag.compound();
                tag
            }
        }
    }

    pub fn getTagList(&self, key: &str) -> Rc<NBTBase>
    {
        match self.lookup(key)
        {
            None => NBTTagList::create(),
            Some(tag) =>
            {
                tag.list();
                tag
            }
        }
    }

    pub fn getBoolean(&self, key: &str) -> bool { self.getByte(key) != 0 }

    pub fn toString(&self) -> String { format!("{} entries", self.m_tagMap.with(JHashMap::size)) }
}
