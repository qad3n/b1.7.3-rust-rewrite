use crate::nbt::{NBTBase, NBTTag};
use jvm::io::{DataInputStream, DataOutputStream, InputStream, IoResult, OutputStream};
use std::cell::Cell;
use std::rc::Rc;

#[derive(Default)]
pub struct NBTTagInt
{
    pub m_intValue: Cell<i32>,
}

impl NBTTagInt
{
    pub fn create(value: i32) -> Rc<NBTBase> { NBTBase::wrap(NBTTag::Int(Self { m_intValue: Cell::new(value) })) }

    pub fn writeTagContents<S: OutputStream>(&self, output: &mut DataOutputStream<S>) -> IoResult<()>
    {
        output.writeInt(self.m_intValue.get())
    }

    pub fn readTagContents<S: InputStream>(&self, input: &mut DataInputStream<S>) -> IoResult<()>
    {
        self.m_intValue.set(input.readInt()?);
        Ok(())
    }

    pub fn toString(&self) -> String { self.m_intValue.get().to_string() }
}
