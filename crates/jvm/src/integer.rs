// Integer.parseInt and Long.parseLong of jdk 6, digits per Character.digit so every unicode 4.0 decimal digit counts
use crate::character;
use crate::string::JString;
use crate::throwable::{NUMBER_FORMAT, Throwable};

fn forInputString(text: &JString) -> Throwable
{
    Throwable::new(&NUMBER_FORMAT, Some(format!("For input string: \"{text}\"")))
}

// the shared body of parseInt and parseLong, which accumulate negatively so the minimum value parses
fn parse(text: &JString, radix: i32, min: i64, max: i64) -> Result<i64, Throwable>
{
    if radix < character::MIN_RADIX
    {
        return Err(Throwable::new(&NUMBER_FORMAT, Some(format!("radix {radix} less than Character.MIN_RADIX"))));
    }
    if radix > character::MAX_RADIX
    {
        return Err(Throwable::new(&NUMBER_FORMAT, Some(format!("radix {radix} greater than Character.MAX_RADIX"))));
    }
    let chars: &[u16] = text.chars();
    if chars.is_empty()
    {
        return Err(forInputString(text));
    }
    let radix: i64 = radix as i64;
    let mut index: usize = 0;
    let negative: bool = chars[0] == u16::from(b'-');
    let limit: i64 = if negative { min } else { -max };
    if negative
    {
        index += 1;
    }
    let multmin: i64 = limit / radix;
    let mut result: i64 = 0;
    if index < chars.len()
    {
        let digit: i32 = character::digit(chars[index] as i32, radix as i32);
        index += 1;
        if digit < 0
        {
            return Err(forInputString(text));
        }
        result = -(digit as i64);
    }
    while index < chars.len()
    {
        let digit: i64 = character::digit(chars[index] as i32, radix as i32) as i64;
        index += 1;
        if digit < 0 || result < multmin
        {
            return Err(forInputString(text));
        }
        result *= radix;
        if result < limit + digit
        {
            return Err(forInputString(text));
        }
        result -= digit;
    }
    if !negative
    {
        return Ok(-result);
    }
    if index > 1
    {
        Ok(result)
    }
    else
    {
        Err(forInputString(text))
    }
}

pub fn parseInt(text: &JString) -> Result<i32, Throwable> { parseIntRadix(text, 10) }

pub fn parseIntRadix(text: &JString, radix: i32) -> Result<i32, Throwable>
{
    parse(text, radix, i32::MIN as i64, i32::MAX as i64).map(|value| value as i32)
}

pub fn parseLong(text: &JString) -> Result<i64, Throwable> { parseLongRadix(text, 10) }

pub fn parseLongRadix(text: &JString, radix: i32) -> Result<i64, Throwable> { parse(text, radix, i64::MIN, i64::MAX) }
