use crate::nbt::{NBTBase, NBTTag};
use jvm::io::{DataInputStream, DataOutputStream, InputStream, IoResult, OutputStream};
use std::cell::Cell;
use std::rc::Rc;

#[derive(Default)]
pub struct NBTTagShort
{
    pub m_shortValue: Cell<i16>,
}

impl NBTTagShort
{
    pub fn create(value: i16) -> Rc<NBTBase> { NBTBase::wrap(NBTTag::Short(Self { m_shortValue: Cell::new(value) })) }

    pub fn writeTagContents<S: OutputStream>(&self, output: &mut DataOutputStream<S>) -> IoResult<()>
    {
        output.writeShort(self.m_shortValue.get() as i32)
    }

    pub fn readTagContents<S: InputStream>(&self, input: &mut DataInputStream<S>) -> IoResult<()>
    {
        self.m_shortValue.set(input.readShort()?);
        Ok(())
    }

    pub fn toString(&self) -> String { self.m_shortValue.get().to_string() }
}
