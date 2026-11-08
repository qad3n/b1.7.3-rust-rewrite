use crate::nbt::{NBTBase, NBTTag};
use jvm::field::Field;
use jvm::io::{DataInputStream, DataOutputStream, InputStream, IoResult, OutputStream};
use jvm::string::JString;
use jvm::throwable;
use std::rc::Rc;

pub struct NBTTagString
{
    pub m_stringValue: Field<Option<JString>>,
}

impl NBTTagString
{
    pub fn createEmpty() -> Rc<NBTBase> { NBTBase::wrap(NBTTag::String(Self { m_stringValue: Field::new(None) })) }

    pub fn create(value: Option<JString>) -> Rc<NBTBase>
    {
        let present: bool = value.is_some();
        let tag: Rc<NBTBase> = NBTBase::wrap(NBTTag::String(Self { m_stringValue: Field::new(value) }));
        if !present
        {
            throwable::throw(&throwable::ILLEGAL_ARGUMENT, "Empty string not allowed");
        }
        tag
    }

    pub fn writeTagContents<S: OutputStream>(&self, output: &mut DataOutputStream<S>) -> IoResult<()>
    {
        match self.m_stringValue.get()
        {
            Some(value) => output.writeUTF(&value),
            None => throwable::npe(),
        }
    }

    pub fn readTagContents<S: InputStream>(&self, input: &mut DataInputStream<S>) -> IoResult<()>
    {
        self.m_stringValue.set(Some(input.readUTF()?));
        Ok(())
    }

    pub fn toString(&self) -> String
    {
        match self.m_stringValue.get()
        {
            Some(value) => value.to_string(),
            None => "null".to_owned(),
        }
    }
}
