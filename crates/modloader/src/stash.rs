use std::cell::RefCell;

// the client's own copy of content a modded server replaced, put back when the client leaves that server
pub struct Stash<T>(RefCell<Option<T>>);

impl<T> Stash<T>
{
    pub const fn new() -> Self { Stash(RefCell::new(None)) }

    // a second replacement keeps the first own copy, never the server's
    pub fn keep(&self, own: T) { self.0.borrow_mut().get_or_insert(own); }

    pub fn take(&self) -> Option<T> { self.0.borrow_mut().take() }
}
