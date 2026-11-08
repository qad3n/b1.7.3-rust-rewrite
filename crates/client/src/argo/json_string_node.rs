use jvm::object::JavaObject;
use jvm::string::JString;

#[derive(Clone)]
pub struct JsonStringNode
{
    m_value: JString,
}

impl JsonStringNode
{
    pub fn new(value: JString) -> JsonStringNode { JsonStringNode { m_value: value } }

    pub fn getText(&self) -> JString { self.m_value.clone() }

    pub fn compareTo(&self, other: &JsonStringNode) -> std::cmp::Ordering { self.m_value.cmp(&other.m_value) }

    pub fn toString(&self) -> String { format!("JsonStringNode value:[{}]", self.m_value) }
}

impl JavaObject for JsonStringNode
{
    fn javaEquals(&self, other: &Self) -> bool { self.m_value == other.m_value }

    fn javaHash(&self) -> i32 { self.m_value.hashCode() }
}
