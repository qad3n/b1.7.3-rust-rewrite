use std::collections::HashMap;

struct Entries<V>
{
    m_value: V,
    m_count: i32,
}

pub struct MCHash<V>
{
    m_slots: HashMap<i32, Entries<V>>,
}

impl<V: Clone> MCHash<V>
{
    pub fn new() -> Self { Self { m_slots: HashMap::new() } }

    pub fn lookup(&self, key: i32) -> Option<V> { self.m_slots.get(&key).map(|entries| entries.m_value.clone()) }

    pub fn addKey(&mut self, key: i32, value: V)
    {
        let entries: &mut Entries<V> =
            self.m_slots.entry(key).or_insert_with(|| Entries { m_value: value.clone(), m_count: 0 });
        entries.m_value = value;
        entries.m_count += 1;
    }

    pub fn removeObject(&mut self, key: i32) -> Option<V>
    {
        let entries: &mut Entries<V> = self.m_slots.get_mut(&key)?;
        let value: V = entries.m_value.clone();
        entries.m_count -= 1;
        if entries.m_count == 0
        {
            self.m_slots.remove(&key);
        }
        Some(value)
    }

    pub fn clearMap(&mut self) { self.m_slots.clear(); }

    pub fn containsItem(&self, key: i32) -> bool { self.lookupEntry(key).is_some() }

    fn lookupEntry(&self, key: i32) -> Option<V> { self.lookup(key) }
}

impl<V: Clone> Default for MCHash<V>
{
    fn default() -> Self { Self::new() }
}
