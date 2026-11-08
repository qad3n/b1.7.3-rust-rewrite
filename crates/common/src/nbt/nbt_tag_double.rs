use crate::nbt::{NBTBase, NBTTag};
use jvm::io::{DataInputStream, DataOutputStream, InputStream, IoResult, OutputStream};
use std::cell::Cell;
use std::rc::Rc;

#[derive(Default)]
pub struct NBTTagDouble
{
    pub m_doubleValue: Cell<f64>,
}

impl NBTTagDouble
{
    pub fn create(value: f64) -> Rc<NBTBase> { NBTBase::wrap(NBTTag::Double(Self { m_doubleValue: Cell::new(value) })) }

    pub fn writeTagContents<S: OutputStream>(&self, output: &mut DataOutputStream<S>) -> IoResult<()>
    {
        output.writeDouble(self.m_doubleValue.get())
    }

    pub fn readTagContents<S: InputStream>(&self, input: &mut DataInputStream<S>) -> IoResult<()>
    {
        self.m_doubleValue.set(input.readDouble()?);
        Ok(())
    }

    pub fn toString(&self) -> String { jvm::floating_decimal::doubleToString(self.m_doubleValue.get()) }
}
