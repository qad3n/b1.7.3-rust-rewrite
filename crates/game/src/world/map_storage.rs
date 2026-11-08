use crate::world::{ISaveHandler, MapDataBase, MapDataKind};
use common::nbt::{CompressedStreamTools, NBTBase, NBTTag, NBTTagCompound};
use jvm::array_list::JArrayList;
use jvm::hash_map::JHashMap;
use jvm::io::{DataInputStream, DataOutputStream, FileInputStream, FileOutputStream, IoResult};
use jvm::string::JString;
use jvm::throwable::{self, Throwable};
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;

pub struct MapStorage
{
    m_storage: Option<Rc<dyn ISaveHandler>>,
    m_loadedDataMap: RefCell<HashMap<String, Rc<MapDataBase>>>,
    m_loadedDataList: JArrayList<Rc<MapDataBase>>,
    m_idCounts: RefCell<JHashMap<JString, i16>>,
}

fn printExceptions(body: impl FnOnce() -> IoResult<()>)
{
    let result: Result<IoResult<()>, Throwable> = throwable::catchThrowable(&throwable::EXCEPTION, body);
    match result
    {
        Ok(Ok(())) =>
        {}
        Ok(Err(error)) | Err(error) => error.printStackTrace(),
    }
}

impl MapStorage
{
    pub fn new(storage: Rc<dyn ISaveHandler>) -> MapStorage
    {
        let mapStorage: MapStorage = MapStorage {
            m_storage: Some(storage),
            m_loadedDataMap: RefCell::new(HashMap::new()),
            m_loadedDataList: JArrayList::new(),
            m_idCounts: RefCell::new(JHashMap::new()),
        };
        mapStorage.loadIdCounts();
        mapStorage
    }

    pub fn loadData(&self, kind: MapDataKind, id: &str) -> Option<Rc<MapDataBase>>
    {
        if let Some(mapdatabase) = self.m_loadedDataMap.borrow().get(id)
        {
            return Some(Rc::clone(mapdatabase));
        }
        let mut mapdatabase: Option<Rc<MapDataBase>> = None;
        if let Some(storage) = &self.m_storage
        {
            printExceptions(|| {
                let file1: Option<PathBuf> = storage.getDataFile(id);
                if let Some(file1) = file1
                    && file1.exists()
                {
                    let data: Rc<MapDataBase> = MapDataBase::newInstance(kind, id);
                    mapdatabase = Some(Rc::clone(&data));
                    let fileinputstream: FileInputStream = FileInputStream::open(&file1)?;
                    let nbttagcompound: Rc<NBTBase> = CompressedStreamTools::readCompressed(fileinputstream)?;
                    data.readFromNBT(nbttagcompound.compound().getCompoundTag("data").compound());
                }
                Ok(())
            });
        }
        if let Some(data) = &mapdatabase
        {
            self.m_loadedDataMap.borrow_mut().insert(id.to_string(), Rc::clone(data));
            self.m_loadedDataList.add(Rc::clone(data));
        }
        mapdatabase
    }

    pub fn setData(&self, id: &str, data: Rc<MapDataBase>)
    {
        let previous: Option<Rc<MapDataBase>> = self.m_loadedDataMap.borrow_mut().remove(id);
        if let Some(previous) = previous
        {
            self.m_loadedDataList.removeIdentity(&previous);
        }
        self.m_loadedDataMap.borrow_mut().insert(id.to_string(), Rc::clone(&data));
        self.m_loadedDataList.add(data);
    }

    pub fn saveAllData(&self)
    {
        let mut i: i32 = 0;
        while i < self.m_loadedDataList.size()
        {
            let mapdatabase: Rc<MapDataBase> = self.m_loadedDataList.get(i);
            if mapdatabase.isDirty()
            {
                self.saveData(&mapdatabase);
                mapdatabase.setDirty(false);
            }
            i += 1;
        }
    }

    fn saveData(&self, data: &MapDataBase)
    {
        let Some(storage) = &self.m_storage
        else
        {
            return;
        };
        printExceptions(|| {
            if let Some(dataFile) = storage.getDataFile(&data.m_id)
            {
                let nbtTagCompound: Rc<NBTBase> = NBTTagCompound::create();
                data.writeToNBT(nbtTagCompound.compound());
                let nbt: Rc<NBTBase> = NBTTagCompound::create();
                nbt.compound().setCompoundTag("data", &nbtTagCompound);
                let os: FileOutputStream = FileOutputStream::create(&dataFile, false)?;
                CompressedStreamTools::writeGzippedCompoundToOutputStream(&nbt, os)?;
            }
            Ok(())
        });
    }

    fn loadIdCounts(&self)
    {
        printExceptions(|| {
            self.m_idCounts.borrow_mut().clear();
            let Some(storage) = &self.m_storage
            else
            {
                return Ok(());
            };
            if let Some(file1) = storage.getDataFile("idcounts")
                && file1.exists()
            {
                let mut datainputstream: DataInputStream<FileInputStream> =
                    DataInputStream::new(FileInputStream::open(&file1)?);
                let nbttagcompound: Rc<NBTBase> = CompressedStreamTools::read(&mut datainputstream)?;
                for nbtbase in nbttagcompound.compound().getValues()
                {
                    if let NBTTag::Short(nbttagshort) = &nbtbase.m_tag
                    {
                        let s: JString = nbtbase.getKey();
                        let short1: i16 = nbttagshort.m_shortValue.get();
                        self.m_idCounts.borrow_mut().put(s, short1);
                    }
                }
            }
            Ok(())
        });
    }

    pub fn getUniqueDataId(&self, id: &str) -> i32
    {
        let key: JString = JString::from(id);
        let previous: Option<i16> = self.m_idCounts.borrow().get(&key);
        let oshort: i16 = match previous
        {
            None => 0,
            Some(count) => count.wrapping_add(1),
        };
        self.m_idCounts.borrow_mut().put(key, oshort);
        let Some(storage) = &self.m_storage
        else
        {
            return oshort as i32;
        };
        printExceptions(|| {
            if let Some(file1) = storage.getDataFile("idcounts")
            {
                let nbttagcompound: Rc<NBTBase> = NBTTagCompound::create();
                let counts: Vec<(JString, i16)> = self.m_idCounts.borrow().entries();
                for (s, short1) in counts
                {
                    nbttagcompound.compound().setShort(&s.to_string(), short1);
                }
                let mut dataoutputstream: DataOutputStream<FileOutputStream> =
                    DataOutputStream::new(FileOutputStream::create(&file1, false)?);
                CompressedStreamTools::write(&nbttagcompound, &mut dataoutputstream)?;
                dataoutputstream.close()?;
            }
            Ok(())
        });
        oshort as i32
    }
}
