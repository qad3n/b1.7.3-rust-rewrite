use crate::nbt::{NBTBase, NBTTag};
use jvm::io::{DataInputStream, DataOutputStream, InputStream, IoResult, OutputStream};
use std::cell::Cell;
use std::rc::Rc;

#[derive(Default)]
pub struct NBTTagLong
{
    pub m_longValue: Cell<i64>,
}

impl NBTTagLong
{
    pub fn create(value: i64) -> Rc<NBTBase> { NBTBase::wrap(NBTTag::Long(Self { m_longValue: Cell::new(value) })) }

    pub fn writeTagContents<S: OutputStream>(&self, output: &mut DataOutputStream<S>) -> IoResult<()>
    {
        output.writeLong(self.m_longValue.get())
    }

    pub fn readTagContents<S: InputStream>(&self, input: &mut DataInputStream<S>) -> IoResult<()>
    {
        self.m_longValue.set(input.readLong()?);
        Ok(())
    }

    pub fn toString(&self) -> String { self.m_longValue.get().to_string() }
}
