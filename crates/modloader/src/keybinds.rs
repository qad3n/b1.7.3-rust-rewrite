use crate::state;
use mod_protocol::abi::{GuestCall, Handle};
use std::cell::RefCell;
use std::collections::BTreeMap;

// the player's own key for each mod keybind by name, kept across starts
const KEYS_FILE: &str = "keybinds.toml";

#[derive(Clone)]
pub struct Keybind
{
    pub m_mod: usize,
    pub m_name: String,
    pub m_key: i32,
    pub m_handler: Handle,
}

thread_local!
{
    static KEYBINDS: RefCell<Vec<Keybind>> = const { RefCell::new(Vec::new()) };
}

pub fn register(modIndex: usize, name: String, key: i32, handler: Handle)
{
    let key: i32 = saved().get(&name).copied().unwrap_or(key);
    KEYBINDS.with(|binds|
    {
        binds.borrow_mut().push(Keybind { m_mod: modIndex, m_name: name, m_key: key, m_handler: handler })
    });
}

fn file() -> Option<std::path::PathBuf> { crate::paths().map(|paths| paths.m_config.join(KEYS_FILE)) }

fn saved() -> BTreeMap<String, i32>
{
    let Some(text) = file().and_then(|path| std::fs::read_to_string(path).ok())
    else
    {
        return BTreeMap::new();
    };

    toml::from_str(&text).unwrap_or_else(|error|
    {
        crate::log::warn(crate::log::LOADER, &format!("{KEYS_FILE} could not be read: {error}"));
        BTreeMap::new()
    })
}

fn save()
{
    let keys: BTreeMap<String, i32> = all().into_iter().map(|bind| (bind.m_name, bind.m_key)).collect();
    let written: Result<(), String> = toml::to_string(&keys)
        .map_err(|error| error.to_string())
        .and_then(|text| file().ok_or_else(|| "no config folder".to_owned()).map(|path| (path, text)))
        .and_then(|(path, text)| std::fs::write(path, text).map_err(|error| error.to_string()));
    if let Err(error) = written
    {
        crate::log::warn(crate::log::LOADER, &format!("{KEYS_FILE} could not be saved: {error}"));
    }
}

pub fn all() -> Vec<Keybind> { KEYBINDS.with(|binds| binds.borrow().clone()) }

pub fn setKey(name: &str, key: i32)
{
    KEYBINDS.with(|binds|
    {
        for bind in binds.borrow_mut().iter_mut().filter(|bind| bind.m_name == name)
        {
            bind.m_key = key;
        }
    });
    save();
}

// conflicts are shown to the player, never refused
pub fn conflicts() -> Vec<(String, String)>
{
    let binds: Vec<Keybind> = all();
    let mut found: Vec<(String, String)> = Vec::new();

    for (index, bind) in binds.iter().enumerate()
    {
        for other in &binds[index + 1..]
        {
            if bind.m_key == other.m_key
            {
                found.push((bind.m_name.clone(), other.m_name.clone()));
            }
        }
    }

    found
}

pub fn pressed(key: i32, down: bool)
{
    for bind in all().into_iter().filter(|bind| bind.m_key == key)
    {
        if let Some(runtime) = state::modAt(bind.m_mod)
        {
            runtime.call(&GuestCall::Key { handler: bind.m_handler, pressed: down });
        }
    }
}
