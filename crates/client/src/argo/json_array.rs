use crate::argo::JsonNode;
use std::rc::Rc;

pub struct JsonArray
{
    m_elements: Vec<Rc<JsonNode>>,
}

impl JsonArray
{
    pub fn new(elements: Vec<Rc<JsonNode>>) -> JsonArray { JsonArray { m_elements: elements } }

    pub fn getElements(&self) -> Vec<Rc<JsonNode>> { self.m_elements.clone() }

    pub fn elements(&self) -> &[Rc<JsonNode>] { &self.m_elements }
}

impl JsonArray
{
    pub fn equals(&self, other: &JsonArray) -> bool
    {
        self.m_elements.len() == other.m_elements.len()
            && self.m_elements.iter().zip(&other.m_elements).all(|(a, b)| a.equals(b))
    }

    pub fn hashCode(&self) -> i32
    {
        self.m_elements.iter().fold(1i32, |hash, element| hash.wrapping_mul(31).wrapping_add(element.hashCode()))
    }
}
