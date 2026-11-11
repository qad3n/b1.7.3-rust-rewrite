use std::cell::{Ref, RefCell, RefMut};
use std::collections::HashMap;

pub const LANG_FILE: &str = "lang/en_US.lang";

#[derive(Default)]
struct Tables
{
    m_loaded: HashMap<String, String>,
    // the mod whose lang file set each key, so a second mod replacing it is reported
    m_owners: HashMap<String, String>,
    // names given in specs, used when no lang file has the key
    m_defaults: HashMap<String, String>,
}

thread_local!
{
    static TABLES: RefCell<Tables> = RefCell::new(Tables::default());
}

// the vanilla key=value format, later files override earlier ones in load order
pub fn load(owner: &str, text: &str)
{
    TABLES.with(|tables|
    {
        let mut tables: RefMut<Tables> = tables.borrow_mut();

        for line in text.lines().map(str::trim).filter(|line| !line.is_empty() && !line.starts_with('#'))
        {
            let Some((key, value)) = line.split_once('=')
            else
            {
                continue;
            };

            let key: String = key.trim().to_owned();
            if let Some(previous) = tables.m_owners.insert(key.clone(), owner.to_owned())
                && previous != owner
            {
                crate::log::warn(owner, &format!("lang key {key} from {previous} is replaced"));
            }

            tables.m_loaded.insert(key, value.trim().to_owned());
        }
    });
}

pub fn addDefault(key: &str, value: &str)
{
    TABLES.with(|tables| tables.borrow_mut().m_defaults.insert(key.to_owned(), value.to_owned()));
}

pub fn add(key: &str, value: &str)
{
    TABLES.with(|tables| tables.borrow_mut().m_loaded.insert(key.to_owned(), value.to_owned()));
}

pub fn translate(key: &str) -> Option<String>
{
    TABLES.with(|tables|
    {
        let tables: Ref<Tables> = tables.borrow();
        tables.m_loaded.get(key).or_else(|| tables.m_defaults.get(key)).cloned()
    })
}

pub fn isEmpty() -> bool
{
    TABLES.with(|tables| tables.borrow().m_loaded.is_empty() && tables.borrow().m_defaults.is_empty())
}
