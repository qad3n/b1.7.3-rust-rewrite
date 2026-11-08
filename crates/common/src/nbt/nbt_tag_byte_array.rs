use crate::nbt::{NBTBase, NBTTag};
use jvm::field::Field;
use jvm::io::{DataInputStream, DataOutputStream, InputStream, IoResult, OutputStream};
use jvm::throwable;
use std::rc::Rc;

pub struct NBTTagByteArray
{
    pub m_byteArray: Field<Option<Rc<Vec<i8>>>>,
}

impl NBTTagByteArray
{
    pub fn createEmpty() -> Rc<NBTBase> { NBTBase::wrap(NBTTag::ByteArray(Self { m_byteArray: Field::new(None) })) }

    pub fn create(value: Rc<Vec<i8>>) -> Rc<NBTBase>
    {
        NBTBase::wrap(NBTTag::ByteArray(Self { m_byteArray: Field::new(Some(value)) }))
    }

    fn bytes(&self) -> Rc<Vec<i8>>
    {
        match self.m_byteArray.get()
        {
            Some(bytes) => bytes,
            None => throwable::npe(),
        }
    }

    pub fn writeTagContents<S: OutputStream>(&self, output: &mut DataOutputStream<S>) -> IoResult<()>
    {
        let bytes: Rc<Vec<i8>> = self.bytes();
        output.writeInt(bytes.len() as i32)?;
        let raw: Vec<u8> = bytes.iter().map(|&value| value as u8).collect();
        output.writeBuf(&raw)
    }

    pub fn readTagContents<S: InputStream>(&self, input: &mut DataInputStream<S>) -> IoResult<()>
    {
        let n: i32 = input.readInt()?;
        if n < 0
        {
            throwable::throwBare(&throwable::NEGATIVE_ARRAY_SIZE);
        }
        let mut raw: Vec<u8> = vec![0; n as usize];
        input.readFully(&mut raw)?;
        self.m_byteArray.set(Some(Rc::new(raw.into_iter().map(|value| value as i8).collect())));
        Ok(())
    }

    pub fn toString(&self) -> String { format!("[{} bytes]", self.bytes().len()) }
}
