use crate::port::awt::Component;
use crate::render::GLAllocation;
use lwjgl::buffers::IntBuffer;
use lwjgl::input::{Cursor, Mouse};
use std::cell::{Cell, RefCell};
use std::rc::Rc;

pub struct MouseHelper
{
    m_component: Rc<dyn Component>,
    m_cursor: RefCell<Option<Cursor>>,
    pub m_deltaX: Cell<i32>,
    pub m_deltaY: Cell<i32>,
    pub m_field_1115_e: Cell<i32>,
}

impl MouseHelper
{
    pub fn new(component: Rc<dyn Component>) -> MouseHelper
    {
        let helper: MouseHelper = MouseHelper {
            m_component: component,
            m_cursor: RefCell::new(None),
            m_deltaX: Cell::new(0),
            m_deltaY: Cell::new(0),
            m_field_1115_e: Cell::new(10),
        };
        let intBuffer: IntBuffer = GLAllocation::createDirectIntBuffer(1);
        intBuffer.put(0);
        intBuffer.flip();
        let intBuffer2: IntBuffer = GLAllocation::createDirectIntBuffer(1024);
        match Cursor::new(32, 32, 16, 16, 1, &intBuffer2, Some(&intBuffer))
        {
            Ok(cursor) => *helper.m_cursor.borrow_mut() = Some(cursor),
            Err(exception) => exception.printStackTrace(),
        }
        helper
    }

    pub fn grabMouseCursor(&self)
    {
        Mouse::setGrabbed(true);
        self.m_deltaX.set(0);
        self.m_deltaY.set(0);
    }

    pub fn ungrabMouseCursor(&self)
    {
        Mouse::setCursorPosition(self.m_component.getWidth() / 2, self.m_component.getHeight() / 2);
        Mouse::setGrabbed(false);
    }

    pub fn mouseXYChange(&self)
    {
        self.m_deltaX.set(Mouse::getDX());
        self.m_deltaY.set(Mouse::getDY());
    }
}
