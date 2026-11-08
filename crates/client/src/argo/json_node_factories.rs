use crate::argo::json_object::JsonFields;
use crate::argo::{EnumJsonNodeType, JsonArray, JsonNode, JsonNumberNode, JsonObject, JsonStringNode};
use jvm::string::JString;
use std::rc::Rc;

thread_local! {
    static NULL: Rc<JsonNode> = Rc::new(JsonNode::Constant(EnumJsonNodeType::Null));
    static TRUE: Rc<JsonNode> = Rc::new(JsonNode::Constant(EnumJsonNodeType::True));
    static FALSE: Rc<JsonNode> = Rc::new(JsonNode::Constant(EnumJsonNodeType::False));
}

pub struct JsonNodeFactories;

impl JsonNodeFactories
{
    pub fn nullNode() -> Rc<JsonNode> { NULL.with(Rc::clone) }

    pub fn trueNode() -> Rc<JsonNode> { TRUE.with(Rc::clone) }

    pub fn falseNode() -> Rc<JsonNode> { FALSE.with(Rc::clone) }

    pub fn string(value: JString) -> JsonStringNode { JsonStringNode::new(value) }

    pub fn number(value: JString) -> Rc<JsonNode> { Rc::new(JsonNode::Number(JsonNumberNode::new(value))) }

    pub fn array(elements: Vec<Rc<JsonNode>>) -> Rc<JsonNode> { Rc::new(JsonNode::Array(JsonArray::new(elements))) }

    pub fn arrayOf(elements: &[Rc<JsonNode>]) -> Rc<JsonNode> { Self::array(elements.to_vec()) }

    pub fn object(fields: &JsonFields) -> Rc<JsonNode> { Rc::new(JsonNode::Object(JsonObject::new(fields))) }
}
