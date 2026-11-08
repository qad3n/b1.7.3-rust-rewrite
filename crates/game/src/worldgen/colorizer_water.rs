use std::cell::RefCell;
use std::rc::Rc;

thread_local! {
    static WATER_BUFFER: RefCell<Rc<RefCell<Vec<i32>>>> = RefCell::new(Rc::new(RefCell::new(vec![0; 65536])));
}

pub struct ColorizerWater;

impl ColorizerWater
{
    pub fn set(colors: Rc<RefCell<Vec<i32>>>) { WATER_BUFFER.with(|buffer| *buffer.borrow_mut() = colors); }
}
