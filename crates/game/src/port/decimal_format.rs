pub fn formatInteger(value: i32) -> String
{
    let digits: String = (value as i64).abs().to_string();
    let mut grouped: String = String::new();
    for (index, digit) in digits.chars().enumerate()
    {
        if index > 0 && (digits.len() - index).is_multiple_of(3)
        {
            grouped.push(',');
        }
        grouped.push(digit);
    }
    if value < 0
    {
        format!("-{grouped}")
    }
    else
    {
        grouped
    }
}

struct DigitList
{
    m_digits: Vec<u8>,
    m_decimalAt: i32,
}

impl DigitList
{
    fn set(source: f64, maximumDigits: i32) -> DigitList
    {
        let rep: String = jvm::floating_decimal::doubleToString(source);
        let mut digits: Vec<u8> = Vec::new();
        let mut decimalAt: i32 = -1;
        let mut exponent: i32 = 0;
        let mut nonZeroDigitSeen: bool = false;
        let mut leadingZerosAfterDecimal: i32 = 0;
        for (index, c) in rep.bytes().enumerate()
        {
            match c
            {
                b'.' => decimalAt = digits.len() as i32,
                b'E' =>
                {
                    exponent = rep[index + 1..].parse().unwrap_or(0);
                    break;
                }
                _ if digits.len() < 19 =>
                {
                    if !nonZeroDigitSeen
                    {
                        nonZeroDigitSeen = c != b'0';
                        if !nonZeroDigitSeen && decimalAt != -1
                        {
                            leadingZerosAfterDecimal += 1;
                        }
                    }
                    if nonZeroDigitSeen
                    {
                        digits.push(c);
                    }
                }
                _ =>
                {}
            }
        }
        if decimalAt == -1
        {
            decimalAt = digits.len() as i32;
        }
        if nonZeroDigitSeen
        {
            decimalAt += exponent - leadingZerosAfterDecimal;
        }

        let mut list: DigitList = DigitList { m_digits: digits, m_decimalAt: decimalAt };
        if -decimalAt > maximumDigits
        {
            list.m_digits.clear();
            return list;
        }
        if -decimalAt == maximumDigits
        {
            if list.shouldRoundUp(0)
            {
                list.m_digits = vec![b'1'];
                list.m_decimalAt += 1;
            }
            else
            {
                list.m_digits.clear();
            }
            return list;
        }
        list.trimTrailingZeros();
        list.round(maximumDigits + decimalAt);
        list
    }

    fn trimTrailingZeros(&mut self)
    {
        while self.m_digits.len() > 1 && self.m_digits.last() == Some(&b'0')
        {
            self.m_digits.pop();
        }
    }

    fn shouldRoundUp(&self, maximumDigits: i32) -> bool
    {
        let index: usize = maximumDigits as usize;
        let Some(&digit) = self.m_digits.get(index)
        else
        {
            return false;
        };
        if digit > b'5'
        {
            return true;
        }
        if digit == b'5'
        {
            if self.m_digits[index + 1..].iter().any(|&rest| rest != b'0')
            {
                return true;
            }
            return maximumDigits > 0 && !(self.m_digits[index - 1] - b'0').is_multiple_of(2);
        }
        false
    }

    fn round(&mut self, mut maximumDigits: i32)
    {
        if maximumDigits < 0 || maximumDigits >= self.m_digits.len() as i32
        {
            return;
        }
        if self.shouldRoundUp(maximumDigits)
        {
            loop
            {
                maximumDigits -= 1;
                if maximumDigits < 0
                {
                    self.m_digits[0] = b'1';
                    self.m_decimalAt += 1;
                    maximumDigits = 0;
                    break;
                }
                self.m_digits[maximumDigits as usize] += 1;
                if self.m_digits[maximumDigits as usize] <= b'9'
                {
                    break;
                }
            }
            maximumDigits += 1;
        }
        self.m_digits.truncate(maximumDigits as usize);
        self.trimTrailingZeros();
    }
}

pub fn formatTwoPlaces(value: f64) -> String
{
    let fractionDigits: i32 = 2;
    let mut list: DigitList = DigitList::set(value, fractionDigits);
    if list.m_digits.iter().all(|&digit| digit == b'0')
    {
        list.m_decimalAt = 0;
    }

    let mut result: String = String::new();
    let mut digitIndex: usize = 0;
    let count: i32 = list.m_decimalAt.max(1);
    for i in (0..count).rev()
    {
        if i < list.m_decimalAt && digitIndex < list.m_digits.len()
        {
            result.push(list.m_digits[digitIndex] as char);
            digitIndex += 1;
        }
        else
        {
            result.push('0');
        }
    }
    result.push('.');
    for i in 0..fractionDigits
    {
        if -1 - i > list.m_decimalAt - 1
        {
            result.push('0');
            continue;
        }
        if digitIndex < list.m_digits.len()
        {
            result.push(list.m_digits[digitIndex] as char);
            digitIndex += 1;
        }
        else
        {
            result.push('0');
        }
    }
    result
}
