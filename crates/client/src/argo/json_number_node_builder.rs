use crate::argo::{JsonNode, JsonNodeBuilder, JsonNodeFactories};
use jvm::string::JString;
use std::rc::Rc;

pub struct JsonNumberNodeBuilder
{
    m_value: Rc<JsonNode>,
}

impl JsonNumberNodeBuilder
{
    pub fn new(value: JString) -> JsonNumberNodeBuilder
    {
        JsonNumberNodeBuilder { m_value: JsonNodeFactories::number(value) }
    }
}

impl JsonNodeBuilder for JsonNumberNodeBuilder
{
    fn build(&self) -> Rc<JsonNode> { Rc::clone(&self.m_value) }
}
