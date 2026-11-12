use super::NBTBase;
use std::cell::RefCell;
use std::rc::Rc;

// list entries a reader could not resolve, such as content of a removed mod, written back unchanged
#[derive(Default)]
pub struct Unread
{
    m_tags: RefCell<Vec<Rc<NBTBase>>>,
}

impl Unread
{
    pub fn empty(&self) -> bool { self.m_tags.borrow().is_empty() }

    pub fn keep(&self, tag: Rc<NBTBase>) { self.m_tags.borrow_mut().push(tag); }

    pub fn clear(&self) { self.m_tags.borrow_mut().clear(); }

    pub fn writeInto(&self, list: &NBTBase)
    {
        for tag in self.m_tags.borrow().iter()
        {
            list.list().setTag(Rc::clone(tag));
        }
    }
}
