use jvm::string::JString;
use jvm::throwable;

pub struct JsonNumberNode
{
    m_value: JString,
}

fn digits(chars: &[u16], mut index: usize) -> usize
{
    while index < chars.len() && (0x30..=0x39).contains(&chars[index])
    {
        index += 1;
    }
    index
}

fn matchesNumber(chars: &[u16]) -> bool
{
    let mut index: usize = 0;
    if chars.get(index) == Some(&0x2D)
    {
        index += 1;
    }
    match chars.get(index)
    {
        Some(0x30) => index += 1,
        Some(0x31..=0x39) => index = digits(chars, index + 1),
        _ => return false,
    }
    if chars.get(index) == Some(&0x2E)
    {
        let end: usize = digits(chars, index + 1);
        if end == index + 1
        {
            return false;
        }
        index = end;
    }
    if matches!(chars.get(index), Some(0x65 | 0x45))
    {
        index += 1;
        if matches!(chars.get(index), Some(0x2B | 0x2D))
        {
            index += 1;
        }
        let end: usize = digits(chars, index);
        if end == index
        {
            return false;
        }
        index = end;
    }
    index == chars.len()
}

impl JsonNumberNode
{
    pub fn new(value: JString) -> JsonNumberNode
    {
        if !matchesNumber(value.chars())
        {
            throwable::throw(
                &throwable::ILLEGAL_ARGUMENT,
                &format!(
                    "Attempt to construct a JsonNumber with a String [{value}] that does not match the JSON number specification."
                ),
            );
        }
        JsonNumberNode { m_value: value }
    }

    pub fn getText(&self) -> JString { self.m_value.clone() }

    pub fn toString(&self) -> String { format!("JsonNumberNode value:[{}]", self.m_value) }
}

impl JsonNumberNode
{
    pub fn equals(&self, other: &JsonNumberNode) -> bool { self.m_value == other.m_value }

    pub fn hashCode(&self) -> i32 { self.m_value.hashCode() }
}
