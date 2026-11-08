use crate::argo::{JsonNode, JsonNodeBuilder, JsonNodeFactories};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Default)]
pub struct JsonArrayNodeBuilder
{
    m_elementBuilders: RefCell<Vec<Rc<dyn JsonNodeBuilder>>>,
}

impl JsonArrayNodeBuilder
{
    pub fn new() -> JsonArrayNodeBuilder { JsonArrayNodeBuilder::default() }

    pub fn withElement(&self, elementBuilder: Rc<dyn JsonNodeBuilder>) -> &Self
    {
        self.m_elementBuilders.borrow_mut().push(elementBuilder);
        self
    }
}

impl JsonNodeBuilder for JsonArrayNodeBuilder
{
    fn build(&self) -> Rc<JsonNode>
    {
        let builders: Vec<Rc<dyn JsonNodeBuilder>> = self.m_elementBuilders.borrow().clone();
        JsonNodeFactories::array(builders.iter().map(|builder| builder.build()).collect())
    }
}
