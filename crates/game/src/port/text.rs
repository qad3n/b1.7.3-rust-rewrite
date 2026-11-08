use jvm::locale::Charset;
use jvm::string::JString;

pub fn readLines(text: &str) -> Vec<&str>
{
    let mut lines: Vec<&str> = Vec::new();
    let mut start: usize = 0;
    let bytes: &[u8] = text.as_bytes();
    let mut index: usize = 0;
    while index < bytes.len()
    {
        match bytes[index]
        {
            b'\n' | b'\r' =>
            {
                lines.push(&text[start..index]);
                if bytes[index] == b'\r' && bytes.get(index + 1) == Some(&b'\n')
                {
                    index += 1;
                }
                start = index + 1;
            }
            _ =>
            {}
        }
        index += 1;
    }
    if start < bytes.len()
    {
        lines.push(&text[start..]);
    }
    lines
}

pub fn split(line: &str, separator: char) -> Vec<&str>
{
    if !line.contains(separator)
    {
        return vec![line];
    }
    let mut parts: Vec<&str> = line.split(separator).collect();
    while parts.last() == Some(&"")
    {
        parts.pop();
    }
    parts
}

pub fn splitUnits(text: &[u16], separator: u16) -> Vec<Vec<u16>>
{
    if text.is_empty()
    {
        return vec![Vec::new()];
    }
    let mut parts: Vec<Vec<u16>> = text.split(|&unit| unit == separator).map(<[u16]>::to_vec).collect();
    while parts.last().is_some_and(Vec::is_empty)
    {
        parts.pop();
    }
    parts
}

pub fn trim(text: &JString) -> JString
{
    let chars: &[u16] = text.chars();
    let start: usize = chars.iter().position(|&c| c > 0x20).unwrap_or(chars.len());
    let end: usize = chars.iter().rposition(|&c| c > 0x20).map_or(start, |index| index + 1);
    JString::fromUtf16(chars[start..end].to_vec())
}

pub fn appendChar(text: &JString, chr: u16) -> JString
{
    let mut chars: Vec<u16> = text.chars().to_vec();
    chars.push(chr);
    JString::fromUtf16(chars)
}

pub fn replaceChar(text: &JString, from: u16, to: u16) -> JString
{
    JString::fromUtf16(text.chars().iter().map(|&unit| if unit == from { to } else { unit }).collect())
}

pub fn trimStr(text: &str) -> &str { text.trim_matches(|c: char| c <= ' ') }

pub fn decodePlatform(bytes: &[u8]) -> String
{
    match jvm::locale::platformCharset()
    {
        Charset::Utf8 => String::from_utf8_lossy(bytes).into_owned(),
        Charset::Latin1 => bytes.iter().map(|&byte| byte as char).collect(),
        Charset::Ascii => bytes
            .iter()
            .map(|&byte| {
                if byte < 0x80
                {
                    byte as char
                }
                else
                {
                    '\u{FFFD}'
                }
            })
            .collect(),
    }
}
