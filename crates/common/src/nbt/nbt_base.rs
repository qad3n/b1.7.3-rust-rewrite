use crate::nbt::*;
use jvm::field::Field;
use jvm::io::{DataInputStream, DataOutputStream, InputStream, IoResult, OutputStream};
use jvm::string::JString;
use jvm::throwable::{self, CLASS_CAST};
use std::rc::Rc;

pub enum NBTTag
{
    End(NBTTagEnd),
    Byte(NBTTagByte),
    Short(NBTTagShort),
    Int(NBTTagInt),
    Long(NBTTagLong),
    Float(NBTTagFloat),
    Double(NBTTagDouble),
    ByteArray(NBTTagByteArray),
    String(NBTTagString),
    List(NBTTagList),
    Compound(NBTTagCompound),
}

pub struct NBTBase
{
    m_key: Field<Option<JString>>,
    pub m_tag: NBTTag,
}

impl NBTBase
{
    pub fn wrap(tag: NBTTag) -> Rc<Self> { Rc::new(Self { m_key: Field::new(None), m_tag: tag }) }

    pub fn writeTagContents<S: OutputStream>(&self, output: &mut DataOutputStream<S>) -> IoResult<()>
    {
        match &self.m_tag
        {
            NBTTag::End(tag) => tag.writeTagContents(output),
            NBTTag::Byte(tag) => tag.writeTagContents(output),
            NBTTag::Short(tag) => tag.writeTagContents(output),
            NBTTag::Int(tag) => tag.writeTagContents(output),
            NBTTag::Long(tag) => tag.writeTagContents(output),
            NBTTag::Float(tag) => tag.writeTagContents(output),
            NBTTag::Double(tag) => tag.writeTagContents(output),
            NBTTag::ByteArray(tag) => tag.writeTagContents(output),
            NBTTag::String(tag) => tag.writeTagContents(output),
            NBTTag::List(tag) => tag.writeTagContents(output),
            NBTTag::Compound(tag) => tag.writeTagContents(output),
        }
    }

    pub fn readTagContents<S: InputStream>(&self, input: &mut DataInputStream<S>) -> IoResult<()>
    {
        match &self.m_tag
        {
            NBTTag::End(tag) => tag.readTagContents(input),
            NBTTag::Byte(tag) => tag.readTagContents(input),
            NBTTag::Short(tag) => tag.readTagContents(input),
            NBTTag::Int(tag) => tag.readTagContents(input),
            NBTTag::Long(tag) => tag.readTagContents(input),
            NBTTag::Float(tag) => tag.readTagContents(input),
            NBTTag::Double(tag) => tag.readTagContents(input),
            NBTTag::ByteArray(tag) => tag.readTagContents(input),
            NBTTag::String(tag) => tag.readTagContents(input),
            NBTTag::List(tag) => tag.readTagContents(input),
            NBTTag::Compound(tag) => tag.readTagContents(input),
        }
    }

    pub fn getType(&self) -> i8
    {
        match &self.m_tag
        {
            NBTTag::End(_) => 0,
            NBTTag::Byte(_) => 1,
            NBTTag::Short(_) => 2,
            NBTTag::Int(_) => 3,
            NBTTag::Long(_) => 4,
            NBTTag::Float(_) => 5,
            NBTTag::Double(_) => 6,
            NBTTag::ByteArray(_) => 7,
            NBTTag::String(_) => 8,
            NBTTag::List(_) => 9,
            NBTTag::Compound(_) => 10,
        }
    }

    pub fn toString(&self) -> String
    {
        match &self.m_tag
        {
            NBTTag::End(tag) => tag.toString(),
            NBTTag::Byte(tag) => tag.toString(),
            NBTTag::Short(tag) => tag.toString(),
            NBTTag::Int(tag) => tag.toString(),
            NBTTag::Long(tag) => tag.toString(),
            NBTTag::Float(tag) => tag.toString(),
            NBTTag::Double(tag) => tag.toString(),
            NBTTag::ByteArray(tag) => tag.toString(),
            NBTTag::String(tag) => tag.toString(),
            NBTTag::List(tag) => tag.toString(),
            NBTTag::Compound(tag) => tag.toString(),
        }
    }

    pub fn getKey(&self) -> JString { self.m_key.get().unwrap_or_default() }

    pub fn setKey(self: &Rc<Self>, name: Option<JString>) -> Rc<Self>
    {
        self.m_key.set(name);
        Rc::clone(self)
    }

    pub fn readTag<S: InputStream>(input: &mut DataInputStream<S>) -> IoResult<Rc<Self>>
    {
        let by: i8 = input.readByte()?;
        if by == 0
        {
            return Ok(NBTTagEnd::create());
        }

        let nbtBase: Option<Rc<Self>> = Self::createTagOfType(by);
        let key: JString = input.readUTF()?;
        let nbtBase: Rc<Self> = nbtBase.unwrap_or_else(|| throwable::npe());
        nbtBase.m_key.set(Some(key));
        nbtBase.readTagContents(input)?;
        Ok(nbtBase)
    }

    pub fn writeTag<S: OutputStream>(nbt: &Rc<Self>, output: &mut DataOutputStream<S>) -> IoResult<()>
    {
        output.writeByte(nbt.getType() as i32)?;
        if nbt.getType() == 0
        {
            return Ok(());
        }
        output.writeUTF(&nbt.getKey())?;
        nbt.writeTagContents(output)
    }

    pub fn createTagOfType(tagType: i8) -> Option<Rc<Self>>
    {
        match tagType
        {
            0 => Some(NBTTagEnd::create()),
            1 => Some(NBTTagByte::create(0)),
            2 => Some(NBTTagShort::create(0)),
            3 => Some(NBTTagInt::create(0)),
            4 => Some(NBTTagLong::create(0)),
            5 => Some(NBTTagFloat::create(0.0)),
            6 => Some(NBTTagDouble::create(0.0)),
            7 => Some(NBTTagByteArray::createEmpty()),
            8 => Some(NBTTagString::createEmpty()),
            9 => Some(NBTTagList::create()),
            10 => Some(NBTTagCompound::create()),
            _ => None,
        }
    }

    pub fn getTagName(tagType: i8) -> &'static str
    {
        match tagType
        {
            0 => "TAG_End",
            1 => "TAG_Byte",
            2 => "TAG_Short",
            3 => "TAG_Int",
            4 => "TAG_Long",
            5 => "TAG_Float",
            6 => "TAG_Double",
            7 => "TAG_Byte_Array",
            8 => "TAG_String",
            9 => "TAG_List",
            10 => "TAG_Compound",
            _ => "UNKNOWN",
        }
    }

    fn classCast(&self, target: &str) -> !
    {
        throwable::throw(
            &CLASS_CAST,
            &format!("net.minecraft.src.{} cannot be cast to net.minecraft.src.{target}", self.className()),
        )
    }

    fn className(&self) -> &'static str
    {
        match &self.m_tag
        {
            NBTTag::End(_) => "NBTTagEnd",
            NBTTag::Byte(_) => "NBTTagByte",
            NBTTag::Short(_) => "NBTTagShort",
            NBTTag::Int(_) => "NBTTagInt",
            NBTTag::Long(_) => "NBTTagLong",
            NBTTag::Float(_) => "NBTTagFloat",
            NBTTag::Double(_) => "NBTTagDouble",
            NBTTag::ByteArray(_) => "NBTTagByteArray",
            NBTTag::String(_) => "NBTTagString",
            NBTTag::List(_) => "NBTTagList",
            NBTTag::Compound(_) => "NBTTagCompound",
        }
    }

    pub fn compound(&self) -> &NBTTagCompound
    {
        match &self.m_tag
        {
            NBTTag::Compound(tag) => tag,
            _ => self.classCast("NBTTagCompound"),
        }
    }

    pub fn list(&self) -> &NBTTagList
    {
        match &self.m_tag
        {
            NBTTag::List(tag) => tag,
            _ => self.classCast("NBTTagList"),
        }
    }

    pub fn byteTag(&self) -> &NBTTagByte
    {
        match &self.m_tag
        {
            NBTTag::Byte(tag) => tag,
            _ => self.classCast("NBTTagByte"),
        }
    }

    pub fn shortTag(&self) -> &NBTTagShort
    {
        match &self.m_tag
        {
            NBTTag::Short(tag) => tag,
            _ => self.classCast("NBTTagShort"),
        }
    }

    pub fn intTag(&self) -> &NBTTagInt
    {
        match &self.m_tag
        {
            NBTTag::Int(tag) => tag,
            _ => self.classCast("NBTTagInt"),
        }
    }

    pub fn longTag(&self) -> &NBTTagLong
    {
        match &self.m_tag
        {
            NBTTag::Long(tag) => tag,
            _ => self.classCast("NBTTagLong"),
        }
    }

    pub fn floatTag(&self) -> &NBTTagFloat
    {
        match &self.m_tag
        {
            NBTTag::Float(tag) => tag,
            _ => self.classCast("NBTTagFloat"),
        }
    }

    pub fn doubleTag(&self) -> &NBTTagDouble
    {
        match &self.m_tag
        {
            NBTTag::Double(tag) => tag,
            _ => self.classCast("NBTTagDouble"),
        }
    }

    pub fn byteArrayTag(&self) -> &NBTTagByteArray
    {
        match &self.m_tag
        {
            NBTTag::ByteArray(tag) => tag,
            _ => self.classCast("NBTTagByteArray"),
        }
    }

    pub fn stringTag(&self) -> &NBTTagString
    {
        match &self.m_tag
        {
            NBTTag::String(tag) => tag,
            _ => self.classCast("NBTTagString"),
        }
    }
}
