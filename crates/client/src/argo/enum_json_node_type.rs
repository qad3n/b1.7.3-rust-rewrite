#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EnumJsonNodeType
{
    Object,
    Array,
    String,
    Number,
    True,
    False,
    Null,
}

impl EnumJsonNodeType
{
    pub fn name(self) -> &'static str
    {
        match self
        {
            EnumJsonNodeType::Object => "OBJECT",
            EnumJsonNodeType::Array => "ARRAY",
            EnumJsonNodeType::String => "STRING",
            EnumJsonNodeType::Number => "NUMBER",
            EnumJsonNodeType::True => "TRUE",
            EnumJsonNodeType::False => "FALSE",
            EnumJsonNodeType::Null => "NULL",
        }
    }
}
