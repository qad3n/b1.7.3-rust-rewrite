use jvm::io::InputStream;
use std::cell::OnceCell;

thread_local! {
    static ALLOWED_CHARACTERS: OnceCell<String> = const { OnceCell::new() };
}

pub struct ChatAllowedCharacters;

fn getAllowedCharacters() -> String
{
    let mut s: String = String::new();
    let Some(mut stream) = crate::port::getResourceAsStream("/font.txt")
    else
    {
        return s;
    };
    let mut bytes: Vec<u8> = Vec::new();
    while let Ok(value) = stream.read()
    {
        if value < 0
        {
            break;
        }
        bytes.push(value as u8);
    }
    for s1 in String::from_utf8_lossy(&bytes).split(['\n', '\r'])
    {
        if !s1.starts_with('#')
        {
            s.push_str(s1);
        }
    }
    s
}

impl ChatAllowedCharacters
{
    pub const ALLOWED_CHARACTERS_ARRAY: [char; 15] =
        ['/', '\n', '\r', '\t', '\u{0}', '\u{c}', '`', '?', '*', '\\', '<', '>', '|', '"', ':'];

    pub fn allowedCharacters() -> String
    {
        ALLOWED_CHARACTERS.with(|characters| characters.get_or_init(getAllowedCharacters).clone())
    }
}
