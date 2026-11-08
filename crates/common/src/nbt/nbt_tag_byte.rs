use crate::nbt::{NBTBase, NBTTag};
use jvm::io::{DataInputStream, DataOutputStream, InputStream, IoResult, OutputStream};
use std::cell::Cell;
use std::rc::Rc;

#[derive(Default)]
pub struct NBTTagByte
{
    pub m_byteValue: Cell<i8>,
}

impl NBTTagByte
{
    pub fn create(value: i8) -> Rc<NBTBase> { NBTBase::wrap(NBTTag::Byte(Self { m_byteValue: Cell::new(value) })) }

    pub fn writeTagContents<S: OutputStream>(&self, output: &mut DataOutputStream<S>) -> IoResult<()>
    {
        output.writeByte(self.m_byteValue.get() as i32)
    }

    pub fn readTagContents<S: InputStream>(&self, input: &mut DataInputStream<S>) -> IoResult<()>
    {
        self.m_byteValue.set(input.readByte()?);
        Ok(())
    }

    pub fn toString(&self) -> String { self.m_byteValue.get().to_string() }
}
