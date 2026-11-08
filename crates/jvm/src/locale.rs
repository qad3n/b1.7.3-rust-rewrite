// the default locale and platform charset java 6 derives on linux from setlocale(LC_ALL, "") and nl_langinfo
// read here from the environment the way setlocale resolves it, LC_ALL then LC_CTYPE then LANG; a locale that is
// not installed would make setlocale fall back to C, which the environment alone cannot show
use std::sync::OnceLock;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Charset
{
    Ascii,
    Latin1,
    Utf8,
}

struct DefaultLocale
{
    m_language: String,
    m_charset: Charset,
}

fn environmentLocale() -> String
{
    ["LC_ALL", "LC_CTYPE", "LANG"]
        .iter()
        .filter_map(|name| std::env::var(name).ok())
        .find(|value| !value.is_empty())
        .unwrap_or_else(|| "C".to_string())
}

// language[_country][.codeset][@modifier] like java_props_md.c ParseLocale
fn parse(locale: &str) -> DefaultLocale
{
    if locale == "C" || locale == "POSIX"
    {
        return DefaultLocale { m_language: "en".to_string(), m_charset: Charset::Ascii };
    }
    let withoutModifier: &str = locale.split('@').next().unwrap_or(locale);
    let (name, codeset): (&str, Option<&str>) = match withoutModifier.split_once('.')
    {
        Some((name, codeset)) => (name, Some(codeset)),
        None => (withoutModifier, None),
    };
    let language: String = name.split('_').next().unwrap_or(name).to_ascii_lowercase();
    let charset: Charset = match codeset.map(|codeset| codeset.to_ascii_lowercase().replace('-', ""))
    {
        Some(codeset) if codeset == "utf8" => Charset::Utf8,
        Some(codeset) if codeset == "ascii" || codeset == "ansi_x3.41968" => Charset::Ascii,
        // glibc gives most locales without a codeset an iso 8859 single byte charset
        _ => Charset::Latin1,
    };
    DefaultLocale { m_language: language, m_charset: charset }
}

fn defaultLocale() -> &'static DefaultLocale
{
    static DEFAULT: OnceLock<DefaultLocale> = OnceLock::new();
    DEFAULT.get_or_init(|| parse(&environmentLocale()))
}

// Locale.getDefault().getLanguage()
pub fn defaultLanguage() -> &'static str { &defaultLocale().m_language }

// sun.jnu.encoding and file.encoding, the charset file names and default text output use
pub fn platformCharset() -> Charset { defaultLocale().m_charset }
