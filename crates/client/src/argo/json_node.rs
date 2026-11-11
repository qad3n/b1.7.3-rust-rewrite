use crate::argo::json_object::JsonFields;
use crate::argo::
{
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
    pub fn equals(&self, other: &Self) -> bool
    {
        match (self, other)
        {
            (Self::Object(a), Self::Object(b)) => a.equals(b),
            (Self::Array(a), Self::Array(b)) => a.equals(b),
            (Self::String(a), Self::String(b)) => a.getText() == b.getText(),
            (Self::Number(a), Self::Number(b)) => a.equals(b),
            (Self::Constant(a), Self::Constant(b)) => a == b,
            _ => false,
        }
    }

    pub fn hashCode(&self) -> i32
    {
        match self
        {
            Self::Object(node) => node.hashCode(),
            Self::Array(node) => node.hashCode(),
            Self::String(node) => node.getText().hashCode(),
            Self::Number(node) => node.hashCode(),
            Self::Constant(kind) => *kind as i32,
        }
    }

    pub fn getType(&self) -> EnumJsonNodeType
    {
        match self
        {
            Self::Object(_) => EnumJsonNodeType::Object,
            Self::Array(_) => EnumJsonNodeType::Array,
            Self::String(_) => EnumJsonNodeType::String,
            Self::Number(_) => EnumJsonNodeType::Number,
            Self::Constant(kind) => *kind,
        }
    }

    pub fn getText(&self) -> JString
    {
        match self
        {
            Self::String(node) => node.getText(),
            Self::Number(node) => node.getText(),
            _ => noText(),
        }
    }

    pub fn getFields(&self) -> JsonFields
    {
        match self
        {
            Self::Object(node) => node.getFields(),
            _ => noFields(),
        }
    }

    pub fn getElements(&self) -> Vec<Rc<Self>>
    {
        match self
        {
            Self::Array(node) => node.getElements(),
            _ => noElements(),
        }
    }

    pub fn getStringValue(self: &Rc<Self>, pathElements: &[PathElement]) -> JString
    {
        Self::wrapExceptionsFor(&JsonNodeSelectors::aStringNode(pathElements), self, pathElements)
    }

    pub fn getArrayNode(self: &Rc<Self>, pathElements: &[PathElement]) -> Vec<Rc<Self>>
    {
        Self::wrapExceptionsFor(&JsonNodeSelectors::anArrayNode(pathElements), self, pathElements)
    }

    fn wrapExceptionsFor<T: 'static>(
        value: &JsonNodeSelector<Rc<Self>, T>,
        node: &Rc<Self>,
        pathElements: &[PathElement],
    ) -> T
    {
        match value.getValue(node)
        {
            Ok(result) => result,

            Err(mismatch) =>
            {
                let rootNode: Rc<Self> = JsonNodeFactories::arrayOf(&[Rc::clone(node)]);
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
            Self::Object(node) =>
            {
                let entries: Vec<String> = node
                    .fields()
                    .entries()
                    .iter()
                    .map(|(key, value)| format!("{}={}", key.toString(), value.toString()))
                    .collect();
                format!("JsonObject fields:[{{{}}}]", entries.join(", "))
            }

            Self::Array(node) =>
            {
                let elements: Vec<String> = node.elements().iter().map(|element| element.toString()).collect();
                format!("JsonArray elements:[[{}]]", elements.join(", "))
            }

            Self::String(node) => node.toString(),
            Self::Number(node) => node.toString(),
            Self::Constant(_) => format!("net.minecraft.src.J_JsonConstants@{:x}", self as *const Self as usize),
        }
    }
}
