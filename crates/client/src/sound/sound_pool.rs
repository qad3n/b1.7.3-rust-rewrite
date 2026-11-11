use crate::port::file_url;
use crate::sound::SoundPoolEntry;
use jvm::random::Random;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;

pub struct SoundPool
{
    m_rand: Random,
    m_nameToSoundPoolEntriesMapping: RefCell<HashMap<String, Vec<Rc<SoundPoolEntry>>>>,
    m_allSoundPoolEntries: RefCell<Vec<Rc<SoundPoolEntry>>>,
    pub m_numberOfSoundPoolEntries: Cell<i32>,
    pub m_trimTrailingDigits: Cell<bool>,
}

fn stringIndex(index: i64) -> !
{
    jvm::throwable::throw(&jvm::throwable::STRING_INDEX, &format!("String index out of range: {index}"))
}

impl SoundPool
{
    pub fn new() -> Self
    {
        Self
        {
            m_rand: Random::new(),
            m_nameToSoundPoolEntriesMapping: RefCell::new(HashMap::new()),
            m_allSoundPoolEntries: RefCell::new(Vec::new()),
            m_numberOfSoundPoolEntries: Cell::new(0),
            m_trimTrailingDigits: Cell::new(true),
        }
    }

    pub fn addSound(&self, path: &str, file: &Path) -> Rc<SoundPoolEntry>
    {
        let key: String = self.key(path);
        self.add(key, path.to_owned(), file)
    }

    // the pool name of a file such as random/click1.ogg is random.click
    pub fn key(&self, path: &str) -> String
    {
        let mut path: String = match path.find('.')
        {
            Some(index) => path[..index].to_owned(),
            None => stringIndex(-1),
        };

        if self.m_trimTrailingDigits.get()
        {
            loop
            {
                match path.encode_utf16().last()
                {
                    Some(unit) if jvm::character::isDigit(unit as i32) => _ = path.pop(),
                    Some(_) => break,
                    None => stringIndex(-1),
                }
            }
        }

        path.replace('/', ".")
    }

    pub fn add(&self, key: String, name: String, file: &Path) -> Rc<SoundPoolEntry>
    {
        // port fileUrl is java File.toURI().toURL()
        let soundPoolEntry: Rc<SoundPoolEntry> = Rc::new(SoundPoolEntry::new(name, file_url::fileUrl(file)));
        self.m_nameToSoundPoolEntriesMapping.borrow_mut().entry(key).or_default().push(Rc::clone(&soundPoolEntry));
        self.m_allSoundPoolEntries.borrow_mut().push(Rc::clone(&soundPoolEntry));
        self.m_numberOfSoundPoolEntries.set(self.m_numberOfSoundPoolEntries.get() + 1);
        soundPoolEntry
    }

    // drops every entry so a texture pack change can add them again
    pub fn clear(&self)
    {
        self.m_nameToSoundPoolEntriesMapping.borrow_mut().clear();
        self.m_allSoundPoolEntries.borrow_mut().clear();
        self.m_numberOfSoundPoolEntries.set(0);
    }

    pub fn remove(&self, key: &str)
    {
        let Some(removed) = self.m_nameToSoundPoolEntriesMapping.borrow_mut().remove(key)
        else
        {
            return;
        };

        self.m_allSoundPoolEntries.borrow_mut().retain(|entry| !removed.iter().any(|gone| Rc::ptr_eq(gone, entry)));
        self.m_numberOfSoundPoolEntries.set(self.m_numberOfSoundPoolEntries.get() - removed.len() as i32);
    }

    pub fn getRandomSoundFromSoundPool(&self, path: &str) -> Option<Rc<SoundPoolEntry>>
    {
        let mapping: std::cell::Ref<HashMap<String, Vec<Rc<SoundPoolEntry>>>> =
            self.m_nameToSoundPoolEntriesMapping.borrow();
        let list: &Vec<Rc<SoundPoolEntry>> = mapping.get(path)?;
        Some(Rc::clone(&list[self.m_rand.nextIntBounded(list.len() as i32) as usize]))
    }

    pub fn getRandomSound(&self) -> Option<Rc<SoundPoolEntry>>
    {
        let entries: std::cell::Ref<Vec<Rc<SoundPoolEntry>>> = self.m_allSoundPoolEntries.borrow();
        if entries.is_empty()
        {
            return None;
        }

        Some(Rc::clone(&entries[self.m_rand.nextIntBounded(entries.len() as i32) as usize]))
    }
}

impl Default for SoundPool
{
    fn default() -> Self { Self::new() }
}
