use crate::world::Chunk;
use jvm::field::Field;
use jvm::hash_map::JHashMap;
use jvm::object::JavaObject;
use std::cell::RefCell;
use std::rc::Rc;

// a provider's chunk map that remembers its last hit, any change to the map forgets it
pub struct ChunkMap<K>
{
    m_map: Field<JHashMap<K, Rc<Chunk>>>,
    m_last: RefCell<Option<(K, Rc<Chunk>)>>,
}

impl<K: JavaObject + Clone> ChunkMap<K>
{
    pub fn new() -> Self { Self { m_map: Field::new(JHashMap::new()), m_last: RefCell::new(None) } }

    pub fn get(&self, key: &K) -> Option<Rc<Chunk>>
    {
        if let Some((last, chunk)) = &*self.m_last.borrow()
            && last.javaEquals(key)
        {
            return Some(Rc::clone(chunk));
        }

        let chunk: Option<Rc<Chunk>> = self.m_map.with(|map| map.get(key));
        if let Some(chunk) = &chunk
        {
            self.m_last.replace(Some((key.clone(), Rc::clone(chunk))));
        }
        chunk
    }

    pub fn containsKey(&self, key: &K) -> bool { self.get(key).is_some() }

    pub fn with<R>(&self, body: impl FnOnce(&JHashMap<K, Rc<Chunk>>) -> R) -> R { self.m_map.with(body) }

    pub fn update<R>(&self, body: impl FnOnce(&mut JHashMap<K, Rc<Chunk>>) -> R) -> R
    {
        self.m_last.replace(None);
        self.m_map.update(body)
    }
}
