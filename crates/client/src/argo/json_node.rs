use crate::argo::json_object::JsonFields;
use crate::argo::{
    CompactJsonFormatter, EnumJsonNodeType, JsonArray, JsonFormatter, JsonNodeFactories, JsonNodeSelector,
    JsonNodeSelectors, JsonNumberNode, JsonObject, JsonStringNode, PATH_ELEMENTS_EXCEPTION, PathElement,
};
use jvm::string::JString;
use jvm::throwable;
use std::rc::Rc;

pub enum JsonNode
{
    Object(JsonObject),
    Array(JsonArray),
    String(JsonStringNode),
    Number(JsonNumberNode),
    Constant(EnumJsonNodeType),
}

pub type JsonRootNode = Rc<JsonNode>;

fn noText() -> ! { throwable::throw(&throwable::ILLEGAL_STATE, "Attempt to get text on a JsonNode without text.") }

fn noFields() -> !
{
    throwable::throw(&throwable::ILLEGAL_STATE, "Attempt to get fields on a JsonNode without fields.")
}

fn noElements() -> !
{
    throwable::throw(&throwable::ILLEGAL_STATE, "Attempt to get elements on a JsonNode without elements.")
}

impl JsonNode
{
    pub fn equals(&self, other: &JsonNode) -> bool
    {
        match (self, other)
        {
            (JsonNode::Object(a), JsonNode::Object(b)) => a.equals(b),
            (JsonNode::Array(a), JsonNode::Array(b)) => a.equals(b),
            (JsonNode::String(a), JsonNode::String(b)) => a.getText() == b.getText(),
            (JsonNode::Number(a), JsonNode::Number(b)) => a.equals(b),
            (JsonNode::Constant(a), JsonNode::Constant(b)) => a == b,
            _ => false,
        }
    }

    pub fn hashCode(&self) -> i32
    {
        match self
        {
            JsonNode::Object(node) => node.hashCode(),
            JsonNode::Array(node) => node.hashCode(),
            JsonNode::String(node) => node.getText().hashCode(),
            JsonNode::Number(node) => node.hashCode(),
            JsonNode::Constant(kind) => *kind as i32,
        }
    }

    pub fn getType(&self) -> EnumJsonNodeType
    {
        match self
        {
            JsonNode::Object(_) => EnumJsonNodeType::Object,
            JsonNode::Array(_) => EnumJsonNodeType::Array,
            JsonNode::String(_) => EnumJsonNodeType::String,
            JsonNode::Number(_) => EnumJsonNodeType::Number,
            JsonNode::Constant(kind) => *kind,
        }
    }

    pub fn getText(&self) -> JString
    {
        match self
        {
            JsonNode::String(node) => node.getText(),
            JsonNode::Number(node) => node.getText(),
            _ => noText(),
        }
    }

    pub fn getFields(&self) -> JsonFields
    {
        match self
        {
            JsonNode::Object(node) => node.getFields(),
            _ => noFields(),
        }
    }

    pub fn getElements(&self) -> Vec<Rc<JsonNode>>
    {
        match self
        {
            JsonNode::Array(node) => node.getElements(),
            _ => noElements(),
        }
    }

    pub fn getStringValue(self: &Rc<Self>, pathElements: &[PathElement]) -> JString
    {
        Self::wrapExceptionsFor(&JsonNodeSelectors::aStringNode(pathElements), self, pathElements)
    }

    pub fn getArrayNode(self: &Rc<Self>, pathElements: &[PathElement]) -> Vec<Rc<JsonNode>>
    {
        Self::wrapExceptionsFor(&JsonNodeSelectors::anArrayNode(pathElements), self, pathElements)
    }

    fn wrapExceptionsFor<T: 'static>(
        value: &JsonNodeSelector<Rc<JsonNode>, T>,
        node: &Rc<JsonNode>,
        pathElements: &[PathElement],
    ) -> T
    {
        match value.getValue(node)
        {
            Ok(result) => result,
            Err(mismatch) =>
            {
                let rootNode: Rc<JsonNode> = JsonNodeFactories::arrayOf(&[Rc::clone(node)]);
                let message: String = format!(
                    "Failed to find {} at [{}] while resolving [{}] in {}.",
                    mismatch.m_failedNode.describe(),
                    mismatch.getShortFormFailPath(),
                    PathElement::commaSeparate(pathElements),
                    CompactJsonFormatter.format(&rootNode)
                );
                throwable::throw(&PATH_ELEMENTS_EXCEPTION, &message)
            }
        }
    }

    pub fn toString(&self) -> String
    {
        match self
        {
            JsonNode::Object(node) =>
            {
                let entries: Vec<String> = node
                    .fields()
                    .entries()
                    .iter()
                    .map(|(key, value)| format!("{}={}", key.toString(), value.toString()))
                    .collect();
                format!("JsonObject fields:[{{{}}}]", entries.join(", "))
            }
            JsonNode::Array(node) =>
            {
                let elements: Vec<String> = node.elements().iter().map(|element| element.toString()).collect();
                format!("JsonArray elements:[[{}]]", elements.join(", "))
            }
            JsonNode::String(node) => node.toString(),
            JsonNode::Number(node) => node.toString(),
            JsonNode::Constant(_) => format!("net.minecraft.src.J_JsonConstants@{:x}", self as *const Self as usize),
        }
    }
}
