use crate::nbt::{NBTBase, NBTTagCompound};
use std::collections::BTreeMap;
use std::rc::Rc;

// per mod data on vanilla objects, one byte array per owning mod id or component identifier
pub const ATTACHMENTS: &str = "loader:attachments";
pub const COMPONENTS: &str = "loader:components";

pub type Attachments = BTreeMap<String, Vec<u8>>;

pub fn read(nbt: &NBTTagCompound) -> Attachments { readAt(nbt, ATTACHMENTS) }

pub fn write(attachments: &Attachments, nbt: &NBTTagCompound) { writeAt(attachments, nbt, ATTACHMENTS) }

pub fn readAt(nbt: &NBTTagCompound, key: &str) -> Attachments
{
    if !nbt.hasKey(key)
    {
        return Attachments::new();
    }
    nbt.getCompoundTag(key)
        .compound()
        .getValues()
        .iter()
        .map(|tag| {
            let bytes: Vec<u8> = tag
                .byteArrayTag()
                .m_byteArray
                .get()
                .map(|bytes| bytes.iter().map(|byte| *byte as u8).collect())
                .unwrap_or_default();
            (tag.getKey().to_string(), bytes)
        })
        .collect()
}

pub fn writeAt(attachments: &Attachments, nbt: &NBTTagCompound, key: &str)
{
    if attachments.is_empty()
    {
        return;
    }
    let compound: Rc<NBTBase> = NBTTagCompound::create();
    for (owner, bytes) in attachments
    {
        compound.compound().setByteArray(owner, Rc::new(bytes.iter().map(|byte| *byte as i8).collect()));
    }
    nbt.setCompoundTag(key, &compound);
}
