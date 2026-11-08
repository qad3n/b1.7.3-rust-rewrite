use crate::object::JavaObject;
use crate::throwable::{self, ARRAY_INDEX, CONCURRENT_MODIFICATION, INDEX_OUT_OF_BOUNDS, NO_SUCH_ELEMENT};
use std::cell::{Cell, RefCell};

// java.util.ArrayList of jdk 6 shared by reference, the modCount drives fail fast iteration like the original
pub struct JArrayList<T>
{
    m_items: RefCell<Vec<T>>,
    m_modCount: Cell<i32>,
}

impl<T: Clone> JArrayList<T>
{
    pub fn new() -> Self { Self { m_items: RefCell::new(Vec::new()), m_modCount: Cell::new(0) } }

    pub fn size(&self) -> i32 { self.m_items.borrow().len() as i32 }

    pub fn isEmpty(&self) -> bool { self.m_items.borrow().is_empty() }

    pub(crate) fn bump(&self) { self.m_modCount.set(self.m_modCount.get().wrapping_add(1)); }

    fn rangeCheck(&self, index: i32)
    {
        let size: i32 = self.size();
        if index >= size
        {
            throwable::throw(&INDEX_OUT_OF_BOUNDS, &format!("Index: {index}, Size: {size}"));
        }
        if index < 0
        {
            throwable::throw(&ARRAY_INDEX, &index.to_string());
        }
    }

    pub fn get(&self, index: i32) -> T
    {
        self.rangeCheck(index);
        self.m_items.borrow()[index as usize].clone()
    }

    pub fn set(&self, index: i32, value: T) -> T
    {
        self.rangeCheck(index);
        std::mem::replace(&mut self.m_items.borrow_mut()[index as usize], value)
    }

    pub fn add(&self, value: T) -> bool
    {
        self.bump();
        self.m_items.borrow_mut().push(value);
        true
    }

    pub fn addAt(&self, index: i32, value: T)
    {
        let size: i32 = self.size();
        if index > size || index < 0
        {
            throwable::throw(&INDEX_OUT_OF_BOUNDS, &format!("Index: {index}, Size: {size}"));
        }
        self.bump();
        self.m_items.borrow_mut().insert(index as usize, value);
    }

    pub fn removeAt(&self, index: i32) -> T
    {
        self.rangeCheck(index);
        self.bump();
        self.m_items.borrow_mut().remove(index as usize)
    }

    pub fn clear(&self)
    {
        self.bump();
        self.m_items.borrow_mut().clear();
    }

    pub fn addAll(&self, values: &[T]) -> bool
    {
        self.bump();
        self.m_items.borrow_mut().extend_from_slice(values);
        !values.is_empty()
    }

    pub fn toVec(&self) -> Vec<T> { self.m_items.borrow().clone() }

    pub fn replaceAll(&self, values: Vec<T>) { *self.m_items.borrow_mut() = values; }

    pub fn iterator(&self) -> ListIterator
    {
        ListIterator { m_cursor: 0, m_lastRet: -1, m_expectedModCount: self.m_modCount.get() }
    }
}

impl<T: Clone + JavaObject> JArrayList<T>
{
    pub fn indexOf(&self, value: &T) -> i32
    {
        let items = self.m_items.borrow();
        items.iter().position(|item| value.javaEquals(item)).map_or(-1, |index| index as i32)
    }

    pub fn contains(&self, value: &T) -> bool { self.indexOf(value) >= 0 }

    pub fn remove(&self, value: &T) -> bool
    {
        let index: i32 = self.indexOf(value);
        if index < 0
        {
            return false;
        }
        self.bump();
        self.m_items.borrow_mut().remove(index as usize);
        true
    }

    // AbstractCollection.removeAll walks its own iterator and asks the argument for contains
    pub fn removeAll(&self, values: &JArrayList<T>) -> bool
    {
        let mut modified: bool = false;
        let mut iterator: ListIterator = self.iterator();
        while iterator.hasNext(self)
        {
            if values.contains(&iterator.next(self))
            {
                iterator.remove(self);
                modified = true;
            }
        }
        modified
    }
}

impl<U: ?Sized> JArrayList<std::rc::Rc<U>>
{
    // remove(Object) for classes that keep Object.equals
    pub fn removeIdentity(&self, value: &std::rc::Rc<U>) -> bool
    {
        let index: Option<usize> = self.m_items.borrow().iter().position(|item| std::rc::Rc::ptr_eq(item, value));
        let Some(index) = index
        else
        {
            return false;
        };
        self.bump();
        self.m_items.borrow_mut().remove(index);
        true
    }

    pub fn containsIdentity(&self, value: &std::rc::Rc<U>) -> bool
    {
        self.m_items.borrow().iter().any(|item| std::rc::Rc::ptr_eq(item, value))
    }
}

impl<T: Clone> Default for JArrayList<T>
{
    fn default() -> Self { Self::new() }
}

// AbstractList.Itr: hasNext compares against size without a modCount check, as in jdk 6
pub struct ListIterator
{
    m_cursor: i32,
    m_lastRet: i32,
    m_expectedModCount: i32,
}

impl ListIterator
{
    pub fn hasNext<T: Clone>(&self, list: &JArrayList<T>) -> bool { self.m_cursor != list.size() }

    fn check<T: Clone>(&self, list: &JArrayList<T>)
    {
        if list.m_modCount.get() != self.m_expectedModCount
        {
            throwable::throwBare(&CONCURRENT_MODIFICATION);
        }
    }

    pub fn next<T: Clone>(&mut self, list: &JArrayList<T>) -> T
    {
        self.check(list);
        if self.m_cursor < 0 || self.m_cursor >= list.size()
        {
            self.check(list);
            throwable::throwBare(&NO_SUCH_ELEMENT);
        }

        let value: T = list.get(self.m_cursor);
        self.m_lastRet = self.m_cursor;
        self.m_cursor += 1;
        value
    }

    pub fn remove<T: Clone>(&mut self, list: &JArrayList<T>)
    {
        if self.m_lastRet == -1
        {
            throwable::throwBare(&throwable::ILLEGAL_STATE);
        }

        self.check(list);
        list.removeAt(self.m_lastRet);
        if self.m_lastRet < self.m_cursor
        {
            self.m_cursor -= 1;
        }
        self.m_lastRet = -1;
        self.m_expectedModCount = list.m_modCount.get();
    }
}
