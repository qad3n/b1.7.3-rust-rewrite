// String.format for the conversions the game's lang files use, %s, %d, %% and %n with optional explicit argument indexes
pub fn format(pattern: &str, args: &[String]) -> String
{
    let chars: Vec<char> = pattern.chars().collect();
    let mut out: String = String::new();
    let mut ordinary: usize = 0;
    let mut index: usize = 0;
    while index < chars.len()
    {
        if chars[index] != '%'
        {
            out.push(chars[index]);
            index += 1;
            continue;
        }
        let start: usize = index;
        index += 1;
        let mut digits: String = String::new();
        while index < chars.len() && chars[index].is_ascii_digit()
        {
            digits.push(chars[index]);
            index += 1;
        }
        let mut explicit: Option<usize> = None;
        if index < chars.len() && chars[index] == '$' && !digits.is_empty()
        {
            explicit = digits.parse::<usize>().ok();
            index += 1;
        }
        let Some(&conversion) = chars.get(index)
        else
        {
            crate::throwable::throw(
                &crate::throwable::ILLEGAL_ARGUMENT,
                &format!("UnknownFormatConversionException: Conversion = '{}'", &pattern[start..]),
            );
        };
        index += 1;
        match conversion
        {
            '%' => out.push('%'),
            'n' => out.push('\n'),
            's' | 'S' | 'd' =>
            {
                let argument: usize = match explicit
                {
                    Some(position) => position - 1,
                    None =>
                    {
                        ordinary += 1;
                        ordinary - 1
                    }
                };
                let Some(value) = args.get(argument)
                else
                {
                    crate::throwable::throw(
                        &crate::throwable::ILLEGAL_ARGUMENT,
                        &format!(
                            "MissingFormatArgumentException: Format specifier '{}'",
                            chars[start..index].iter().collect::<String>()
                        ),
                    );
                };
                if conversion == 'S'
                {
                    out.push_str(&value.to_uppercase());
                }
                else
                {
                    out.push_str(value);
                }
            }
            other => crate::throwable::throw(
                &crate::throwable::ILLEGAL_ARGUMENT,
                &format!("UnknownFormatConversionException: Conversion = '{other}'"),
            ),
        }
    }
    out
}
