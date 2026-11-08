use std::cell::RefCell;
use std::rc::Rc;

thread_local! {
    static FOLIAGE_BUFFER: RefCell<Rc<RefCell<Vec<i32>>>> = RefCell::new(Rc::new(RefCell::new(vec![0; 65536])));
}

pub struct ColorizerFoliage;

impl ColorizerFoliage
{
    pub fn set(colors: Rc<RefCell<Vec<i32>>>) { FOLIAGE_BUFFER.with(|buffer| *buffer.borrow_mut() = colors); }

    pub fn getFoliageColor(temperature: f64, mut humidity: f64) -> i32
    {
        humidity *= temperature;
        let i: i32 = ((1.0 - temperature) * 255.0) as i32;
        let j: i32 = ((1.0 - humidity) * 255.0) as i32;
        FOLIAGE_BUFFER.with(|buffer| buffer.borrow().borrow()[((j << 8) | i) as usize])
    }

    pub fn getFoliageColorPine() -> i32 { 6396257 }

    pub fn getFoliageColorBirch() -> i32 { 8431445 }

    pub fn getDefaultColor() -> i32 { 4764952 }
}
