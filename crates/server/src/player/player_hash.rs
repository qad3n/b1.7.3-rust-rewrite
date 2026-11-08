use std::cell::{Cell, RefCell};

pub struct PlayerHashEntry<V>
{
    m_key: i64,
    m_value: RefCell<V>,
    m_hash: i32,
}

impl<V: Clone> PlayerHashEntry<V>
{
    fn new(hash: i32, key: i64, value: V) -> PlayerHashEntry<V>
    {
        PlayerHashEntry { m_value: RefCell::new(value), m_key: key, m_hash: hash }
    }

    pub fn getKey(&self) -> i64 { self.m_key }

    pub fn getValue(&self) -> V { self.m_value.borrow().clone() }

    pub fn hashCode(&self) -> i32 { PlayerHash::<V>::getHashedKey(self.m_key) }
}

pub struct PlayerHash<V>
{
    m_hashArray: RefCell<Vec<Vec<PlayerHashEntry<V>>>>,
    m_numHashElements: Cell<i32>,
    m_capacity: Cell<i32>,
    m_percentUsable: f32,
    m_modCount: Cell<i32>,
}

impl<V: Clone> PlayerHash<V>
{
    pub fn new() -> PlayerHash<V>
    {
        PlayerHash {
            m_percentUsable: 0.75,
            m_capacity: Cell::new(12),
            m_hashArray: RefCell::new((0..16).map(|_| Vec::new()).collect()),
            m_numHashElements: Cell::new(0),
            m_modCount: Cell::new(0),
        }
    }

    fn getHashedKey(key: i64) -> i32 { Self::hash((key ^ (key as u64 >> 32) as i64) as i32) }

    fn hash(key: i32) -> i32
    {
        let key: i32 = key ^ ((key as u32 >> 20) as i32 ^ (key as u32 >> 12) as i32);
        key ^ (key as u32 >> 7) as i32 ^ (key as u32 >> 4) as i32
    }

    fn getHashIndex(hash: i32, cap: i32) -> usize { (hash & cap - 1) as usize }

    pub fn getValueByKey(&self, key: i64) -> Option<V>
    {
        let i: i32 = Self::getHashedKey(key);
        let table: std::cell::Ref<'_, Vec<Vec<PlayerHashEntry<V>>>> = self.m_hashArray.borrow();
        let bucket: &Vec<PlayerHashEntry<V>> = &table[Self::getHashIndex(i, table.len() as i32)];
        bucket.iter().find(|entry| entry.m_key == key).map(PlayerHashEntry::getValue)
    }

    pub fn add(&self, key: i64, value: V)
    {
        let i: i32 = Self::getHashedKey(key);
        let j: usize = Self::getHashIndex(i, self.m_hashArray.borrow().len() as i32);
        for entry in self.m_hashArray.borrow()[j].iter()
        {
            if entry.m_key == key
            {
                *entry.m_value.borrow_mut() = value.clone();
            }
        }
        self.m_modCount.set(self.m_modCount.get() + 1);
        self.createKey(i, key, value, j);
    }

    fn resizeTable(&self, cap: i32)
    {
        let i: usize = self.m_hashArray.borrow().len();
        if i == 1073741824
        {
            self.m_capacity.set(i32::MAX);
        }
        else
        {
            let mut nodes: Vec<Vec<PlayerHashEntry<V>>> = (0..cap).map(|_| Vec::new()).collect();
            self.copyHashTableTo(&mut nodes);
            *self.m_hashArray.borrow_mut() = nodes;
            self.m_capacity.set((cap as f32 * self.m_percentUsable) as i32);
        }
    }

    fn copyHashTableTo(&self, nodes: &mut [Vec<PlayerHashEntry<V>>])
    {
        let length: i32 = nodes.len() as i32;
        let mut hashArray: std::cell::RefMut<'_, Vec<Vec<PlayerHashEntry<V>>>> = self.m_hashArray.borrow_mut();
        for bucket in hashArray.iter_mut()
        {
            for playerHashEntry in std::mem::take(bucket)
            {
                let hashIndex: usize = Self::getHashIndex(playerHashEntry.m_hash, length);
                nodes[hashIndex].insert(0, playerHashEntry);
            }
        }
    }

    pub fn remove(&self, key: i64) -> Option<V> { self.removeKey(key).map(|entry| entry.getValue()) }

    fn removeKey(&self, key: i64) -> Option<PlayerHashEntry<V>>
    {
        let i: i32 = Self::getHashedKey(key);
        let mut table: std::cell::RefMut<'_, Vec<Vec<PlayerHashEntry<V>>>> = self.m_hashArray.borrow_mut();
        let j: usize = Self::getHashIndex(i, table.len() as i32);
        let position: usize = table[j].iter().position(|entry| entry.m_key == key)?;
        self.m_modCount.set(self.m_modCount.get() + 1);
        self.m_numHashElements.set(self.m_numHashElements.get() - 1);
        Some(table[j].remove(position))
    }

    fn createKey(&self, hash: i32, key: i64, value: V, index: usize)
    {
        self.m_hashArray.borrow_mut()[index].insert(0, PlayerHashEntry::new(hash, key, value));
        let numHashElements: i32 = self.m_numHashElements.get();
        self.m_numHashElements.set(numHashElements + 1);
        if numHashElements >= self.m_capacity.get()
        {
            let length: i32 = self.m_hashArray.borrow().len() as i32;
            self.resizeTable(2 * length);
        }
    }
}

impl<V: Clone> Default for PlayerHash<V>
{
    fn default() -> Self { Self::new() }
}
