use crate::argo::json_object::JsonFields;
use crate::argo::{
    ChainedFunctor, EnumJsonNodeType, JsonNode, JsonNodeFactories, JsonNodeSelector, JsonStringNode, LeafFunctor,
};
use jvm::string::JString;
use jvm::throwable::OrNpe;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub enum PathElement
{
    Index(i32),
    Field(String),
}

impl PathElement
{
    pub fn commaSeparate(pathElements: &[PathElement]) -> String
    {
        let parts: Vec<String> = pathElements
            .iter()
            .map(|element| match element
            {
                PathElement::Index(index) => index.to_string(),
                PathElement::Field(name) => format!("\"{name}\""),
            })
            .collect();
        parts.join(".")
    }
}

type NodeSelector<T> = Rc<JsonNodeSelector<Rc<JsonNode>, T>>;

fn leaf<T: 'static, V: 'static>(
    matchesNode: impl Fn(&T) -> bool + 'static,
    typeSafeApplyTo: impl Fn(&T) -> V + 'static,
    shortForm: String,
    toString: String,
) -> Rc<JsonNodeSelector<T, V>>
{
    Rc::new(JsonNodeSelector::new(Rc::new(LeafFunctor {
        m_matchesNode: Box::new(matchesNode),
        m_typeSafeApplyTo: Box::new(typeSafeApplyTo),
        m_shortForm: shortForm,
        m_toString: toString,
    })))
}

fn nodeOfType<V: 'static>(
    kind: EnumJsonNodeType,
    apply: fn(&Rc<JsonNode>) -> V,
    shortForm: &str,
    toString: &str,
) -> NodeSelector<V>
{
    leaf(move |node: &Rc<JsonNode>| kind == node.getType(), apply, shortForm.to_string(), toString.to_string())
}

pub struct JsonNodeSelectors;

impl JsonNodeSelectors
{
    pub fn aStringNode(pathElements: &[PathElement]) -> NodeSelector<JString>
    {
        Self::chainOn(
            pathElements,
            nodeOfType(
                EnumJsonNodeType::String,
                |node| node.getText(),
                "A short form string",
                "a value that is a string",
            ),
        )
    }

    pub fn anArrayNode(pathElements: &[PathElement]) -> NodeSelector<Vec<Rc<JsonNode>>>
    {
        Self::chainOn(
            pathElements,
            nodeOfType(EnumJsonNodeType::Array, |node| node.getElements(), "A short form array", "an array"),
        )
    }

    pub fn anObjectNode(pathElements: &[PathElement]) -> NodeSelector<JsonFields>
    {
        Self::chainOn(
            pathElements,
            nodeOfType(EnumJsonNodeType::Object, |node| node.getFields(), "A short form object", "an object"),
        )
    }

    pub fn aField(fieldName: &str) -> Rc<JsonNodeSelector<JsonFields, Rc<JsonNode>>>
    {
        Self::aFieldNode(JsonNodeFactories::string(JString::from(fieldName)))
    }

    pub fn aFieldNode(fieldName: JsonStringNode) -> Rc<JsonNodeSelector<JsonFields, Rc<JsonNode>>>
    {
        let text: JString = fieldName.getText();
        let key: JsonStringNode = fieldName.clone();
        leaf(
            move |map: &JsonFields| map.containsKey(&fieldName),
            move |map: &JsonFields| map.get(&key).orNpe(),
            format!("\"{text}\""),
            format!("a field called [\"{text}\"]"),
        )
    }

    pub fn anObjectNodeWithField(fieldName: &str) -> NodeSelector<Rc<JsonNode>>
    {
        Self::anObjectNode(&[]).with(&Self::aField(fieldName))
    }

    pub fn anElement(index: i32) -> Rc<JsonNodeSelector<Vec<Rc<JsonNode>>, Rc<JsonNode>>>
    {
        leaf(
            move |list: &Vec<Rc<JsonNode>>| list.len() as i32 > index,
            move |list: &Vec<Rc<JsonNode>>| Rc::clone(&list[index as usize]),
            index.to_string(),
            format!("an element at index [{index}]"),
        )
    }

    pub fn anArrayNodeWithElement(index: i32) -> NodeSelector<Rc<JsonNode>>
    {
        Self::anArrayNode(&[]).with(&Self::anElement(index))
    }

    fn chainOn<T: 'static>(pathElements: &[PathElement], parentSelector: NodeSelector<T>) -> NodeSelector<T>
    {
        let mut selector: NodeSelector<T> = parentSelector;
        for element in pathElements.iter().rev()
        {
            let parent: NodeSelector<Rc<JsonNode>> = match element
            {
                PathElement::Index(index) => Self::anArrayNodeWithElement(*index),
                PathElement::Field(name) => Self::anObjectNodeWithField(name),
            };
            selector = Rc::new(JsonNodeSelector::new(Rc::new(ChainedFunctor::new(parent, selector))));
        }
        selector
    }
}
