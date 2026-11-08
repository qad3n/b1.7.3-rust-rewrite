use std::cell::RefCell;
use std::rc::{Rc, Weak};

// a java reference or object field, the borrow never outlives one call so reentrant code cannot panic
#[derive(Default)]
pub struct Field<T>
{
    m_value: RefCell<T>,
}

impl<T> Field<T>
{
    pub fn new(value: T) -> Self { Self { m_value: RefCell::new(value) } }

    pub fn set(&self, value: T) { *self.m_value.borrow_mut() = value; }

    pub fn replace(&self, value: T) -> T { std::mem::replace(&mut *self.m_value.borrow_mut(), value) }

    pub fn with<R>(&self, body: impl FnOnce(&T) -> R) -> R { body(&self.m_value.borrow()) }

    pub fn update<R>(&self, body: impl FnOnce(&mut T) -> R) -> R { body(&mut self.m_value.borrow_mut()) }
}

impl<T: Clone> Field<T>
{
    pub fn get(&self) -> T { self.m_value.borrow().clone() }
}

impl<T: Default> Field<T>
{
    pub fn take(&self) -> T { std::mem::take(&mut *self.m_value.borrow_mut()) }
}

// a java reference field whose target owns this object, held weakly so the pair is freed like the java gc frees it
pub struct WeakField<T>
{
    m_value: RefCell<Weak<T>>,
}

impl<T> WeakField<T>
{
    pub fn new(value: Option<Rc<T>>) -> Self { Self { m_value: RefCell::new(downgrade(value)) } }

    pub fn get(&self) -> Option<Rc<T>> { self.m_value.borrow().upgrade() }

    pub fn set(&self, value: Option<Rc<T>>) { *self.m_value.borrow_mut() = downgrade(value); }

    pub fn with<R>(&self, body: impl FnOnce(&Option<Rc<T>>) -> R) -> R { body(&self.get()) }
}

impl<T> Default for WeakField<T>
{
    fn default() -> Self { Self { m_value: RefCell::new(Weak::new()) } }
}

fn downgrade<T>(value: Option<Rc<T>>) -> Weak<T> { value.as_ref().map_or_else(Weak::new, Rc::downgrade) }
