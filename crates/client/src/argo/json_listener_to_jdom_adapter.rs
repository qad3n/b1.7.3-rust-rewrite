use crate::argo::{
    JsonArrayNodeBuilder, JsonFieldBuilder, JsonListener, JsonNodeBuilder, JsonNodeBuilders, JsonObjectNodeBuilder,
    JsonRootNode, NodeContainer,
};
use jvm::string::JString;
use jvm::throwable::{self, OrNpe};
use std::rc::Rc;

fn codingFailure(what: &str) -> ! { throwable::throw(&throwable::RUNTIME, &format!("Coding failure in Argo:  {what}")) }

struct ArrayContainer(Rc<JsonArrayNodeBuilder>);

impl NodeContainer for ArrayContainer
{
    fn addNode(&self, node: Rc<dyn JsonNodeBuilder>) { self.0.withElement(node); }

    fn addField(&self, _field: Rc<JsonFieldBuilder>) { codingFailure("Attempt to add a field to an array.") }
}

struct ObjectContainer(Rc<JsonObjectNodeBuilder>);

impl NodeContainer for ObjectContainer
{
    fn addNode(&self, _node: Rc<dyn JsonNodeBuilder>) { codingFailure("Attempt to add a node to an object.") }

    fn addField(&self, field: Rc<JsonFieldBuilder>) { self.0.withField(field); }
}

struct FieldContainer(Rc<JsonFieldBuilder>);

impl NodeContainer for FieldContainer
{
    fn addNode(&self, node: Rc<dyn JsonNodeBuilder>) { self.0.withValue(node); }

    fn addField(&self, _field: Rc<JsonFieldBuilder>) { codingFailure("Attempt to add a field to a field.") }
}

#[derive(Default)]
pub struct JsonListenerToJdomAdapter
{
    m_stack: Vec<Rc<dyn NodeContainer>>,
    m_root: Option<Rc<dyn JsonNodeBuilder>>,
}

impl JsonListenerToJdomAdapter
{
    pub fn new() -> JsonListenerToJdomAdapter { JsonListenerToJdomAdapter::default() }

    pub fn getDocument(&self) -> JsonRootNode { self.m_root.as_ref().orNpe().build() }

    fn peek(&self) -> Rc<dyn NodeContainer>
    {
        match self.m_stack.last()
        {
            Some(container) => Rc::clone(container),
            None => throwable::throw(&throwable::RUNTIME, "java.util.EmptyStackException"),
        }
    }

    fn addRootNode(&mut self, node: Rc<dyn JsonNodeBuilder>)
    {
        if self.m_root.is_none()
        {
            self.m_root = Some(node);
        }
        else
        {
            self.addValue(node);
        }
    }

    fn addValue(&self, node: Rc<dyn JsonNodeBuilder>) { self.peek().addNode(node); }
}

impl JsonListener for JsonListenerToJdomAdapter
{
    fn startDocument(&mut self) {}

    fn endDocument(&mut self) {}

    fn startArray(&mut self)
    {
        let builder: Rc<JsonArrayNodeBuilder> = JsonNodeBuilders::anArrayBuilder();
        self.addRootNode(Rc::clone(&builder) as Rc<dyn JsonNodeBuilder>);
        self.m_stack.push(Rc::new(ArrayContainer(builder)));
    }

    fn endArray(&mut self) { self.m_stack.pop(); }

    fn startObject(&mut self)
    {
        let builder: Rc<JsonObjectNodeBuilder> = JsonNodeBuilders::anObjectBuilder();
        self.addRootNode(Rc::clone(&builder) as Rc<dyn JsonNodeBuilder>);
        self.m_stack.push(Rc::new(ObjectContainer(builder)));
    }

    fn endObject(&mut self) { self.m_stack.pop(); }

    fn startField(&mut self, name: JString)
    {
        let field: Rc<JsonFieldBuilder> =
            Rc::new(JsonFieldBuilder::aJsonFieldBuilder().withKey(JsonNodeBuilders::aStringBuilder(name)));
        self.peek().addField(Rc::clone(&field));
        self.m_stack.push(Rc::new(FieldContainer(field)));
    }

    fn endField(&mut self) { self.m_stack.pop(); }

    fn stringValue(&mut self, value: JString) { self.addValue(JsonNodeBuilders::aStringBuilder(value)); }

    fn numberValue(&mut self, value: JString) { self.addValue(JsonNodeBuilders::aNumberBuilder(value)); }

    fn trueValue(&mut self) { self.addValue(JsonNodeBuilders::aTrueBuilder()); }

    fn falseValue(&mut self) { self.addValue(JsonNodeBuilders::aFalseBuilder()); }

    fn nullValue(&mut self) { self.addValue(JsonNodeBuilders::aNullBuilder()); }
}
