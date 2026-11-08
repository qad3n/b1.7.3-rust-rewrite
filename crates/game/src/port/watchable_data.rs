use crate::entity::{WatchableObject, WatchedValue};
use crate::network::packet::{self, PacketInput, PacketOutput};
use crate::port::packet_item_stack::PacketItemStack;
use crate::world::ChunkCoordinates;
use jvm::io::IoResult;
use jvm::string::JString;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub enum WatchedData
{
    Byte(i8),
    Short(i16),
    Int(i32),
    Float(f32),
    String(JString),
    ItemStack(PacketItemStack),
    ChunkCoordinates(i32, i32, i32),
}

#[derive(Clone, Debug)]
pub struct WatchableData
{
    m_objectType: i32,
    m_dataValueId: i32,
    m_watchedObject: WatchedData,
}

impl WatchableData
{
    fn toWatchableObject(&self) -> Rc<WatchableObject>
    {
        let value: WatchedValue = match &self.m_watchedObject
        {
            WatchedData::Byte(value) => WatchedValue::Byte(*value),
            WatchedData::Short(value) => WatchedValue::Short(*value),
            WatchedData::Int(value) => WatchedValue::Int(*value),
            WatchedData::Float(value) => WatchedValue::Float(*value),
            WatchedData::String(value) => WatchedValue::String(value.clone()),
            WatchedData::ItemStack(item) => WatchedValue::ItemStack(item.toItemStack()),
            WatchedData::ChunkCoordinates(x, y, z) => WatchedValue::ChunkCoordinates(ChunkCoordinates::new(*x, *y, *z)),
        };
        Rc::new(WatchableObject::new(self.m_objectType, self.m_dataValueId, value))
    }

    fn fromWatchableObject(object: &WatchableObject) -> WatchableData
    {
        let value: WatchedData = match object.getObject()
        {
            WatchedValue::Byte(value) => WatchedData::Byte(value),
            WatchedValue::Short(value) => WatchedData::Short(value),
            WatchedValue::Int(value) => WatchedData::Int(value),
            WatchedValue::Float(value) => WatchedData::Float(value),
            WatchedValue::String(value) => WatchedData::String(value),
            WatchedValue::ItemStack(item) => WatchedData::ItemStack(PacketItemStack {
                m_itemID: item.getItem().m_shiftedIndex,
                m_stackSize: item.m_stackSize.get(),
                m_itemDamage: item.getItemDamage(),
            }),
            WatchedValue::ChunkCoordinates(coordinates) =>
            {
                WatchedData::ChunkCoordinates(coordinates.m_x, coordinates.m_y, coordinates.m_z)
            }
        };
        WatchableData {
            m_objectType: object.getObjectType(),
            m_dataValueId: object.getDataValueId(),
            m_watchedObject: value,
        }
    }

    pub fn fromWatchableObjects(entries: Option<&[Rc<WatchableObject>]>) -> Option<Vec<WatchableData>>
    {
        entries.map(|entries| entries.iter().map(|entry| WatchableData::fromWatchableObject(entry)).collect())
    }

    pub fn toWatchableObjects(entries: Option<&[WatchableData]>) -> Option<Vec<Rc<WatchableObject>>>
    {
        entries.map(|entries| entries.iter().map(WatchableData::toWatchableObject).collect())
    }

    pub fn readWatchableObjects(input: &mut PacketInput) -> IoResult<Option<Vec<WatchableData>>>
    {
        let mut arraylist: Option<Vec<WatchableData>> = None;
        let mut b0: i8 = input.readByte()?;
        while b0 != 127
        {
            let list: &mut Vec<WatchableData> = arraylist.get_or_insert_with(Vec::new);
            let i: i32 = (b0 as i32 & 224) >> 5;
            let j: i32 = b0 as i32 & 31;
            let value: Option<WatchedData> = match i
            {
                0 => Some(WatchedData::Byte(input.readByte()?)),
                1 => Some(WatchedData::Short(input.readShort()?)),
                2 => Some(WatchedData::Int(input.readInt()?)),
                3 => Some(WatchedData::Float(input.readFloat()?)),
                4 => Some(WatchedData::String(packet::readString(input, 64)?)),
                5 =>
                {
                    let short1: i16 = input.readShort()?;
                    let b1: i8 = input.readByte()?;
                    let short2: i16 = input.readShort()?;
                    Some(WatchedData::ItemStack(PacketItemStack {
                        m_itemID: short1 as i32,
                        m_stackSize: b1 as i32,
                        m_itemDamage: short2 as i32,
                    }))
                }
                6 =>
                {
                    let k: i32 = input.readInt()?;
                    let l: i32 = input.readInt()?;
                    let i1: i32 = input.readInt()?;
                    Some(WatchedData::ChunkCoordinates(k, l, i1))
                }
                _ => None,
            };
            let Some(value) = value
            else
            {
                jvm::throwable::npe();
            };
            list.push(WatchableData { m_objectType: i, m_dataValueId: j, m_watchedObject: value });
            b0 = input.readByte()?;
        }
        Ok(arraylist)
    }

    pub fn writeObjectsInListToStream(entries: Option<&[WatchableData]>, output: &mut PacketOutput) -> IoResult<()>
    {
        if let Some(entries) = entries
        {
            for entry in entries
            {
                let i: i32 = (entry.m_objectType << 5 | entry.m_dataValueId & 31) & 0xFF;
                output.writeByte(i)?;
                match &entry.m_watchedObject
                {
                    WatchedData::Byte(value) => output.writeByte(*value as i32)?,
                    WatchedData::Short(value) => output.writeShort(*value as i32)?,
                    WatchedData::Int(value) => output.writeInt(*value)?,
                    WatchedData::Float(value) => output.writeFloat(*value)?,
                    WatchedData::String(value) => packet::writeString(value, output)?,
                    WatchedData::ItemStack(item) => PacketItemStack::write(Some(item), output)?,
                    WatchedData::ChunkCoordinates(x, y, z) =>
                    {
                        output.writeInt(*x)?;
                        output.writeInt(*y)?;
                        output.writeInt(*z)?;
                    }
                }
            }
        }
        output.writeByte(127)
    }
}
