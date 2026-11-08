use std::cell::RefCell;
use std::rc::Rc;

thread_local! {
    static GRASS_BUFFER: RefCell<Rc<RefCell<Vec<i32>>>> = RefCell::new(Rc::new(RefCell::new(vec![0; 65536])));
}

pub struct ColorizerGrass;

impl ColorizerGrass
{
    pub fn set(colors: Rc<RefCell<Vec<i32>>>) { GRASS_BUFFER.with(|buffer| *buffer.borrow_mut() = colors); }

    pub fn getGrassColor(temperature: f64, mut humidity: f64) -> i32
    {
        humidity *= temperature;
        let i: i32 = ((1.0 - temperature) * 255.0) as i32;
        let j: i32 = ((1.0 - humidity) * 255.0) as i32;
        GRASS_BUFFER.with(|buffer| buffer.borrow().borrow()[((j << 8) | i) as usize])
    }
}
