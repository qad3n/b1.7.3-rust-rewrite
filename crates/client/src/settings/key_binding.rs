use std::cell::Cell;
use std::rc::Rc;

pub struct KeyBinding
{
    pub m_keyDescription: String,
    pub m_keyCode: Cell<i32>,
}

impl KeyBinding
{
    pub fn new(name: &str, keyCode: i32) -> Rc<KeyBinding>
    {
        Rc::new(KeyBinding { m_keyDescription: name.to_owned(), m_keyCode: Cell::new(keyCode) })
    }
}
