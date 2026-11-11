use crate::state;
use mod_protocol::abi::{GuestCall, Handle};
use std::cell::RefCell;

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
    KEYBINDS.with(|binds|
    {
        binds.borrow_mut().push(Keybind { m_mod: modIndex, m_name: name, m_key: key, m_handler: handler })
    });
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
