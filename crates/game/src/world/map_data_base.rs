use crate::item::MapData;
use common::nbt::NBTTagCompound;
use std::cell::Cell;
use std::rc::Rc;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MapDataKind
{
    MapData,
}

pub struct MapDataBase
{
    pub m_id: String,
    m_dirty: Cell<bool>,
    pub m_map: MapData,
}

impl MapDataBase
{
    pub fn newMap(id: &str) -> Rc<MapDataBase>
    {
        Rc::new(MapDataBase { m_id: id.to_string(), m_dirty: Cell::new(false), m_map: MapData::new() })
    }

    pub fn newInstance(kind: MapDataKind, id: &str) -> Rc<MapDataBase>
    {
        match kind
        {
            MapDataKind::MapData => MapDataBase::newMap(id),
        }
    }

    pub fn readFromNBT(&self, nbt: &NBTTagCompound) { self.m_map.readFromNBT(nbt); }

    pub fn writeToNBT(&self, nbt: &NBTTagCompound) { self.m_map.writeToNBT(nbt); }

    pub fn markDirty(&self) { self.setDirty(true); }

    pub fn setDirty(&self, dirty: bool) { self.m_dirty.set(dirty); }

    pub fn isDirty(&self) -> bool { self.m_dirty.get() }
}
