use std::cell::Cell;

pub struct ChatLine
{
    pub m_message: String,
    pub m_updateCounter: Cell<i32>,
}

impl ChatLine
{
    pub fn new(text: String) -> ChatLine { ChatLine { m_message: text, m_updateCounter: Cell::new(0) } }
}
