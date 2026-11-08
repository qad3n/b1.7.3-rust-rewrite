use crate::io::{InputStream, IoResult, OutputStream};
use crate::string::JString;

struct HashtableEntry
{
    m_hash: i32,
    m_key: String,
    m_value: String,
}

struct Hashtable
{
    m_table: Vec<Vec<HashtableEntry>>,
    m_count: i32,
    m_threshold: i32,
}

impl Default for Hashtable
{
    fn default() -> Self { Hashtable { m_table: (0..11).map(|_| Vec::new()).collect(), m_count: 0, m_threshold: 8 } }
}

impl Hashtable
{
    fn index(hash: i32, length: usize) -> usize { ((hash & 0x7FFFFFFF) % length as i32) as usize }

    fn get(&self, key: &str) -> Option<&String>
    {
        let hash: i32 = JString::from(key).hashCode();
        let bucket: &Vec<HashtableEntry> = &self.m_table[Self::index(hash, self.m_table.len())];
        bucket.iter().find(|entry| entry.m_hash == hash && entry.m_key == key).map(|entry| &entry.m_value)
    }

    fn rehash(&mut self)
    {
        let oldCapacity: usize = self.m_table.len();
        let newCapacity: usize = oldCapacity * 2 + 1;
        let mut newMap: Vec<Vec<HashtableEntry>> = (0..newCapacity).map(|_| Vec::new()).collect();
        self.m_threshold = (newCapacity as f32 * 0.75) as i32;
        for i in (0..oldCapacity).rev()
        {
            for entry in std::mem::take(&mut self.m_table[i])
            {
                let index: usize = Self::index(entry.m_hash, newCapacity);
                newMap[index].insert(0, entry);
            }
        }
        self.m_table = newMap;
    }

    fn put(&mut self, key: String, value: String)
    {
        let hash: i32 = JString::from(key.as_str()).hashCode();
        let mut index: usize = Self::index(hash, self.m_table.len());
        if let Some(entry) = self.m_table[index].iter_mut().find(|entry| entry.m_hash == hash && entry.m_key == key)
        {
            entry.m_value = value;
            return;
        }
        if self.m_count >= self.m_threshold
        {
            self.rehash();
            index = Self::index(hash, self.m_table.len());
        }
        self.m_table[index].insert(0, HashtableEntry { m_hash: hash, m_key: key, m_value: value });
        self.m_count += 1;
    }

    fn entries(&self) -> Vec<(&String, &String)>
    {
        self.m_table.iter().rev().flat_map(|bucket| bucket.iter().map(|entry| (&entry.m_key, &entry.m_value))).collect()
    }
}

// java.util.Properties, keys and values decoded the way Properties.load(InputStream) does in java 6
#[derive(Default)]
pub struct Properties
{
    m_table: Hashtable,
}

fn saveConvert(theString: &str, escapeSpace: bool) -> String
{
    let mut outBuffer: String = String::new();
    for (x, aChar) in JString::from(theString).chars().iter().map(|&unit| unit as u32).enumerate()
    {
        if aChar > 61 && aChar < 127
        {
            if aChar == '\\' as u32
            {
                outBuffer.push_str("\\\\");
                continue;
            }
            outBuffer.push(char::from_u32(aChar).unwrap_or('?'));
            continue;
        }
        match aChar
        {
            0x20 =>
            {
                if x == 0 || escapeSpace
                {
                    outBuffer.push('\\');
                }
                outBuffer.push(' ');
            }
            0x09 => outBuffer.push_str("\\t"),
            0x0A => outBuffer.push_str("\\n"),
            0x0D => outBuffer.push_str("\\r"),
            0x0C => outBuffer.push_str("\\f"),
            0x3D | 0x3A | 0x23 | 0x21 =>
            {
                outBuffer.push('\\');
                outBuffer.push(char::from_u32(aChar).unwrap_or('?'));
            }
            _ if aChar < 0x0020 || aChar > 0x007E => outBuffer.push_str(&format!("\\u{aChar:04X}")),
            _ => outBuffer.push(char::from_u32(aChar).unwrap_or('?')),
        }
    }
    outBuffer
}

fn dateToString(millis: i64) -> String
{
    const DAYS: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
    const MONTHS: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
    let fields: crate::calendar::DateFields = crate::calendar::dateFields(millis);
    let offsetMinutes: i64 = fields.m_offsetMillis / 60_000;
    let zone: String = if offsetMinutes == 0
    {
        String::from("GMT")
    }
    else
    {
        let sign: char = if offsetMinutes < 0 { '-' } else { '+' };
        format!("GMT{sign}{:02}:{:02}", offsetMinutes.abs() / 60, offsetMinutes.abs() % 60)
    };
    format!(
        "{} {} {:02} {:02}:{:02}:{:02} {zone} {}",
        DAYS[fields.m_dayOfWeek as usize],
        MONTHS[(fields.m_month - 1) as usize],
        fields.m_day,
        fields.m_hour,
        fields.m_minute,
        fields.m_second,
        fields.m_year
    )
}

fn isWhitespace(ch: char) -> bool { ch == ' ' || ch == '\t' || ch == '\u{c}' }

impl Properties
{
    pub fn new() -> Properties { Properties::default() }

    pub fn getProperty(&self, key: &str, defaultValue: &str) -> String
    {
        self.m_table.get(key).cloned().unwrap_or_else(|| defaultValue.to_string())
    }

    pub fn getPropertyOrNull(&self, key: &str) -> Option<String> { self.m_table.get(key).cloned() }

    pub fn containsKey(&self, key: &str) -> bool { self.m_table.get(key).is_some() }

    pub fn setProperty(&mut self, key: &str, value: &str) { self.m_table.put(key.to_string(), value.to_string()); }

    pub fn store<S: OutputStream>(&self, output: &mut S, comments: &str) -> IoResult<()>
    {
        let newLine: &str = if cfg!(windows) { "\r\n" } else { "\n" };
        let mut text: String = format!("#{comments}{newLine}");
        text.push_str(&format!("#{}{newLine}", dateToString(crate::system::currentTimeMillis())));
        for (key, value) in self.m_table.entries()
        {
            text.push_str(&format!("{}={}{newLine}", saveConvert(key, true), saveConvert(value, false)));
        }
        let bytes: Vec<u8> = text.chars().map(|ch| ch as u32 as u8).collect();
        output.writeFrom(&bytes, 0, bytes.len() as i32)?;
        output.flush()
    }

    pub fn load<S: InputStream>(&mut self, input: &mut S) -> IoResult<()>
    {
        let mut bytes: Vec<u8> = Vec::new();
        let mut buffer: [u8; 8192] = [0; 8192];
        loop
        {
            let count: i32 = input.readInto(&mut buffer, 0, 8192)?;
            if count < 0
            {
                break;
            }
            bytes.extend_from_slice(&buffer[..count as usize]);
        }
        // the stream is read as ISO-8859-1, one char per byte
        let text: Vec<char> = bytes.iter().map(|&byte| byte as char).collect();
        for line in logicalLines(&text)
        {
            self.loadLine(&line)?;
        }
        Ok(())
    }

    fn loadLine(&mut self, line: &[char]) -> IoResult<()>
    {
        let limit: usize = line.len();
        let mut keyLen: usize = 0;
        let mut valueStart: usize = limit;
        let mut hasSep: bool = false;
        let mut precedingBackslash: bool = false;
        while keyLen < limit
        {
            let c: char = line[keyLen];
            if (c == '=' || c == ':') && !precedingBackslash
            {
                valueStart = keyLen + 1;
                hasSep = true;
                break;
            }
            else if isWhitespace(c) && !precedingBackslash
            {
                valueStart = keyLen + 1;
                break;
            }
            if c == '\\'
            {
                precedingBackslash = !precedingBackslash;
            }
            else
            {
                precedingBackslash = false;
            }
            keyLen += 1;
        }
        while valueStart < limit
        {
            let c: char = line[valueStart];
            if !isWhitespace(c)
            {
                if !hasSep && (c == '=' || c == ':')
                {
                    hasSep = true;
                }
                else
                {
                    break;
                }
            }
            valueStart += 1;
        }
        let key: String = loadConvert(&line[..keyLen])?;
        let value: String = loadConvert(&line[valueStart.min(limit)..])?;
        self.m_table.put(key, value);
        Ok(())
    }
}

// splits into logical lines like Properties.LineReader, dropping comments, blank lines and leading whitespace, joining continuations
fn logicalLines(text: &[char]) -> Vec<Vec<char>>
{
    let mut lines: Vec<Vec<char>> = Vec::new();
    let mut current: Vec<char> = Vec::new();
    let mut skipWhiteSpace: bool = true;
    let mut isCommentLine: bool = false;
    let mut isNewLine: bool = true;
    let mut appendedLineBegin: bool = false;
    let mut precedingBackslash: bool = false;
    let mut skipLF: bool = false;
    let mut index: usize = 0;
    while index < text.len()
    {
        let c: char = text[index];
        index += 1;
        if skipLF
        {
            skipLF = false;
            if c == '\n'
            {
                continue;
            }
        }
        if skipWhiteSpace
        {
            if c == ' ' || c == '\t' || c == '\u{c}'
            {
                continue;
            }
            if !appendedLineBegin && (c == '\r' || c == '\n')
            {
                continue;
            }
            skipWhiteSpace = false;
            appendedLineBegin = false;
        }
        if isNewLine
        {
            isNewLine = false;
            if c == '#' || c == '!'
            {
                isCommentLine = true;
                continue;
            }
        }
        if c != '\n' && c != '\r'
        {
            current.push(c);
            if c == '\\'
            {
                precedingBackslash = !precedingBackslash;
            }
            else
            {
                precedingBackslash = false;
            }
        }
        else
        {
            if isCommentLine || current.is_empty()
            {
                isCommentLine = false;
                isNewLine = true;
                skipWhiteSpace = true;
                current.clear();
                continue;
            }
            if precedingBackslash
            {
                current.pop();
                skipWhiteSpace = true;
                appendedLineBegin = true;
                precedingBackslash = false;
                if c == '\r'
                {
                    skipLF = true;
                }
            }
            else
            {
                lines.push(std::mem::take(&mut current));
                isNewLine = true;
                skipWhiteSpace = true;
            }
        }
    }
    if !isCommentLine && !current.is_empty()
    {
        if precedingBackslash
        {
            current.pop();
        }
        lines.push(current);
    }
    lines
}

fn loadConvert(chars: &[char]) -> IoResult<String>
{
    let mut out: Vec<u16> = Vec::with_capacity(chars.len());
    let mut index: usize = 0;
    while index < chars.len()
    {
        let mut c: char = chars[index];
        index += 1;
        if c == '\\'
        {
            c = chars.get(index).copied().unwrap_or('\0');
            index += 1;
            if c == 'u'
            {
                let mut value: u32 = 0;
                for _ in 0..4
                {
                    let digit: char = chars.get(index).copied().unwrap_or('\0');
                    index += 1;
                    match digit.to_digit(16)
                    {
                        Some(nibble) => value = (value << 4) + nibble,
                        None =>
                        {
                            crate::throwable::throw(&crate::throwable::ILLEGAL_ARGUMENT, "Malformed \\uxxxx encoding.")
                        }
                    }
                }
                out.push(value as u16);
                continue;
            }
            c = match c
            {
                't' => '\t',
                'r' => '\r',
                'n' => '\n',
                'f' => '\u{c}',
                other => other,
            };
        }
        out.push(c as u16);
    }
    Ok(String::from_utf16_lossy(&out))
}
