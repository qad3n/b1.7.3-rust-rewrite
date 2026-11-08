use crate::argo::{
    JsonArrayNodeBuilder, JsonNode, JsonNodeBuilder, JsonNodeFactories, JsonNumberNodeBuilder, JsonObjectNodeBuilder,
    JsonStringNodeBuilder,
};
use jvm::string::JString;
use std::rc::Rc;

struct ConstantBuilder
{
    m_factory: fn() -> Rc<JsonNode>,
}

impl JsonNodeBuilder for ConstantBuilder
{
    fn build(&self) -> Rc<JsonNode> { (self.m_factory)() }
}

pub struct JsonNodeBuilders;

impl JsonNodeBuilders
{
    pub fn aNullBuilder() -> Rc<dyn JsonNodeBuilder>
    {
        Rc::new(ConstantBuilder { m_factory: JsonNodeFactories::nullNode })
    }

    pub fn aTrueBuilder() -> Rc<dyn JsonNodeBuilder>
    {
        Rc::new(ConstantBuilder { m_factory: JsonNodeFactories::trueNode })
    }

    pub fn aFalseBuilder() -> Rc<dyn JsonNodeBuilder>
    {
        Rc::new(ConstantBuilder { m_factory: JsonNodeFactories::falseNode })
    }

    pub fn aNumberBuilder(value: JString) -> Rc<dyn JsonNodeBuilder> { Rc::new(JsonNumberNodeBuilder::new(value)) }

    pub fn aStringBuilder(value: JString) -> Rc<JsonStringNodeBuilder> { Rc::new(JsonStringNodeBuilder::new(value)) }

    pub fn anObjectBuilder() -> Rc<JsonObjectNodeBuilder> { Rc::new(JsonObjectNodeBuilder::new()) }

    pub fn anArrayBuilder() -> Rc<JsonArrayNodeBuilder> { Rc::new(JsonArrayNodeBuilder::new()) }
}
