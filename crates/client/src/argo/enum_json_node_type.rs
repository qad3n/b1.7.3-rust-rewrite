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
            Self::Object => "OBJECT",
            Self::Array => "ARRAY",
            Self::String => "STRING",
            Self::Number => "NUMBER",
            Self::True => "TRUE",
            Self::False => "FALSE",
            Self::Null => "NULL",
        }
    }
}
