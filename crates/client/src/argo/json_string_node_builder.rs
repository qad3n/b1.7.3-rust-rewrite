use crate::argo::{JsonNode, JsonNodeBuilder, JsonNodeFactories, JsonStringNode};
use jvm::string::JString;
use std::rc::Rc;

pub struct JsonStringNodeBuilder
{
    m_value: JString,
}

impl JsonStringNodeBuilder
{
    pub fn new(value: JString) -> JsonStringNodeBuilder { JsonStringNodeBuilder { m_value: value } }

    pub fn buildString(&self) -> JsonStringNode { JsonNodeFactories::string(self.m_value.clone()) }
}

impl JsonNodeBuilder for JsonStringNodeBuilder
{
    fn build(&self) -> Rc<JsonNode> { Rc::new(JsonNode::String(self.buildString())) }

    fn buildKey(&self) -> JsonStringNode { self.buildString() }
}
