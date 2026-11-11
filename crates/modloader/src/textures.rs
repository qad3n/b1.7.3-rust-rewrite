use std::cell::{RefCell, RefMut};
use std::collections::HashMap;
use std::rc::Rc;

// textures mods draw at runtime, argb pixels by texture name, a new version each time a mod sends them
#[derive(Clone)]
pub struct Dynamic
{
    pub m_version: u32,
    pub m_width: u32,
    pub m_height: u32,
    pub m_pixels: Rc<Vec<u32>>,
}

thread_local!
{
    static DYNAMIC: RefCell<HashMap<String, Dynamic>> = RefCell::new(HashMap::new());
}

pub fn set(name: String, width: u32, height: u32, pixels: Vec<u32>)
{
    DYNAMIC.with(|textures|
    {
        let mut textures: RefMut<HashMap<String, Dynamic>> = textures.borrow_mut();
        let version: u32 = textures.get(&name).map_or(1, |known| known.m_version.wrapping_add(1));
        textures
            .insert(name, Dynamic { m_version: version, m_width: width, m_height: height, m_pixels: Rc::new(pixels) });
    });
}

pub fn get(name: &str) -> Option<Dynamic> { DYNAMIC.with(|textures| textures.borrow().get(name).cloned()) }
