// java.lang.ConditionalSpecialCasing of jdk 6
use crate::character;

const FINAL_CASED: i32 = 1;
const AFTER_SOFT_DOTTED: i32 = 2;
const MORE_ABOVE: i32 = 3;
const AFTER_I: i32 = 4;
const NOT_BEFORE_DOT: i32 = 5;
const COMBINING_CLASS_ABOVE: i32 = 230;
const COMBINING_DOT_ABOVE: i32 = 0x0307;

struct Entry
{
    m_codePoint: i32,
    m_lower: &'static [u16],
    m_upper: &'static [u16],
    m_language: Option<&'static str>,
    m_condition: i32,
}

const ENTRIES: &[Entry] = &[
    Entry { m_codePoint: 0x03A3, m_lower: &[0x03C2], m_upper: &[0x03A3], m_language: None, m_condition: FINAL_CASED },
    Entry {
        m_codePoint: 0x0307,
        m_lower: &[0x0307],
        m_upper: &[],
        m_language: Some("lt"),
        m_condition: AFTER_SOFT_DOTTED,
    },
    Entry {
        m_codePoint: 0x0049,
        m_lower: &[0x0069, 0x0307],
        m_upper: &[0x0049],
        m_language: Some("lt"),
        m_condition: MORE_ABOVE,
    },
    Entry {
        m_codePoint: 0x004A,
        m_lower: &[0x006A, 0x0307],
        m_upper: &[0x004A],
        m_language: Some("lt"),
        m_condition: MORE_ABOVE,
    },
    Entry {
        m_codePoint: 0x012E,
        m_lower: &[0x012F, 0x0307],
        m_upper: &[0x012E],
        m_language: Some("lt"),
        m_condition: MORE_ABOVE,
    },
    Entry {
        m_codePoint: 0x00CC,
        m_lower: &[0x0069, 0x0307, 0x0300],
        m_upper: &[0x00CC],
        m_language: Some("lt"),
        m_condition: 0,
    },
    Entry {
        m_codePoint: 0x00CD,
        m_lower: &[0x0069, 0x0307, 0x0301],
        m_upper: &[0x00CD],
        m_language: Some("lt"),
        m_condition: 0,
    },
    Entry {
        m_codePoint: 0x0128,
        m_lower: &[0x0069, 0x0307, 0x0303],
        m_upper: &[0x0128],
        m_language: Some("lt"),
        m_condition: 0,
    },
    Entry { m_codePoint: 0x0307, m_lower: &[], m_upper: &[0x0307], m_language: Some("tr"), m_condition: AFTER_I },
    Entry { m_codePoint: 0x0307, m_lower: &[], m_upper: &[0x0307], m_language: Some("az"), m_condition: AFTER_I },
    Entry {
        m_codePoint: 0x0049,
        m_lower: &[0x0131],
        m_upper: &[0x0049],
        m_language: Some("tr"),
        m_condition: NOT_BEFORE_DOT,
    },
    Entry {
        m_codePoint: 0x0049,
        m_lower: &[0x0131],
        m_upper: &[0x0049],
        m_language: Some("az"),
        m_condition: NOT_BEFORE_DOT,
    },
    Entry { m_codePoint: 0x0069, m_lower: &[0x0069], m_upper: &[0x0130], m_language: Some("tr"), m_condition: 0 },
    Entry { m_codePoint: 0x0069, m_lower: &[0x0069], m_upper: &[0x0130], m_language: Some("az"), m_condition: 0 },
];

const SOFT_DOTTED: &[i32] = &[0x0069, 0x006A, 0x012F, 0x0268, 0x0456, 0x0458, 0x1D62, 0x1E2D, 0x1ECB, 0x2071];

// java's other cased characters beside the Lu, Ll and Lt letters
const OTHER_CASED: &[(i32, i32)] = &[
    (0x02B0, 0x02B8),
    (0x02C0, 0x02C1),
    (0x02E0, 0x02E4),
    (0x0345, 0x0345),
    (0x037A, 0x037A),
    (0x1D2C, 0x1D61),
    (0x2160, 0x217F),
    (0x24B6, 0x24E9),
];

pub(crate) fn toLowerCaseEx(src: &[u16], index: usize, language: &str) -> i32
{
    match lookUpTable(src, index, language, true)
    {
        Some(result) if result.len() == 1 => result[0] as i32,
        Some(_) => character::ERROR,
        None => character::toLowerCase(character::codePointAt(src, index)),
    }
}

pub(crate) fn toUpperCaseEx(src: &[u16], index: usize, language: &str) -> i32
{
    match lookUpTable(src, index, language, false)
    {
        Some(result) if result.len() == 1 => result[0] as i32,
        Some(_) => character::ERROR,
        None => character::toUpperCaseEx(character::codePointAt(src, index)),
    }
}

// only reached after toLowerCaseEx returned ERROR, so the table always has the entry
pub(crate) fn toLowerCaseCharArray(src: &[u16], index: usize, language: &str) -> Vec<u16>
{
    lookUpTable(src, index, language, true).map(<[u16]>::to_vec).unwrap_or_default()
}

pub(crate) fn toUpperCaseCharArray(src: &[u16], index: usize, language: &str) -> Vec<u16>
{
    match lookUpTable(src, index, language, false)
    {
        Some(result) => result.to_vec(),
        None => character::toUpperCaseCharArray(character::codePointAt(src, index)),
    }
}

// java walks a HashSet per code point, at most one entry per language can match so its order never shows
fn lookUpTable(src: &[u16], index: usize, language: &str, lowerCasing: bool) -> Option<&'static [u16]>
{
    let codePoint: i32 = character::codePointAt(src, index);
    ENTRIES
        .iter()
        .filter(|entry| entry.m_codePoint == codePoint)
        .find(|entry| {
            entry.m_language.is_none_or(|entryLanguage| entryLanguage == language)
                && isConditionMet(src, index, entry.m_condition)
        })
        .map(|entry| {
            if lowerCasing
            {
                entry.m_lower
            }
            else
            {
                entry.m_upper
            }
        })
}

fn isConditionMet(src: &[u16], index: usize, condition: i32) -> bool
{
    match condition
    {
        FINAL_CASED => isFinalCased(src, index),
        AFTER_SOFT_DOTTED => isAfter(src, index, |ch| SOFT_DOTTED.contains(&ch)),
        MORE_ABOVE => isMoreAbove(src, index),
        AFTER_I => isAfter(src, index, |ch| ch == i32::from(b'I')),
        NOT_BEFORE_DOT => !isBeforeDot(src, index),
        _ => true,
    }
}

// BreakIterator.getWordInstance boundaries reduced to runs of letters, digits and marks; java's MidLetter and
// MidNum rules that keep apostrophes and periods inside a word are not reproduced
fn isWordBoundary(src: &[u16], index: usize) -> bool
{
    if index == 0 || index >= src.len()
    {
        return true;
    }
    !(character::isWordCharacter(character::codePointBefore(src, index))
        && character::isWordCharacter(character::codePointAt(src, index)))
}

fn isFinalCased(src: &[u16], index: usize) -> bool
{
    let mut i: usize = index;
    while !isWordBoundary(src, i)
    {
        let ch: i32 = character::codePointBefore(src, i);
        if isCased(ch)
        {
            let mut j: usize = index + character::charCount(character::codePointAt(src, index));
            while j < src.len() && !isWordBoundary(src, j)
            {
                let next: i32 = character::codePointAt(src, j);
                if isCased(next)
                {
                    return false;
                }
                j += character::charCount(next);
            }
            return true;
        }
        i -= character::charCount(ch);
    }
    false
}

// isAfterI and isAfterSoftDotted, a match before any base or above combining character
fn isAfter(src: &[u16], index: usize, matches: impl Fn(i32) -> bool) -> bool
{
    let mut i: usize = index;
    while i > 0
    {
        let ch: i32 = character::codePointBefore(src, i);
        if matches(ch)
        {
            return true;
        }
        let combiningClass: i32 = character::getCombiningClass(ch);
        if combiningClass == 0 || combiningClass == COMBINING_CLASS_ABOVE
        {
            return false;
        }
        i -= character::charCount(ch);
    }
    false
}

fn isMoreAbove(src: &[u16], index: usize) -> bool
{
    let mut i: usize = index + character::charCount(character::codePointAt(src, index));
    while i < src.len()
    {
        let ch: i32 = character::codePointAt(src, i);
        match character::getCombiningClass(ch)
        {
            COMBINING_CLASS_ABOVE => return true,
            0 => return false,
            _ => i += character::charCount(ch),
        }
    }
    false
}

fn isBeforeDot(src: &[u16], index: usize) -> bool
{
    let mut i: usize = index + character::charCount(character::codePointAt(src, index));
    while i < src.len()
    {
        let ch: i32 = character::codePointAt(src, i);
        if ch == COMBINING_DOT_ABOVE
        {
            return true;
        }
        let combiningClass: i32 = character::getCombiningClass(ch);
        if combiningClass == 0 || combiningClass == COMBINING_CLASS_ABOVE
        {
            return false;
        }
        i += character::charCount(ch);
    }
    false
}

fn isCased(ch: i32) -> bool
{
    character::isCasedLetter(ch) || OTHER_CASED.iter().any(|&(start, end)| (start..=end).contains(&ch))
}
