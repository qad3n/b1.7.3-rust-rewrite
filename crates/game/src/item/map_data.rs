use crate::entity::{Entity, entity_player};
use crate::item::ItemStack;
use common::nbt::NBTTagCompound;
use jvm::array_list::JArrayList;
use std::cell::{Cell, RefCell, RefMut};
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Clone, Copy, Debug)]
pub struct MapCoord
{
    pub m_type: i8,
    pub m_x: i8,
    pub m_z: i8,
    pub m_rotation: i8,
}

impl MapCoord
{
    pub fn new(kind: i8, x: i8, z: i8, rotation: i8) -> MapCoord
    {
        MapCoord { m_type: kind, m_x: x, m_z: z, m_rotation: rotation }
    }
}

pub struct MapInfo
{
    pub m_entityplayerObj: Rc<Entity>,
    pub m_dirtyMinY: RefCell<[i32; 128]>,
    pub m_dirtyMaxY: RefCell<[i32; 128]>,
    pub m_nextDirtyPixel: Cell<i32>,
    pub m_cooldown: Cell<i32>,
    m_decorations: RefCell<Option<Vec<i8>>>,
}

impl MapInfo
{
    pub fn new(player: &Rc<Entity>) -> Rc<MapInfo>
    {
        Rc::new(MapInfo {
            m_entityplayerObj: Rc::clone(player),
            m_dirtyMinY: RefCell::new([0; 128]),
            m_dirtyMaxY: RefCell::new([127; 128]),
            m_nextDirtyPixel: Cell::new(0),
            m_cooldown: Cell::new(0),
            m_decorations: RefCell::new(None),
        })
    }

    pub fn getUpdateData(&self, _item: &Rc<ItemStack>, outer: &MapData) -> Option<Vec<i8>>
    {
        let mut n: i32;
        self.m_cooldown.set(self.m_cooldown.get() - 1);
        if self.m_cooldown.get() < 0
        {
            self.m_cooldown.set(4);
            let decorations: Vec<MapCoord> = outer.m_decorations.borrow().clone();
            let mut byArray: Vec<i8> = vec![0; decorations.len() * 3 + 1];
            byArray[0] = 1;
            n = 0;
            while (n as usize) < decorations.len()
            {
                let mapCoord: MapCoord = decorations[n as usize];
                byArray[(n * 3 + 1) as usize] =
                    (mapCoord.m_type as i32 + (mapCoord.m_rotation as i32 & 0xF) * 16) as i8;
                byArray[(n * 3 + 2) as usize] = mapCoord.m_x;
                byArray[(n * 3 + 3) as usize] = mapCoord.m_z;
                n += 1;
            }
            n = 1;
            match &*self.m_decorations.borrow()
            {
                Some(previous) if previous.len() == byArray.len() =>
                {
                    for i in 0..byArray.len()
                    {
                        if byArray[i] == previous[i]
                        {
                            continue;
                        }
                        n = 0;
                        break;
                    }
                }
                _ => n = 0,
            }
            if n == 0
            {
                *self.m_decorations.borrow_mut() = Some(byArray.clone());
                return Some(byArray);
            }
        }
        for _ in 0..10
        {
            n = self.m_nextDirtyPixel.get().wrapping_mul(11) % 128;
            self.m_nextDirtyPixel.set(self.m_nextDirtyPixel.get() + 1);
            let column: usize = n as usize;
            if self.m_dirtyMinY.borrow()[column] < 0
            {
                continue;
            }
            let n2: i32 = self.m_dirtyMaxY.borrow()[column] - self.m_dirtyMinY.borrow()[column] + 1;
            let n3: i32 = self.m_dirtyMinY.borrow()[column];
            let mut byArray: Vec<i8> = vec![0; (n2 + 3) as usize];
            byArray[0] = 0;
            byArray[1] = n as i8;
            byArray[2] = n3 as i8;
            let colors: std::cell::Ref<'_, Vec<i8>> = outer.m_colors.borrow();
            for j in 0..byArray.len() - 3
            {
                byArray[j + 3] = colors[((j as i32 + n3) * 128 + n) as usize];
            }
            self.m_dirtyMaxY.borrow_mut()[column] = -1;
            self.m_dirtyMinY.borrow_mut()[column] = -1;
            return Some(byArray);
        }
        None
    }
}

pub struct MapData
{
    pub m_centerX: Cell<i32>,
    pub m_centerZ: Cell<i32>,
    pub m_dimension: Cell<i8>,
    pub m_scale: Cell<i8>,
    pub m_colors: RefCell<Vec<i8>>,
    pub m_ticks: Cell<i32>,
    pub m_holders: JArrayList<Rc<MapInfo>>,
    m_holdersByPlayer: RefCell<HashMap<i32, Rc<MapInfo>>>,
    pub m_decorations: RefCell<Vec<MapCoord>>,
}

impl MapData
{
    pub fn new() -> MapData
    {
        MapData {
            m_centerX: Cell::new(0),
            m_centerZ: Cell::new(0),
            m_dimension: Cell::new(0),
            m_scale: Cell::new(0),
            m_colors: RefCell::new(vec![0; 16384]),
            m_ticks: Cell::new(0),
            m_holders: JArrayList::new(),
            m_holdersByPlayer: RefCell::new(HashMap::new()),
            m_decorations: RefCell::new(Vec::new()),
        }
    }

    pub fn readFromNBT(&self, nbt: &NBTTagCompound)
    {
        self.m_dimension.set(nbt.getByte("dimension"));
        self.m_centerX.set(nbt.getInteger("xCenter"));
        self.m_centerZ.set(nbt.getInteger("zCenter"));
        self.m_scale.set(nbt.getByte("scale"));
        if self.m_scale.get() < 0
        {
            self.m_scale.set(0);
        }
        if self.m_scale.get() > 4
        {
            self.m_scale.set(4);
        }
        let n: i32 = nbt.getShort("width") as i32;
        let n2: i32 = nbt.getShort("height") as i32;
        let colors: Vec<i8> = nbt.getByteArray("colors").map(|array| (*array).clone()).unwrap_or_default();
        if n == 128 && n2 == 128
        {
            *self.m_colors.borrow_mut() = colors;
        }
        else
        {
            let byArray: Vec<i8> = colors;
            let mut target: Vec<i8> = vec![0; 16384];
            let n3: i32 = (128 - n) / 2;
            let n4: i32 = (128 - n2) / 2;
            for i in 0..n2
            {
                let n5: i32 = i + n4;
                for j in 0..n
                {
                    let n6: i32 = j + n3;
                    target[(n6 + n5 * 128) as usize] = byArray[(j + i * n) as usize];
                }
            }
            *self.m_colors.borrow_mut() = target;
        }
    }

    pub fn writeToNBT(&self, nbt: &NBTTagCompound)
    {
        nbt.setByte("dimension", self.m_dimension.get());
        nbt.setInteger("xCenter", self.m_centerX.get());
        nbt.setInteger("zCenter", self.m_centerZ.get());
        nbt.setByte("scale", self.m_scale.get());
        nbt.setShort("width", 128);
        nbt.setShort("height", 128);
        nbt.setByteArray("colors", Rc::new(self.m_colors.borrow().clone()));
    }

    pub fn tickHolder(&self, player: &Rc<Entity>, item: &Rc<ItemStack>)
    {
        let key: i32 = player.m_entityId.get();
        if !self.m_holdersByPlayer.borrow().contains_key(&key)
        {
            let mapInfo: Rc<MapInfo> = MapInfo::new(player);
            self.m_holdersByPlayer.borrow_mut().insert(key, Rc::clone(&mapInfo));
            self.m_holders.add(mapInfo);
        }
        self.m_decorations.borrow_mut().clear();
        let mut i: i32 = 0;
        while i < self.m_holders.size()
        {
            let mapInfo: Rc<MapInfo> = self.m_holders.get(i);
            i += 1;
            let holder: &Rc<Entity> = &mapInfo.m_entityplayerObj;
            if holder.m_isDead.get() || !entity_player::inventory(holder).contains(item)
            {
                self.m_holdersByPlayer.borrow_mut().remove(&holder.m_entityId.get());
                self.m_holders.removeIdentity(&mapInfo);
                continue;
            }
            let scale: f32 = (1 << self.m_scale.get()) as f32;
            let f: f32 = (holder.m_posX.get() - self.m_centerX.get() as f64) as f32 / scale;
            let f2: f32 = (holder.m_posZ.get() - self.m_centerZ.get() as f64) as f32 / scale;
            let n: i32 = 64;
            let n2: i32 = 64;
            if !(f >= -n as f32) || !(f2 >= -n2 as f32) || !(f <= n as f32) || !(f2 <= n2 as f32)
            {
                continue;
            }
            let by: i8 = 0;
            let by2: i8 = ((f * 2.0) as f64 + 0.5) as i32 as i8;
            let by3: i8 = ((f2 * 2.0) as f64 + 0.5) as i32 as i8;
            let mut by4: i8 = ((player.m_rotationYaw.get() * 16.0 / 360.0) as f64 + 0.5) as i32 as i8;
            if self.m_dimension.get() < 0
            {
                let n3: i32 = self.m_ticks.get() / 10;
                by4 =
                    ((n3.wrapping_mul(n3).wrapping_mul(34187121).wrapping_add(n3.wrapping_mul(121)) >> 15) & 0xF) as i8;
            }
            if entity_player::fields(holder).m_dimension.get() != self.m_dimension.get() as i32
            {
                continue;
            }
            self.m_decorations.borrow_mut().push(MapCoord::new(by, by2, by3, by4));
        }
    }

    pub fn getUpdateData(
        &self,
        item: &Rc<ItemStack>,
        _world: &Rc<crate::world::World>,
        player: &Rc<Entity>,
    ) -> Option<Vec<i8>>
    {
        let mapInfo: Option<Rc<MapInfo>> = self.m_holdersByPlayer.borrow().get(&player.m_entityId.get()).cloned();
        let mapInfo: Rc<MapInfo> = mapInfo?;
        mapInfo.getUpdateData(item, self)
    }

    pub fn markDirtyColumn(&self, base: &crate::world::MapDataBase, index: i32, minY: i32, maxY: i32)
    {
        base.markDirty();
        for i in 0..self.m_holders.size()
        {
            let mapInfo: Rc<MapInfo> = self.m_holders.get(i);
            let column: usize = index as usize;
            let mut dirtyMinY: RefMut<[i32; 128]> = mapInfo.m_dirtyMinY.borrow_mut();
            if dirtyMinY[column] < 0 || dirtyMinY[column] > minY
            {
                dirtyMinY[column] = minY;
            }
            let mut dirtyMaxY: RefMut<[i32; 128]> = mapInfo.m_dirtyMaxY.borrow_mut();
            if dirtyMaxY[column] < 0 || dirtyMaxY[column] < maxY
            {
                dirtyMaxY[column] = maxY;
            }
        }
    }

    pub fn update(&self, base: &crate::world::MapDataBase, data: &[i8])
    {
        if data[0] == 0
        {
            let n: i32 = data[1] as i32 & 0xFF;
            let n2: i32 = data[2] as i32 & 0xFF;
            for i in 0..data.len() as i32 - 3
            {
                self.m_colors.borrow_mut()[((i + n2) * 128 + n) as usize] = data[(i + 3) as usize];
            }
            base.markDirty();
        }
        else if data[0] == 1
        {
            self.m_decorations.borrow_mut().clear();
            for i in 0..(data.len() as i32 - 1) / 3
            {
                let by: i8 = data[(i * 3 + 1) as usize] % 16;
                let by2: i8 = data[(i * 3 + 2) as usize];
                let by3: i8 = data[(i * 3 + 3) as usize];
                let by4: i8 = data[(i * 3 + 1) as usize] / 16;
                self.m_decorations.borrow_mut().push(MapCoord::new(by, by2, by3, by4));
            }
        }
    }
}
