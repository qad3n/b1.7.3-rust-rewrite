use crate::argo::json_object::JsonFields;
use crate::argo::{JsonFieldBuilder, JsonNode, JsonNodeBuilder, JsonNodeFactories};
use jvm::hash_map::JHashMap;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Default)]
pub struct JsonObjectNodeBuilder
{
    m_fieldBuilders: RefCell<Vec<Rc<JsonFieldBuilder>>>,
}

impl JsonObjectNodeBuilder
{
    pub fn new() -> JsonObjectNodeBuilder { JsonObjectNodeBuilder::default() }

    pub fn withField(&self, fieldBuilder: Rc<JsonFieldBuilder>) -> &Self
    {
        self.m_fieldBuilders.borrow_mut().push(fieldBuilder);
        self
    }
}

impl JsonNodeBuilder for JsonObjectNodeBuilder
{
    fn build(&self) -> Rc<JsonNode>
    {
        let mut fields: JsonFields = JHashMap::new();
        for fieldBuilder in self.m_fieldBuilders.borrow().clone()
        {
            fields.put(fieldBuilder.buildKey(), fieldBuilder.buildValue());
        }
        JsonNodeFactories::object(&fields)
    }
}
