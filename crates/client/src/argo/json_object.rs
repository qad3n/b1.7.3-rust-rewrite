use crate::argo::{JsonNode, JsonStringNode};
use jvm::hash_map::JHashMap;
use std::rc::Rc;

pub type JsonFields = JHashMap<JsonStringNode, Rc<JsonNode>>;

pub fn copyFields(fields: &JsonFields) -> JsonFields
{
    let capacity: i32 = ((fields.size() as f32 / 0.75) as i32 + 1).max(16);
    let mut copy: JsonFields = JHashMap::withCapacity(capacity);
    for (key, value) in fields.entries()
    {
        copy.put(key, value);
    }
    copy
}

pub struct JsonObject
{
    m_fields: JsonFields,
}

impl JsonObject
{
    pub fn new(fields: &JsonFields) -> JsonObject { JsonObject { m_fields: copyFields(fields) } }

    pub fn getFields(&self) -> JsonFields { copyFields(&self.m_fields) }

    pub fn fields(&self) -> &JsonFields { &self.m_fields }
}

impl JsonObject
{
    pub fn equals(&self, other: &JsonObject) -> bool
    {
        if self.m_fields.size() != other.m_fields.size()
        {
            return false;
        }
        self.m_fields
            .entries()
            .iter()
            .all(|(key, value)| other.m_fields.get(key).is_some_and(|found| value.equals(&found)))
    }

    pub fn hashCode(&self) -> i32
    {
        self.m_fields
            .entries()
            .iter()
            .fold(0i32, |hash, (key, value)| hash.wrapping_add(key.getText().hashCode() ^ value.hashCode()))
    }
}
