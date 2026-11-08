use crate::nbt::{NBTBase, NBTTag};
use jvm::field::Field;
use jvm::io::{DataInputStream, DataOutputStream, InputStream, IoResult, OutputStream};
use jvm::throwable;
use std::cell::Cell;
use std::rc::Rc;

pub struct NBTTagList
{
    m_tagList: Field<Vec<Rc<NBTBase>>>,
    m_tagType: Cell<i8>,
}

impl NBTTagList
{
    pub fn create() -> Rc<NBTBase>
    {
        NBTBase::wrap(NBTTag::List(Self { m_tagList: Field::new(Vec::new()), m_tagType: Cell::new(0) }))
    }

    pub fn writeTagContents<S: OutputStream>(&self, output: &mut DataOutputStream<S>) -> IoResult<()>
    {
        let tags: Vec<Rc<NBTBase>> = self.m_tagList.get();
        if !tags.is_empty()
        {
            self.m_tagType.set(tags[0].getType());
        }
        else
        {
            self.m_tagType.set(1);
        }

        output.writeByte(self.m_tagType.get() as i32)?;
        output.writeInt(tags.len() as i32)?;
        for tag in &tags
        {
            tag.writeTagContents(output)?;
        }
        Ok(())
    }

    pub fn readTagContents<S: InputStream>(&self, input: &mut DataInputStream<S>) -> IoResult<()>
    {
        self.m_tagType.set(input.readByte()?);
        let i: i32 = input.readInt()?;
        self.m_tagList.set(Vec::new());
        for _ in 0..i
        {
            let nbtbase: Rc<NBTBase> = match NBTBase::createTagOfType(self.m_tagType.get())
            {
                Some(tag) => tag,
                None => throwable::npe(),
            };
            nbtbase.readTagContents(input)?;
            self.m_tagList.update(|list| list.push(nbtbase));
        }
        Ok(())
    }

    pub fn toString(&self) -> String
    {
        format!("{} entries of type {}", self.tagCount(), NBTBase::getTagName(self.m_tagType.get()))
    }

    pub fn setTag(&self, element: Rc<NBTBase>)
    {
        self.m_tagType.set(element.getType());
        self.m_tagList.update(|list| list.push(element));
    }

    pub fn tagAt(&self, index: i32) -> Rc<NBTBase>
    {
        let size: i32 = self.tagCount();
        if index >= size
        {
            throwable::throw(&throwable::INDEX_OUT_OF_BOUNDS, &format!("Index: {index}, Size: {size}"));
        }
        if index < 0
        {
            throwable::throw(&throwable::ARRAY_INDEX, &index.to_string());
        }
        self.m_tagList.with(|list| Rc::clone(&list[index as usize]))
    }

    pub fn tagCount(&self) -> i32 { self.m_tagList.with(Vec::len) as i32 }
}
