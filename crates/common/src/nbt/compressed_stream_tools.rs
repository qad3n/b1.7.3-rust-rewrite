use crate::nbt::NBTBase;
use jvm::io::{DataInputStream, DataOutputStream, InputStream, IoResult, OutputStream};
use jvm::throwable::{self, Throwable};
use jvm::zip::{GZIPInputStream, GZIPOutputStream};
use std::rc::Rc;

pub struct CompressedStreamTools;

impl CompressedStreamTools
{
    pub fn readCompressed<S: InputStream>(input: S) -> IoResult<Rc<NBTBase>>
    {
        let mut datainputstream: DataInputStream<GZIPInputStream<S>> =
            DataInputStream::new(GZIPInputStream::new(input)?);
        let result: Result<IoResult<Rc<NBTBase>>, Throwable> =
            throwable::catchThrowable(&throwable::THROWABLE, || Self::read(&mut datainputstream));
        datainputstream.m_in.close()?;
        result.unwrap_or_else(|thrown| throwable::rethrow(thrown))
    }

    pub fn writeGzippedCompoundToOutputStream<S: OutputStream>(nbt: &Rc<NBTBase>, output: S) -> IoResult<()>
    {
        let mut dataoutputstream: DataOutputStream<GZIPOutputStream<S>> =
            DataOutputStream::new(GZIPOutputStream::new(output)?);
        let result: Result<IoResult<()>, Throwable> =
            throwable::catchThrowable(&throwable::THROWABLE, || Self::write(nbt, &mut dataoutputstream));
        dataoutputstream.close()?;
        result.unwrap_or_else(|thrown| throwable::rethrow(thrown))
    }

    pub fn read<S: InputStream>(input: &mut DataInputStream<S>) -> IoResult<Rc<NBTBase>>
    {
        let nbtbase: Rc<NBTBase> = NBTBase::readTag(input)?;
        if nbtbase.getType() == 10
        {
            return Ok(nbtbase);
        }
        Err(jvm::io::ioError(&throwable::IO, "Root tag must be a named compound tag"))
    }

    pub fn write<S: OutputStream>(nbt: &Rc<NBTBase>, output: &mut DataOutputStream<S>) -> IoResult<()>
    {
        NBTBase::writeTag(nbt, output)
    }
}
