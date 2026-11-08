// java.lang.Character of jdk 6 over its unicode 4.0 data, code points as i32 like java
use crate::unicode_data::{
    CASED_LETTERS, COMBINING_CLASSES, DECIMAL_DIGITS, DIGIT_VALUES, LOWER_CASE, UPPER_CASE, UPPER_CASE_EXPANSIONS,
    WORD_CHARACTERS,
};

pub const MIN_RADIX: i32 = 2;
pub const MAX_RADIX: i32 = 36;
pub const ERROR: i32 = -1;
pub const MIN_SUPPLEMENTARY_CODE_POINT: i32 = 0x10000;
const MIN_HIGH_SURROGATE: u16 = 0xD800;
const MAX_HIGH_SURROGATE: u16 = 0xDBFF;
const MIN_LOW_SURROGATE: u16 = 0xDC00;
const MAX_LOW_SURROGATE: u16 = 0xDFFF;

fn inRanges(ranges: &[(u32, u32)], codePoint: i32) -> bool
{
    let point: u32 = codePoint as u32;
    ranges.binary_search_by(|&(start, end)| order(start, end, point)).is_ok()
}

fn order(start: u32, end: u32, point: u32) -> std::cmp::Ordering
{
    if end < point
    {
        std::cmp::Ordering::Less
    }
    else if start > point
    {
        std::cmp::Ordering::Greater
    }
    else
    {
        std::cmp::Ordering::Equal
    }
}

fn mapped(table: &[(u32, u32, u32, i32)], codePoint: i32) -> i32
{
    let point: u32 = codePoint as u32;
    match table.binary_search_by(|&(start, end, _, _)| order(start, end, point))
    {
        Ok(index) =>
        {
            let (start, _, step, delta) = table[index];
            if (point - start).is_multiple_of(step)
            {
                codePoint + delta
            }
            else
            {
                codePoint
            }
        }
        Err(_) => codePoint,
    }
}

pub fn isHighSurrogate(ch: u16) -> bool { (MIN_HIGH_SURROGATE..=MAX_HIGH_SURROGATE).contains(&ch) }

pub fn isLowSurrogate(ch: u16) -> bool { (MIN_LOW_SURROGATE..=MAX_LOW_SURROGATE).contains(&ch) }

pub fn charCount(codePoint: i32) -> usize
{
    if codePoint >= MIN_SUPPLEMENTARY_CODE_POINT
    {
        2
    }
    else
    {
        1
    }
}

pub fn toChars(codePoint: i32) -> Vec<u16>
{
    if codePoint < MIN_SUPPLEMENTARY_CODE_POINT
    {
        return vec![codePoint as u16];
    }
    let offset: i32 = codePoint - MIN_SUPPLEMENTARY_CODE_POINT;
    vec![MIN_HIGH_SURROGATE + (offset >> 10) as u16, MIN_LOW_SURROGATE + (offset & 0x3FF) as u16]
}

// String.codePointAt, a lone surrogate is its own code point
pub fn codePointAt(chars: &[u16], index: usize) -> i32
{
    let high: u16 = chars[index];
    if isHighSurrogate(high) && index + 1 < chars.len() && isLowSurrogate(chars[index + 1])
    {
        let low: u16 = chars[index + 1];
        return (((high - MIN_HIGH_SURROGATE) as i32) << 10)
            + (low - MIN_LOW_SURROGATE) as i32
            + MIN_SUPPLEMENTARY_CODE_POINT;
    }
    high as i32
}

// String.codePointBefore
pub fn codePointBefore(chars: &[u16], index: usize) -> i32
{
    let low: u16 = chars[index - 1];
    if isLowSurrogate(low) && index >= 2 && isHighSurrogate(chars[index - 2])
    {
        return codePointAt(chars, index - 2);
    }
    low as i32
}

pub fn digit(codePoint: i32, radix: i32) -> i32
{
    if !(MIN_RADIX..=MAX_RADIX).contains(&radix)
    {
        return -1;
    }
    let point: u32 = codePoint as u32;
    let value: i32 = match DIGIT_VALUES.binary_search_by(|&(start, end, _)| order(start, end, point))
    {
        Ok(index) => DIGIT_VALUES[index].2 as i32 + (point - DIGIT_VALUES[index].0) as i32,
        Err(_) => -1,
    };
    if value < radix { value } else { -1 }
}

pub fn isDigit(codePoint: i32) -> bool { inRanges(DECIMAL_DIGITS, codePoint) }

pub fn toLowerCase(codePoint: i32) -> i32 { mapped(LOWER_CASE, codePoint) }

pub fn toUpperCase(codePoint: i32) -> i32 { mapped(UPPER_CASE, codePoint) }

// Character.toUpperCaseEx, ERROR when the upper case form has more than one char
pub fn toUpperCaseEx(codePoint: i32) -> i32
{
    if UPPER_CASE_EXPANSIONS.binary_search_by_key(&(codePoint as u32), |&(point, _)| point).is_ok()
    {
        return ERROR;
    }
    mapped(UPPER_CASE, codePoint)
}

pub fn toUpperCaseCharArray(codePoint: i32) -> Vec<u16>
{
    match UPPER_CASE_EXPANSIONS.binary_search_by_key(&(codePoint as u32), |&(point, _)| point)
    {
        Ok(index) => UPPER_CASE_EXPANSIONS[index].1.to_vec(),
        Err(_) => toChars(toUpperCaseEx(codePoint)),
    }
}

// letters of category Lu, Ll or Lt
pub(crate) fn isCasedLetter(codePoint: i32) -> bool { inRanges(CASED_LETTERS, codePoint) }

// letters, digits and marks, the characters a word break iterator keeps together
pub(crate) fn isWordCharacter(codePoint: i32) -> bool { inRanges(WORD_CHARACTERS, codePoint) }

// sun.text.Normalizer.getCombiningClass
pub(crate) fn getCombiningClass(codePoint: i32) -> i32
{
    let point: u32 = codePoint as u32;
    match COMBINING_CLASSES.binary_search_by(|&(start, end, _)| order(start, end, point))
    {
        Ok(index) => COMBINING_CLASSES[index].2 as i32,
        Err(_) => 0,
    }
}
