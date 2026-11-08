use crate::object::JavaObject;
use crate::throwable::{self, CONCURRENT_MODIFICATION, NO_SUCH_ELEMENT};

const DEFAULT_INITIAL_CAPACITY: usize = 16;
const MAXIMUM_CAPACITY: usize = 1 << 30;
const DEFAULT_LOAD_FACTOR: f32 = 0.75;

#[derive(Clone)]
struct Entry<K, V>
{
    m_hash: i32,
    m_key: K,
    m_value: V,
    m_next: Option<usize>,
}

// java.util.HashMap of jdk 6: same bucket spread, head insertion, resize order and iteration order
#[derive(Clone)]
pub struct JHashMap<K, V>
{
    m_table: Vec<Option<usize>>,
    m_entries: Vec<Option<Entry<K, V>>>,
    m_free: Vec<usize>,
    m_size: i32,
    m_threshold: i32,
    m_loadFactor: f32,
    m_modCount: i32,
}

fn spread(h: i32) -> i32
{
    let h: i32 = h ^ (((h as u32) >> 20) ^ ((h as u32) >> 12)) as i32;
    h ^ ((h as u32) >> 7) as i32 ^ ((h as u32) >> 4) as i32
}

fn indexFor(h: i32, length: usize) -> usize { (h & (length as i32 - 1)) as usize }

impl<K: JavaObject + Clone, V: Clone> JHashMap<K, V>
{
    pub fn new() -> Self
    {
        Self {
            m_table: vec![None; DEFAULT_INITIAL_CAPACITY],
            m_entries: Vec::new(),
            m_free: Vec::new(),
            m_size: 0,
            m_threshold: (DEFAULT_INITIAL_CAPACITY as f32 * DEFAULT_LOAD_FACTOR) as i32,
            m_loadFactor: DEFAULT_LOAD_FACTOR,
            m_modCount: 0,
        }
    }

    pub fn withCapacity(initialCapacity: i32) -> Self
    {
        if initialCapacity < 0
        {
            throwable::throw(&throwable::ILLEGAL_ARGUMENT, &format!("Illegal initial capacity: {initialCapacity}"));
        }

        let wanted: usize = (initialCapacity as usize).min(MAXIMUM_CAPACITY);
        let mut capacity: usize = 1;
        while capacity < wanted
        {
            capacity <<= 1;
        }

        let mut map: Self = Self::new();
        map.m_table = vec![None; capacity];
        map.m_threshold = (capacity as f32 * DEFAULT_LOAD_FACTOR) as i32;
        map
    }

    pub fn size(&self) -> i32 { self.m_size }

    pub fn isEmpty(&self) -> bool { self.m_size == 0 }

    pub fn modCount(&self) -> i32 { self.m_modCount }

    fn entry(&self, index: usize) -> &Entry<K, V>
    {
        self.m_entries[index].as_ref().expect("hash map chain only links live entries")
    }

    fn entryMut(&mut self, index: usize) -> &mut Entry<K, V>
    {
        self.m_entries[index].as_mut().expect("hash map chain only links live entries")
    }

    fn find(&self, key: &K) -> Option<usize>
    {
        let hash: i32 = spread(key.javaHash());
        let mut current: Option<usize> = self.m_table[indexFor(hash, self.m_table.len())];
        while let Some(index) = current
        {
            let entry: &Entry<K, V> = self.entry(index);
            if entry.m_hash == hash && key.javaEquals(&entry.m_key)
            {
                return Some(index);
            }
            current = entry.m_next;
        }
        None
    }

    pub fn get(&self, key: &K) -> Option<V> { self.find(key).map(|index| self.entry(index).m_value.clone()) }

    pub fn containsKey(&self, key: &K) -> bool { self.find(key).is_some() }

    pub fn put(&mut self, key: K, value: V) -> Option<V>
    {
        let hash: i32 = spread(key.javaHash());
        let bucket: usize = indexFor(hash, self.m_table.len());
        let mut current: Option<usize> = self.m_table[bucket];
        while let Some(index) = current
        {
            let entry: &mut Entry<K, V> = self.entryMut(index);
            if entry.m_hash == hash && key.javaEquals(&entry.m_key)
            {
                return Some(std::mem::replace(&mut entry.m_value, value));
            }
            current = entry.m_next;
        }

        self.m_modCount = self.m_modCount.wrapping_add(1);
        self.addEntry(hash, key, value, bucket);
        None
    }

    fn addEntry(&mut self, hash: i32, key: K, value: V, bucket: usize)
    {
        let next: Option<usize> = self.m_table[bucket];
        let entry: Entry<K, V> = Entry { m_hash: hash, m_key: key, m_value: value, m_next: next };
        let index: usize = match self.m_free.pop()
        {
            Some(index) =>
            {
                self.m_entries[index] = Some(entry);
                index
            }
            None =>
            {
                self.m_entries.push(Some(entry));
                self.m_entries.len() - 1
            }
        };
        self.m_table[bucket] = Some(index);

        let previous: i32 = self.m_size;
        self.m_size += 1;
        if previous >= self.m_threshold
        {
            self.resize(2 * self.m_table.len());
        }
    }

    fn resize(&mut self, newCapacity: usize)
    {
        if self.m_table.len() == MAXIMUM_CAPACITY
        {
            self.m_threshold = i32::MAX;
            return;
        }

        let mut newTable: Vec<Option<usize>> = vec![None; newCapacity];
        for j in 0..self.m_table.len()
        {
            let mut current: Option<usize> = self.m_table[j].take();
            while let Some(index) = current
            {
                let hash: i32 = self.entry(index).m_hash;
                let next: Option<usize> = self.entry(index).m_next;
                let bucket: usize = indexFor(hash, newCapacity);
                self.entryMut(index).m_next = newTable[bucket];
                newTable[bucket] = Some(index);
                current = next;
            }
        }

        self.m_table = newTable;
        self.m_threshold = (newCapacity as f32 * self.m_loadFactor) as i32;
    }

    pub fn remove(&mut self, key: &K) -> Option<V>
    {
        let hash: i32 = spread(key.javaHash());
        let bucket: usize = indexFor(hash, self.m_table.len());
        let mut previous: Option<usize> = None;
        let mut current: Option<usize> = self.m_table[bucket];
        while let Some(index) = current
        {
            let next: Option<usize> = self.entry(index).m_next;
            if self.entry(index).m_hash == hash && key.javaEquals(&self.entry(index).m_key)
            {
                self.m_modCount = self.m_modCount.wrapping_add(1);
                self.m_size -= 1;
                match previous
                {
                    None => self.m_table[bucket] = next,
                    Some(previous) => self.entryMut(previous).m_next = next,
                }
                let removed: Entry<K, V> = self.m_entries[index].take().expect("removed entry is live");
                self.m_free.push(index);
                return Some(removed.m_value);
            }
            previous = Some(index);
            current = next;
        }
        None
    }

    pub fn clear(&mut self)
    {
        self.m_modCount = self.m_modCount.wrapping_add(1);
        self.m_table.iter_mut().for_each(|slot| *slot = None);
        self.m_entries.clear();
        self.m_free.clear();
        self.m_size = 0;
    }

    pub fn cursor(&self) -> HashCursor
    {
        let mut cursor: HashCursor =
            HashCursor { m_next: None, m_index: 0, m_current: None, m_expectedModCount: self.m_modCount };
        if self.m_size > 0
        {
            cursor.advance(self);
        }
        cursor
    }

    pub fn entries(&self) -> Vec<(K, V)>
    {
        let mut cursor: HashCursor = self.cursor();
        let mut out: Vec<(K, V)> = Vec::with_capacity(self.m_size as usize);
        while cursor.hasNext()
        {
            out.push(cursor.nextEntry(self));
        }
        out
    }

    pub fn keys(&self) -> Vec<K> { self.entries().into_iter().map(|(key, _)| key).collect() }

    pub fn values(&self) -> Vec<V> { self.entries().into_iter().map(|(_, value)| value).collect() }
}

impl<K: JavaObject + Clone, V: Clone> Default for JHashMap<K, V>
{
    fn default() -> Self { Self::new() }
}

// HashMap.HashIterator detached from the map so the map can change between steps as it can in java
pub struct HashCursor
{
    m_next: Option<usize>,
    m_index: usize,
    m_current: Option<usize>,
    m_expectedModCount: i32,
}

impl HashCursor
{
    fn advance<K: JavaObject + Clone, V: Clone>(&mut self, map: &JHashMap<K, V>)
    {
        while self.m_index < map.m_table.len()
        {
            self.m_next = map.m_table[self.m_index];
            self.m_index += 1;
            if self.m_next.is_some()
            {
                return;
            }
        }
        self.m_next = None;
    }

    pub fn hasNext(&self) -> bool { self.m_next.is_some() }

    pub fn nextEntry<K: JavaObject + Clone, V: Clone>(&mut self, map: &JHashMap<K, V>) -> (K, V)
    {
        if map.m_modCount != self.m_expectedModCount
        {
            throwable::throwBare(&CONCURRENT_MODIFICATION);
        }

        let Some(index) = self.m_next
        else
        {
            throwable::throwBare(&NO_SUCH_ELEMENT);
        };

        let entry: &Entry<K, V> = map.entry(index);
        self.m_next = entry.m_next;
        if self.m_next.is_none()
        {
            self.advance(map);
        }
        self.m_current = Some(index);
        (entry.m_key.clone(), entry.m_value.clone())
    }

    pub fn remove<K: JavaObject + Clone, V: Clone>(&mut self, map: &mut JHashMap<K, V>)
    {
        let Some(index) = self.m_current.take()
        else
        {
            throwable::throwBare(&throwable::ILLEGAL_STATE);
        };

        if map.m_modCount != self.m_expectedModCount
        {
            throwable::throwBare(&CONCURRENT_MODIFICATION);
        }

        let key: K = map.entry(index).m_key.clone();
        map.remove(&key);
        self.m_expectedModCount = map.m_modCount;
    }
}

// java.util.HashSet is a HashMap with a shared present value
#[derive(Clone)]
pub struct JHashSet<K>
{
    m_map: JHashMap<K, ()>,
}

impl<K: JavaObject + Clone> JHashSet<K>
{
    pub fn new() -> Self { Self { m_map: JHashMap::new() } }

    pub fn copyOf(source: &JHashSet<K>) -> Self
    {
        let mut set: Self = Self { m_map: JHashMap::withCapacity(((source.size() as f32 / 0.75) as i32 + 1).max(16)) };
        for key in source.items()
        {
            set.add(key);
        }
        set
    }

    pub fn size(&self) -> i32 { self.m_map.size() }

    pub fn isEmpty(&self) -> bool { self.m_map.isEmpty() }

    pub fn contains(&self, key: &K) -> bool { self.m_map.containsKey(key) }

    pub fn add(&mut self, key: K) -> bool { self.m_map.put(key, ()).is_none() }

    pub fn remove(&mut self, key: &K) -> bool { self.m_map.remove(key).is_some() }

    pub fn clear(&mut self) { self.m_map.clear(); }

    pub fn cursor(&self) -> HashCursor { self.m_map.cursor() }

    pub fn nextKey(&self, cursor: &mut HashCursor) -> K { cursor.nextEntry(&self.m_map).0 }

    pub fn removeAt(&mut self, cursor: &mut HashCursor) { cursor.remove(&mut self.m_map); }

    pub fn items(&self) -> Vec<K> { self.m_map.keys() }
}

impl<K: JavaObject + Clone> Default for JHashSet<K>
{
    fn default() -> Self { Self::new() }
}
