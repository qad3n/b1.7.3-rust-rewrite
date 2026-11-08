use crate::nbt::{NBTBase, NBTTag};
use jvm::io::{DataInputStream, DataOutputStream, InputStream, IoResult, OutputStream};
use std::rc::Rc;

pub struct NBTTagEnd;

impl NBTTagEnd
{
    pub fn create() -> Rc<NBTBase> { NBTBase::wrap(NBTTag::End(Self)) }

    pub fn readTagContents<S: InputStream>(&self, _input: &mut DataInputStream<S>) -> IoResult<()> { Ok(()) }

    pub fn writeTagContents<S: OutputStream>(&self, _output: &mut DataOutputStream<S>) -> IoResult<()> { Ok(()) }

    pub fn toString(&self) -> String { "END".to_owned() }
}
