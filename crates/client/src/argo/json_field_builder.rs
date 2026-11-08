use crate::argo::{JsonNode, JsonNodeBuilder, JsonStringNode};
use jvm::field::Field;
use jvm::throwable::OrNpe;
use std::rc::Rc;

pub struct JsonFieldBuilder
{
    m_keyBuilder: Field<Option<Rc<dyn JsonNodeBuilder>>>,
    m_valueBuilder: Field<Option<Rc<dyn JsonNodeBuilder>>>,
}

impl JsonFieldBuilder
{
    pub fn aJsonFieldBuilder() -> JsonFieldBuilder
    {
        JsonFieldBuilder { m_keyBuilder: Field::new(None), m_valueBuilder: Field::new(None) }
    }

    pub fn withKey(self, keyBuilder: Rc<dyn JsonNodeBuilder>) -> JsonFieldBuilder
    {
        self.m_keyBuilder.set(Some(keyBuilder));
        self
    }

    pub fn withValue(&self, valueBuilder: Rc<dyn JsonNodeBuilder>) -> &Self
    {
        self.m_valueBuilder.set(Some(valueBuilder));
        self
    }

    pub fn buildKey(&self) -> JsonStringNode { self.m_keyBuilder.get().orNpe().buildKey() }

    pub fn buildValue(&self) -> Rc<JsonNode> { self.m_valueBuilder.get().orNpe().build() }
}
