use crate::nbt::{NBTBase, NBTTag};
use jvm::io::{DataInputStream, DataOutputStream, InputStream, IoResult, OutputStream};
use std::cell::Cell;
use std::rc::Rc;

#[derive(Default)]
pub struct NBTTagFloat
{
    pub m_floatValue: Cell<f32>,
}

impl NBTTagFloat
{
    pub fn create(value: f32) -> Rc<NBTBase> { NBTBase::wrap(NBTTag::Float(Self { m_floatValue: Cell::new(value) })) }

    pub fn writeTagContents<S: OutputStream>(&self, output: &mut DataOutputStream<S>) -> IoResult<()>
    {
        output.writeFloat(self.m_floatValue.get())
    }

    pub fn readTagContents<S: InputStream>(&self, input: &mut DataInputStream<S>) -> IoResult<()>
    {
        self.m_floatValue.set(input.readFloat()?);
        Ok(())
    }

    pub fn toString(&self) -> String { jvm::floating_decimal::floatToString(self.m_floatValue.get()) }
}
