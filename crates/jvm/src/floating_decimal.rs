// sun.misc.FloatingDecimal and FDBigInt of jdk 6u26, decompiled from rt.jar because src.zip leaves sun.* out

use crate::throwable::{self, NUMBER_FORMAT};

const LONG_5_POW: [i64; 27] = [
    1,
    5,
    25,
    125,
    625,
    3125,
    15625,
    78125,
    390625,
    1953125,
    9765625,
    48828125,
    244140625,
    1220703125,
    6103515625,
    30517578125,
    152587890625,
    762939453125,
    3814697265625,
    19073486328125,
    95367431640625,
    476837158203125,
    2384185791015625,
    11920928955078125,
    59604644775390625,
    298023223876953125,
    1490116119384765625,
];
const SMALL_5_POW: [i32; 14] =
    [1, 5, 25, 125, 625, 3125, 15625, 78125, 390625, 1953125, 9765625, 48828125, 244140625, 1220703125];
const N_5_BITS: [i32; 27] =
    [0, 3, 5, 7, 10, 12, 14, 17, 19, 21, 24, 26, 28, 31, 33, 35, 38, 40, 42, 45, 47, 49, 52, 54, 56, 59, 61];
const SMALL_10_POW: [f64; 23] = [
    1.0, 10.0, 100.0, 1000.0, 10000.0, 100000.0, 1000000.0, 1.0E7, 1.0E8, 1.0E9, 1.0E10, 1.0E11, 1.0E12, 1.0E13,
    1.0E14, 1.0E15, 1.0E16, 1.0E17, 1.0E18, 1.0E19, 1.0E20, 1.0E21, 1.0E22,
];
const SINGLE_SMALL_10_POW: [f32; 11] =
    [1.0, 10.0, 100.0, 1000.0, 10000.0, 100000.0, 1000000.0, 1.0E7, 1.0E8, 1.0E9, 1.0E10];
const BIG_10_POW: [f64; 5] = [1.0E16, 1.0E32, 1.0E64, 1.0E128, 1.0E256];
const TINY_10_POW: [f64; 5] = [1.0E-16, 1.0E-32, 1.0E-64, 1.0E-128, 1.0E-256];
const MAX_SMALL_TEN: i32 = SMALL_10_POW.len() as i32 - 1;
const SINGLE_MAX_SMALL_TEN: i32 = SINGLE_SMALL_10_POW.len() as i32 - 1;
const INFINITY: &[u8] = b"Infinity";
const NOT_A_NUMBER: &[u8] = b"NaN";
const ZERO: &[u8] = b"00000000";

mod bits
{
    pub const EXP_MASK: i64 = 0x7FF0000000000000;
    pub const FRACT_MASK: i64 = 0x000FFFFFFFFFFFFF;
    pub const FRACT_HOB: i64 = 0x0010000000000000;
    pub const EXP_ONE: i64 = 0x3FF0000000000000;
    pub const HIGH_BYTE: i64 = 0xFF00000000000000u64 as i64;
    pub const LOW_BYTES: i64 = 0x00FFFFFFFFFFFFFF;
    pub const SINGLE_EXP_MASK: i32 = 0x7F800000;
    pub const SINGLE_FRACT_MASK: i32 = 0x007FFFFF;
    pub const SINGLE_FRACT_HOB: i32 = 0x00800000;
}

#[derive(Clone, PartialEq)]
enum Kind
{
    Finite,
    Infinity,
    NotANumber,
}

struct FloatingDecimal
{
    m_negative: bool,
    m_kind: Kind,
    m_decExponent: i32,
    m_digits: Vec<u8>,
    m_nDigits: i32,
    m_bigIntExp: i32,
    m_bigIntNBits: i32,
    m_mustSetRoundDir: bool,
    m_fromHex: bool,
    m_roundDir: i32,
}

#[derive(Clone)]
struct FdBigInt
{
    m_nWords: i32,
    m_data: Vec<i32>,
}

impl FdBigInt
{
    fn fromInt(value: i32) -> Self { Self { m_nWords: 1, m_data: vec![value] } }

    fn fromLong(value: i64) -> Self
    {
        let data: Vec<i32> = vec![value as i32, ((value as u64) >> 32) as i32];
        let words: i32 = if data[1] == 0 { 1 } else { 2 };
        Self { m_nWords: words, m_data: data }
    }

    fn copy(other: &Self) -> Self
    {
        Self { m_nWords: other.m_nWords, m_data: other.m_data[..other.m_nWords as usize].to_vec() }
    }

    fn fromDigits(lValue: i64, digits: &[u8], kDigits: i32, nDigits: i32) -> Self
    {
        let mut n: i32 = (nDigits + 8) / 9;
        if n < 2
        {
            n = 2;
        }

        let mut data: Vec<i32> = vec![0; n as usize];
        data[0] = lValue as i32;
        data[1] = ((lValue as u64) >> 32) as i32;
        let words: i32 = if data[1] == 0 { 1 } else { 2 };
        let mut result: Self = Self { m_nWords: words, m_data: data };

        let mut i: i32 = kDigits;
        let limit: i32 = nDigits - 5;
        while i < limit
        {
            let ilim: i32 = i + 5;
            let mut v: i32 = digits[i as usize] as i32 - '0' as i32;
            i += 1;
            while i < ilim
            {
                v = 10 * v + digits[i as usize] as i32 - '0' as i32;
                i += 1;
            }
            result.multaddMe(100000, v);
        }

        let mut factor: i32 = 1;
        let mut v: i32 = 0;
        while i < nDigits
        {
            v = 10 * v + digits[i as usize] as i32 - '0' as i32;
            factor *= 10;
            i += 1;
        }
        if factor != 1
        {
            result.multaddMe(factor, v);
        }
        result
    }

    fn word(value: i32) -> i64 { value as i64 & 0xFFFFFFFF }

    fn lshiftMe(&mut self, c: i32)
    {
        if c <= 0
        {
            if c != 0
            {
                throwable::throw(&throwable::ILLEGAL_ARGUMENT, "negative shift count");
            }
            return;
        }

        let wordcount: i32 = c >> 5;
        let bitcount: i32 = c & 31;
        let anticount: i32 = 32 - bitcount;
        let source: Vec<i32> = self.m_data.clone();
        let mut target: Vec<i32> = self.m_data.clone();
        if self.m_nWords + wordcount + 1 > target.len() as i32
        {
            target = vec![0; (self.m_nWords + wordcount + 1) as usize];
        }

        let mut t: i32 = self.m_nWords + wordcount;
        let mut s: i32 = self.m_nWords - 1;
        if bitcount == 0
        {
            target[wordcount as usize..(wordcount + self.m_nWords) as usize]
                .copy_from_slice(&source[..self.m_nWords as usize]);
            t = wordcount - 1;
        }
        else
        {
            target[t as usize] = ((source[s as usize] as u32) >> anticount) as i32;
            t -= 1;
            while s >= 1
            {
                let high: i32 = source[s as usize] << bitcount;
                s -= 1;
                target[t as usize] = high | ((source[s as usize] as u32) >> anticount) as i32;
                t -= 1;
            }
            target[t as usize] = source[s as usize] << bitcount;
            t -= 1;
        }

        while t >= 0
        {
            target[t as usize] = 0;
            t -= 1;
        }

        self.m_data = target;
        self.m_nWords += wordcount + 1;
        while self.m_nWords > 1 && self.m_data[(self.m_nWords - 1) as usize] == 0
        {
            self.m_nWords -= 1;
        }
    }

    fn normalizeMe(&mut self) -> i32
    {
        let mut wordcount: i32 = 0;
        let mut bitcount: i32 = 0;
        let mut v: i32 = 0;
        let mut src: i32 = self.m_nWords - 1;
        while src >= 0
        {
            v = self.m_data[src as usize];
            if v != 0
            {
                break;
            }
            wordcount += 1;
            src -= 1;
        }

        if src < 0
        {
            throwable::throw(&throwable::ILLEGAL_ARGUMENT, "zero value");
        }

        self.m_nWords -= wordcount;
        if (v & 0xF0000000u32 as i32) != 0
        {
            bitcount = 32;
            while (v & 0xF0000000u32 as i32) != 0
            {
                v = ((v as u32) >> 1) as i32;
                bitcount -= 1;
            }
        }
        else
        {
            while v <= 0x000FFFFF
            {
                v <<= 8;
                bitcount += 8;
            }
            while v <= 0x07FFFFFF
            {
                v <<= 1;
                bitcount += 1;
            }
        }

        if bitcount != 0
        {
            self.lshiftMe(bitcount);
        }
        bitcount
    }

    fn multInt(&self, iv: i32) -> Self
    {
        let v: i64 = iv as i64;
        let top: i64 = Self::word(self.m_data[(self.m_nWords - 1) as usize]);
        let size: i32 = if v.wrapping_mul(top) > 0x0FFFFFFF
        {
            self.m_nWords + 1
        }
        else
        {
            self.m_nWords
        };
        let mut r: Vec<i32> = vec![0; size as usize];
        let mut p: i64 = 0;
        for i in 0..self.m_nWords
        {
            p = p.wrapping_add(v.wrapping_mul(Self::word(self.m_data[i as usize])));
            r[i as usize] = p as i32;
            p = ((p as u64) >> 32) as i64;
        }

        if p == 0
        {
            return Self { m_nWords: self.m_nWords, m_data: r };
        }
        r[self.m_nWords as usize] = p as i32;
        Self { m_nWords: self.m_nWords + 1, m_data: r }
    }

    fn multaddMe(&mut self, iv: i32, addend: i32)
    {
        let v: i64 = iv as i64;
        let mut p: i64 = v.wrapping_mul(Self::word(self.m_data[0])).wrapping_add(Self::word(addend));
        self.m_data[0] = p as i32;
        p = ((p as u64) >> 32) as i64;
        for i in 1..self.m_nWords
        {
            p = p.wrapping_add(v.wrapping_mul(Self::word(self.m_data[i as usize])));
            self.m_data[i as usize] = p as i32;
            p = ((p as u64) >> 32) as i64;
        }

        if p != 0
        {
            self.m_data[self.m_nWords as usize] = p as i32;
            self.m_nWords += 1;
        }
    }

    fn mult(&self, other: &Self) -> Self
    {
        let mut r: Vec<i32> = vec![0; (self.m_nWords + other.m_nWords) as usize];
        for i in 0..self.m_nWords
        {
            let v: i64 = Self::word(self.m_data[i as usize]);
            let mut p: i64 = 0;
            let mut j: i32 = 0;
            while j < other.m_nWords
            {
                p = p.wrapping_add(
                    Self::word(r[(i + j) as usize]).wrapping_add(v.wrapping_mul(Self::word(other.m_data[j as usize]))),
                );
                r[(i + j) as usize] = p as i32;
                p = ((p as u64) >> 32) as i64;
                j += 1;
            }
            r[(i + j) as usize] = p as i32;
        }

        let mut i: i32 = r.len() as i32 - 1;
        while i > 0 && r[i as usize] == 0
        {
            i -= 1;
        }
        Self { m_nWords: i + 1, m_data: r }
    }

    fn add(&self, other: &Self) -> Self
    {
        let (a, b, nWordsA, nWordsB): (&[i32], &[i32], i32, i32) = if self.m_nWords >= other.m_nWords
        {
            (&self.m_data, &other.m_data, self.m_nWords, other.m_nWords)
        }
        else
        {
            (&other.m_data, &self.m_data, other.m_nWords, self.m_nWords)
        };

        let mut c: i64 = 0;
        let mut r: Vec<i32> = vec![0; nWordsA as usize];
        let mut i: i32 = 0;
        while i < nWordsA
        {
            c += Self::word(a[i as usize]);
            if i < nWordsB
            {
                c += Self::word(b[i as usize]);
            }
            r[i as usize] = c as i32;
            c >>= 32;
            i += 1;
        }

        if c != 0
        {
            r.push(c as i32);
            i += 1;
        }
        Self { m_nWords: i, m_data: r }
    }

    fn sub(&self, other: &Self) -> Self
    {
        let mut r: Vec<i32> = vec![0; self.m_nWords as usize];
        let n: i32 = self.m_nWords;
        let m: i32 = other.m_nWords;
        let mut nzeros: i32 = 0;
        let mut c: i64 = 0;
        for i in 0..n
        {
            c += Self::word(self.m_data[i as usize]);
            if i < m
            {
                c -= Self::word(other.m_data[i as usize]);
            }
            r[i as usize] = c as i32;
            if r[i as usize] == 0
            {
                nzeros += 1;
            }
            else
            {
                nzeros = 0;
            }
            c >>= 32;
        }
        Self { m_nWords: n - nzeros, m_data: r }
    }

    fn cmp(&self, other: &Self) -> i32
    {
        let mut i: i32;
        if self.m_nWords > other.m_nWords
        {
            let j: i32 = other.m_nWords - 1;
            i = self.m_nWords - 1;
            while i > j
            {
                if self.m_data[i as usize] != 0
                {
                    return 1;
                }
                i -= 1;
            }
        }
        else if self.m_nWords < other.m_nWords
        {
            let j: i32 = self.m_nWords - 1;
            i = other.m_nWords - 1;
            while i > j
            {
                if other.m_data[i as usize] != 0
                {
                    return -1;
                }
                i -= 1;
            }
        }
        else
        {
            i = self.m_nWords - 1;
        }

        while i > 0 && self.m_data[i as usize] == other.m_data[i as usize]
        {
            i -= 1;
        }

        let a: i32 = self.m_data[i as usize];
        let b: i32 = other.m_data[i as usize];
        if a < 0
        {
            return if b < 0 { a.wrapping_sub(b) } else { 1 };
        }
        if b < 0 { -1 } else { a.wrapping_sub(b) }
    }

    fn quoRemIteration(&mut self, s: &Self) -> i32
    {
        if self.m_nWords != s.m_nWords
        {
            throwable::throw(&throwable::ILLEGAL_ARGUMENT, "disparate values");
        }

        let n: i32 = self.m_nWords - 1;
        let mut q: i64 = Self::word(self.m_data[n as usize]) / s.m_data[n as usize] as i64;
        let mut diff: i64 = 0;
        for i in 0..=n
        {
            diff = diff.wrapping_add(
                Self::word(self.m_data[i as usize]).wrapping_sub(q.wrapping_mul(Self::word(s.m_data[i as usize]))),
            );
            self.m_data[i as usize] = diff as i32;
            diff >>= 32;
        }

        if diff != 0
        {
            let mut sum: i64 = 0;
            while sum == 0
            {
                sum = 0;
                for i in 0..=n
                {
                    sum += Self::word(self.m_data[i as usize]) + Self::word(s.m_data[i as usize]);
                    self.m_data[i as usize] = sum as i32;
                    sum >>= 32;
                }
                q -= 1;
            }
        }

        let mut p: i64 = 0;
        for i in 0..=n
        {
            p += 10 * Self::word(self.m_data[i as usize]);
            self.m_data[i as usize] = p as i32;
            p >>= 32;
        }
        q as i32
    }
}

fn countBits(v: i64) -> i32
{
    let mut v: i64 = v;
    if v == 0
    {
        return 0;
    }

    while (v & bits::HIGH_BYTE) == 0
    {
        v <<= 8;
    }
    while v > 0
    {
        v <<= 1;
    }

    let mut n: i32 = 0;
    while (v & bits::LOW_BYTES) != 0
    {
        v <<= 8;
        n += 8;
    }
    while v != 0
    {
        v <<= 1;
        n += 1;
    }
    n
}

fn big5pow(p: i32) -> FdBigInt
{
    if p < SMALL_5_POW.len() as i32
    {
        return FdBigInt::fromInt(SMALL_5_POW[p as usize]);
    }
    if p < LONG_5_POW.len() as i32
    {
        return FdBigInt::fromLong(LONG_5_POW[p as usize]);
    }

    let q: i32 = p >> 1;
    let r: i32 = p - q;
    let bigq: FdBigInt = big5pow(q);
    if r < SMALL_5_POW.len() as i32
    {
        return bigq.multInt(SMALL_5_POW[r as usize]);
    }
    bigq.mult(&big5pow(r))
}

fn multPow52(v: FdBigInt, p5: i32, p2: i32) -> FdBigInt
{
    let mut v: FdBigInt = v;
    if p5 != 0
    {
        if p5 < SMALL_5_POW.len() as i32
        {
            v = v.multInt(SMALL_5_POW[p5 as usize]);
        }
        else
        {
            v = v.mult(&big5pow(p5));
        }
    }

    if p2 != 0
    {
        v.lshiftMe(p2);
    }
    v
}

fn constructPow52(p5: i32, p2: i32) -> FdBigInt
{
    let mut v: FdBigInt = FdBigInt::copy(&big5pow(p5));
    if p2 != 0
    {
        v.lshiftMe(p2);
    }
    v
}

fn ulp(dval: f64, subtracting: bool) -> f64
{
    let lbits: i64 = dval.to_bits() as i64 & i64::MAX;
    let mut binexp: i32 = ((lbits as u64) >> 52) as i32;
    if subtracting && binexp >= 52 && (lbits & bits::FRACT_MASK) == 0
    {
        binexp -= 1;
    }

    let ulpval: f64 = if binexp > 52
    {
        f64::from_bits(((binexp - 52) as i64 as u64) << 52)
    }
    else if binexp == 0
    {
        f64::from_bits(1)
    }
    else
    {
        f64::from_bits(1u64 << (binexp - 1))
    };
    if subtracting { -ulpval } else { ulpval }
}

impl FloatingDecimal
{
    fn with(negative: bool, kind: Kind, decExponent: i32, digits: Vec<u8>, nDigits: i32) -> Self
    {
        Self {
            m_negative: negative,
            m_kind: kind,
            m_decExponent: decExponent,
            m_digits: digits,
            m_nDigits: nDigits,
            m_bigIntExp: 0,
            m_bigIntNBits: 0,
            m_mustSetRoundDir: false,
            m_fromHex: false,
            m_roundDir: 0,
        }
    }

    fn exceptional(negative: bool, kind: Kind) -> Self
    {
        let digits: &[u8] = if kind == Kind::Infinity
        {
            INFINITY
        }
        else
        {
            NOT_A_NUMBER
        };
        Self::with(negative, kind, 0, digits.to_vec(), digits.len() as i32)
    }

    fn fromDouble(d: f64) -> Self
    {
        let mut dBits: i64 = d.to_bits() as i64;
        let negative: bool = (dBits & i64::MIN) != 0;
        if negative
        {
            dBits ^= i64::MIN;
        }

        let mut binExp: i32 = ((dBits & bits::EXP_MASK) >> 52) as i32;
        let mut fractBits: i64 = dBits & bits::FRACT_MASK;
        if binExp == 0x7FF
        {
            if fractBits == 0
            {
                return Self::exceptional(negative, Kind::Infinity);
            }
            return Self::exceptional(false, Kind::NotANumber);
        }

        let mut result: Self = Self::with(negative, Kind::Finite, 0, Vec::new(), 0);
        let nSignificantBits: i32;
        if binExp == 0
        {
            if fractBits == 0
            {
                result.m_digits = ZERO.to_vec();
                result.m_nDigits = 1;
                return result;
            }

            while (fractBits & bits::FRACT_HOB) == 0
            {
                fractBits <<= 1;
                binExp -= 1;
            }
            nSignificantBits = 52 + binExp + 1;
            binExp += 1;
        }
        else
        {
            fractBits |= bits::FRACT_HOB;
            nSignificantBits = 53;
        }

        binExp -= 1023;
        result.dtoa(binExp, fractBits, nSignificantBits);
        result
    }

    fn fromFloat(f: f32) -> Self
    {
        let mut fBits: i32 = f.to_bits() as i32;
        let negative: bool = (fBits & i32::MIN) != 0;
        if negative
        {
            fBits ^= i32::MIN;
        }

        let mut binExp: i32 = (fBits & bits::SINGLE_EXP_MASK) >> 23;
        let mut fractBits: i32 = fBits & bits::SINGLE_FRACT_MASK;
        if binExp == 0xFF
        {
            if fractBits == 0
            {
                return Self::exceptional(negative, Kind::Infinity);
            }
            return Self::exceptional(false, Kind::NotANumber);
        }

        let mut result: Self = Self::with(negative, Kind::Finite, 0, Vec::new(), 0);
        let nSignificantBits: i32;
        if binExp == 0
        {
            if fractBits == 0
            {
                result.m_digits = ZERO.to_vec();
                result.m_nDigits = 1;
                return result;
            }

            while (fractBits & bits::SINGLE_FRACT_HOB) == 0
            {
                fractBits <<= 1;
                binExp -= 1;
            }
            nSignificantBits = 23 + binExp + 1;
            binExp += 1;
        }
        else
        {
            fractBits |= bits::SINGLE_FRACT_HOB;
            nSignificantBits = 24;
        }

        binExp -= 127;
        result.dtoa(binExp, (fractBits as i64) << 29, nSignificantBits);
        result
    }

    fn doubleToBigInt(&mut self, dval: f64) -> FdBigInt
    {
        let mut lbits: i64 = dval.to_bits() as i64 & i64::MAX;
        let mut binexp: i32 = ((lbits as u64) >> 52) as i32;
        lbits &= bits::FRACT_MASK;
        if binexp > 0
        {
            lbits |= bits::FRACT_HOB;
        }
        else
        {
            binexp += 1;
            while (lbits & bits::FRACT_HOB) == 0
            {
                lbits <<= 1;
                binexp -= 1;
            }
        }

        binexp -= 1023;
        let nbits: i32 = countBits(lbits);
        let lowOrderZeros: i32 = 53 - nbits;
        lbits = ((lbits as u64) >> lowOrderZeros) as i64;
        self.m_bigIntExp = binexp + 1 - nbits;
        self.m_bigIntNBits = nbits;
        FdBigInt::fromLong(lbits)
    }

    fn stickyRound(&self, dval: f64) -> f32
    {
        let mut lbits: i64 = dval.to_bits() as i64;
        let binexp: i64 = lbits & bits::EXP_MASK;
        if binexp == 0 || binexp == bits::EXP_MASK
        {
            return dval as f32;
        }
        lbits += self.m_roundDir as i64;
        f64::from_bits(lbits as u64) as f32
    }

    fn developLongDigits(&mut self, decExponent: i32, lvalue: i64, insignificant: i64)
    {
        let mut decExponent: i32 = decExponent;
        let mut lvalue: i64 = lvalue;
        let mut insignificant: i64 = insignificant;
        let mut i: i32 = 0;
        while insignificant >= 10
        {
            insignificant /= 10;
            i += 1;
        }

        if i != 0
        {
            let pow10: i64 = LONG_5_POW[i as usize] << i;
            let residue: i64 = lvalue % pow10;
            lvalue /= pow10;
            decExponent += i;
            if residue >= (pow10 >> 1)
            {
                lvalue += 1;
            }
        }

        let mut buffer: [u8; 26] = [0; 26];
        let ndigits: i32;
        let mut digitno: i32;
        if lvalue <= i32::MAX as i64
        {
            let mut ivalue: i32 = lvalue as i32;
            ndigits = 10;
            digitno = ndigits - 1;
            let mut c: i32 = ivalue % 10;
            ivalue /= 10;
            while c == 0
            {
                decExponent += 1;
                c = ivalue % 10;
                ivalue /= 10;
            }
            while ivalue != 0
            {
                buffer[digitno as usize] = (c + '0' as i32) as u8;
                digitno -= 1;
                decExponent += 1;
                c = ivalue % 10;
                ivalue /= 10;
            }
            buffer[digitno as usize] = (c + '0' as i32) as u8;
        }
        else
        {
            ndigits = 20;
            digitno = ndigits - 1;
            let mut c: i32 = (lvalue % 10) as i32;
            lvalue /= 10;
            while c == 0
            {
                decExponent += 1;
                c = (lvalue % 10) as i32;
                lvalue /= 10;
            }
            while lvalue != 0
            {
                buffer[digitno as usize] = (c + '0' as i32) as u8;
                digitno -= 1;
                decExponent += 1;
                c = (lvalue % 10) as i32;
                lvalue /= 10;
            }
            buffer[digitno as usize] = (c + '0' as i32) as u8;
        }

        let count: i32 = ndigits - digitno;
        self.m_digits = buffer[digitno as usize..ndigits as usize].to_vec();
        self.m_decExponent = decExponent + 1;
        self.m_nDigits = count;
    }

    fn roundup(&mut self)
    {
        let mut i: i32 = self.m_nDigits - 1;
        let mut q: u8 = self.m_digits[i as usize];
        if q == b'9'
        {
            while q == b'9' && i > 0
            {
                self.m_digits[i as usize] = b'0';
                i -= 1;
                q = self.m_digits[i as usize];
            }

            if q == b'9'
            {
                self.m_decExponent += 1;
                self.m_digits[0] = b'1';
                return;
            }
        }
        self.m_digits[i as usize] = q + 1;
    }

    fn dtoa(&mut self, binExp: i32, fractBits: i64, nSignificantBits: i32)
    {
        let mut fractBits: i64 = fractBits;
        let nFractBits: i32 = countBits(fractBits);
        let nTinyBits: i32 = 0.max(nFractBits - binExp - 1);
        if binExp <= 62
            && binExp >= -21
            && nTinyBits < LONG_5_POW.len() as i32
            && nFractBits + N_5_BITS[nTinyBits as usize] < 64
            && nTinyBits == 0
        {
            let halfULP: i64 = if binExp > nSignificantBits
            {
                1i64 << (binExp - nSignificantBits - 1)
            }
            else
            {
                0
            };
            if binExp >= 52
            {
                fractBits <<= binExp - 52;
            }
            else
            {
                fractBits = ((fractBits as u64) >> (52 - binExp)) as i64;
            }
            self.developLongDigits(0, fractBits, halfULP);
            return;
        }

        let d2: f64 = f64::from_bits((bits::EXP_ONE | (fractBits & !bits::FRACT_HOB)) as u64);
        let mut decExp: i32 =
            ((d2 - 1.5) * 0.289529654 + 0.176091259 + binExp as f64 * 0.301029995663981).floor() as i32;
        let b5: i32 = 0.max(-decExp);
        let mut b2: i32 = b5 + nTinyBits + binExp;
        let s5: i32 = 0.max(decExp);
        let mut s2: i32 = s5 + nTinyBits;
        let m5: i32 = b5;
        let mut m2: i32 = b2 - nSignificantBits;

        fractBits = ((fractBits as u64) >> (53 - nFractBits)) as i64;
        b2 -= nFractBits - 1;
        let common2factor: i32 = b2.min(s2);
        b2 -= common2factor;
        s2 -= common2factor;
        m2 -= common2factor;
        if nFractBits == 1
        {
            m2 -= 1;
        }

        if m2 < 0
        {
            b2 -= m2;
            s2 -= m2;
            m2 = 0;
        }

        let mut digits: Vec<u8> = vec![0; 18];
        let mut ndigit: i32 = 0;
        let bbits: i32 = nFractBits
            + b2
            + if b5 < N_5_BITS.len() as i32
            {
                N_5_BITS[b5 as usize]
            }
            else
            {
                b5 * 3
            };
        let tenSbits: i32 = s2
            + 1
            + if s5 + 1 < N_5_BITS.len() as i32
            {
                N_5_BITS[(s5 + 1) as usize]
            }
            else
            {
                (s5 + 1) * 3
            };
        let low: bool;
        let high: bool;
        let lowDigitDifference: i64;
        if bbits >= 64 || tenSbits >= 64
        {
            let mut b: FdBigInt = multPow52(FdBigInt::fromLong(fractBits), b5, b2);
            let mut s: FdBigInt = constructPow52(s5, s2);
            let mut m: FdBigInt = constructPow52(m5, m2);
            let shiftBias: i32 = s.normalizeMe();
            b.lshiftMe(shiftBias);
            m.lshiftMe(shiftBias);
            let tenS: FdBigInt = s.multInt(10);

            ndigit = 0;
            let mut q: i32 = b.quoRemIteration(&s);
            m = m.multInt(10);
            let mut isLow: bool = b.cmp(&m) < 0;
            let mut isHigh: bool = b.add(&m).cmp(&tenS) > 0;
            if q == 0 && !isHigh
            {
                decExp -= 1;
            }
            else
            {
                digits[ndigit as usize] = (b'0' as i32 + q) as u8;
                ndigit += 1;
            }

            if decExp <= -3 || decExp >= 8
            {
                isLow = false;
                isHigh = false;
            }

            while !isLow && !isHigh
            {
                q = b.quoRemIteration(&s);
                m = m.multInt(10);
                isLow = b.cmp(&m) < 0;
                isHigh = b.add(&m).cmp(&tenS) > 0;
                digits[ndigit as usize] = (b'0' as i32 + q) as u8;
                ndigit += 1;
            }

            if isHigh && isLow
            {
                b.lshiftMe(1);
                lowDigitDifference = b.cmp(&tenS) as i64;
            }
            else
            {
                lowDigitDifference = 0;
            }
            low = isLow;
            high = isHigh;
        }
        else if bbits < 32 && tenSbits < 32
        {
            let mut b: i32 = ((fractBits as i32).wrapping_mul(SMALL_5_POW[b5 as usize])) << b2;
            let s: i32 = SMALL_5_POW[s5 as usize] << s2;
            let mut m: i32 = SMALL_5_POW[m5 as usize] << m2;
            let tens: i32 = s.wrapping_mul(10);

            ndigit = 0;
            let mut q: i32 = b / s;
            b = 10i32.wrapping_mul(b % s);
            m = m.wrapping_mul(10);
            let mut isLow: bool = b < m;
            let mut isHigh: bool = b.wrapping_add(m) > tens;
            if q == 0 && !isHigh
            {
                decExp -= 1;
            }
            else
            {
                digits[ndigit as usize] = (b'0' as i32 + q) as u8;
                ndigit += 1;
            }

            if decExp <= -3 || decExp >= 8
            {
                isLow = false;
                isHigh = false;
            }

            while !isLow && !isHigh
            {
                q = b / s;
                b = 10i32.wrapping_mul(b % s);
                m = m.wrapping_mul(10);
                if m > 0
                {
                    isLow = b < m;
                    isHigh = b.wrapping_add(m) > tens;
                }
                else
                {
                    isLow = true;
                    isHigh = true;
                }
                digits[ndigit as usize] = (b'0' as i32 + q) as u8;
                ndigit += 1;
            }

            lowDigitDifference = ((b << 1).wrapping_sub(tens)) as i64;
            low = isLow;
            high = isHigh;
        }
        else
        {
            let mut b: i64 = (fractBits.wrapping_mul(LONG_5_POW[b5 as usize])) << b2;
            let s: i64 = LONG_5_POW[s5 as usize] << s2;
            let mut m: i64 = LONG_5_POW[m5 as usize] << m2;
            let tens: i64 = s.wrapping_mul(10);

            ndigit = 0;
            let mut q: i32 = (b / s) as i32;
            b = 10i64.wrapping_mul(b % s);
            m = m.wrapping_mul(10);
            let mut isLow: bool = b < m;
            let mut isHigh: bool = b.wrapping_add(m) > tens;
            if q == 0 && !isHigh
            {
                decExp -= 1;
            }
            else
            {
                digits[ndigit as usize] = (b'0' as i32 + q) as u8;
                ndigit += 1;
            }

            if decExp <= -3 || decExp >= 8
            {
                isLow = false;
                isHigh = false;
            }

            while !isLow && !isHigh
            {
                q = (b / s) as i32;
                b = 10i64.wrapping_mul(b % s);
                m = m.wrapping_mul(10);
                if m > 0
                {
                    isLow = b < m;
                    isHigh = b.wrapping_add(m) > tens;
                }
                else
                {
                    isLow = true;
                    isHigh = true;
                }
                digits[ndigit as usize] = (b'0' as i32 + q) as u8;
                ndigit += 1;
            }

            lowDigitDifference = (b << 1).wrapping_sub(tens);
            low = isLow;
            high = isHigh;
        }

        self.m_decExponent = decExp + 1;
        self.m_digits = digits;
        self.m_nDigits = ndigit;
        if high
        {
            if low
            {
                if lowDigitDifference == 0
                {
                    if (self.m_digits[(self.m_nDigits - 1) as usize] & 1) != 0
                    {
                        self.roundup();
                    }
                }
                else if lowDigitDifference > 0
                {
                    self.roundup();
                }
            }
            else
            {
                self.roundup();
            }
        }
    }

    fn toJavaFormatString(&self) -> String
    {
        let digits: &[u8] = &self.m_digits[..self.m_nDigits as usize];
        let mut out: Vec<u8> = Vec::with_capacity(26);
        if self.m_negative
        {
            out.push(b'-');
        }

        if self.m_kind != Kind::Finite
        {
            out.extend_from_slice(digits);
        }
        else if self.m_decExponent > 0 && self.m_decExponent < 8
        {
            let mut charLength: i32 = self.m_nDigits.min(self.m_decExponent);
            out.extend_from_slice(&digits[..charLength as usize]);
            if charLength < self.m_decExponent
            {
                charLength = self.m_decExponent - charLength;
                out.extend_from_slice(&ZERO[..charLength as usize]);
                out.extend_from_slice(b".0");
            }
            else
            {
                out.push(b'.');
                if charLength < self.m_nDigits
                {
                    out.extend_from_slice(&digits[charLength as usize..]);
                }
                else
                {
                    out.push(b'0');
                }
            }
        }
        else if self.m_decExponent <= 0 && self.m_decExponent > -3
        {
            out.extend_from_slice(b"0.");
            if self.m_decExponent != 0
            {
                out.extend_from_slice(&ZERO[..(-self.m_decExponent) as usize]);
            }
            out.extend_from_slice(digits);
        }
        else
        {
            out.push(digits[0]);
            out.push(b'.');
            if self.m_nDigits > 1
            {
                out.extend_from_slice(&digits[1..]);
            }
            else
            {
                out.push(b'0');
            }

            out.push(b'E');
            let mut e: i32;
            if self.m_decExponent <= 0
            {
                out.push(b'-');
                e = -self.m_decExponent + 1;
            }
            else
            {
                e = self.m_decExponent - 1;
            }

            if e <= 9
            {
                out.push((e + '0' as i32) as u8);
            }
            else if e <= 99
            {
                out.push((e / 10 + '0' as i32) as u8);
                out.push((e % 10 + '0' as i32) as u8);
            }
            else
            {
                out.push((e / 100 + '0' as i32) as u8);
                e %= 100;
                out.push((e / 10 + '0' as i32) as u8);
                out.push((e % 10 + '0' as i32) as u8);
            }
        }
        out.into_iter().map(char::from).collect()
    }

    fn readJavaFormatString(input: &str) -> Self
    {
        let text: Vec<u16> = javaTrim(input);
        match Self::parse(&text)
        {
            Some(parsed) => parsed,
            None => numberFormat(&text),
        }
    }

    fn parse(text: &[u16]) -> Option<Self>
    {
        let len: i32 = text.len() as i32;
        if len == 0
        {
            throwable::throw(&NUMBER_FORMAT, "empty String");
        }

        let at = |index: i32| -> Option<u16> {
            if index >= 0 && index < len
            {
                Some(text[index as usize])
            }
            else
            {
                None
            }
        };
        let mut negative: bool = false;
        let mut signSeen: bool = false;
        let mut i: i32 = 0;
        match at(i)?
        {
            0x2D =>
            {
                negative = true;
                i += 1;
                signSeen = true;
            }
            0x2B =>
            {
                i += 1;
                signSeen = true;
            }
            _ => (),
        }

        let mut c: u16 = at(i)?;
        if c == b'N' as u16 || c == b'I' as u16
        {
            let (target, nan): (&[u8], bool) = if c == b'N' as u16
            {
                (NOT_A_NUMBER, true)
            }
            else
            {
                (INFINITY, false)
            };
            let mut j: usize = 0;
            while i < len && j < target.len()
            {
                if text[i as usize] != target[j] as u16
                {
                    numberFormat(text);
                }
                i += 1;
                j += 1;
            }

            if j == target.len() && i == len
            {
                if nan
                {
                    return Some(Self::exceptional(false, Kind::NotANumber));
                }
                return Some(Self::exceptional(negative, Kind::Infinity));
            }
            return None;
        }

        if c == b'0' as u16 && len > i + 1
        {
            let ch: u16 = at(i + 1)?;
            if ch == b'x' as u16 || ch == b'X' as u16
            {
                return Some(parseHexString(text));
            }
        }

        let mut digits: Vec<u8> = vec![0; len as usize];
        let mut nDigits: i32 = 0;
        let mut decSeen: bool = false;
        let mut decPt: i32 = 0;
        let mut nLeadZero: i32 = 0;
        let mut nTrailZero: i32 = 0;
        while i < len
        {
            c = text[i as usize];
            if c >= b'1' as u16 && c <= b'9' as u16
            {
                while nTrailZero > 0
                {
                    digits[nDigits as usize] = b'0';
                    nDigits += 1;
                    nTrailZero -= 1;
                }
                digits[nDigits as usize] = c as u8;
                nDigits += 1;
            }
            else if c == b'0' as u16
            {
                if nDigits > 0
                {
                    nTrailZero += 1;
                }
                else
                {
                    nLeadZero += 1;
                }
            }
            else if c == b'.' as u16
            {
                if decSeen
                {
                    throwable::throw(&NUMBER_FORMAT, "multiple points");
                }
                decPt = i;
                if signSeen
                {
                    decPt -= 1;
                }
                decSeen = true;
            }
            else
            {
                break;
            }
            i += 1;
        }

        if nDigits == 0
        {
            digits = ZERO.to_vec();
            nDigits = 1;
            if nLeadZero == 0
            {
                numberFormat(text);
            }
        }

        let mut decExp: i32 = if decSeen
        {
            decPt - nLeadZero
        }
        else
        {
            nDigits + nTrailZero
        };
        if i < len
        {
            c = text[i as usize];
            if c == b'e' as u16 || c == b'E' as u16
            {
                let mut expSign: i32 = 1;
                let mut expVal: i32 = 0;
                let reallyBig: i32 = i32::MAX / 10;
                let mut expOverflow: bool = false;
                i += 1;
                match at(i)?
                {
                    0x2D =>
                    {
                        expSign = -1;
                        i += 1;
                    }
                    0x2B => i += 1,
                    _ => (),
                }
                let expAt: i32 = i;
                while i < len
                {
                    if expVal >= reallyBig
                    {
                        expOverflow = true;
                    }
                    c = text[i as usize];
                    i += 1;
                    if c >= b'0' as u16 && c <= b'9' as u16
                    {
                        expVal = expVal.wrapping_mul(10).wrapping_add(c as i32 - '0' as i32);
                    }
                    else
                    {
                        i -= 1;
                        break;
                    }
                }

                let expLimit: i32 = 324 + nDigits + nTrailZero;
                if expOverflow || expVal > expLimit
                {
                    decExp = expSign * expLimit;
                }
                else
                {
                    decExp += expSign * expVal;
                }

                if i == expAt
                {
                    numberFormat(text);
                }
            }
        }

        let suffix: bool = i == len - 1 && matches!(text[i as usize], 0x66 | 0x46 | 0x64 | 0x44);
        if i < len && !suffix
        {
            return None;
        }

        Some(Self::with(negative, Kind::Finite, decExp, digits, nDigits))
    }

    fn doubleValue(&mut self) -> f64
    {
        let kDigits: i32 = self.m_nDigits.min(16);
        match self.m_kind
        {
            Kind::NotANumber => return f64::NAN,
            Kind::Infinity =>
            {
                return if self.m_negative
                {
                    f64::NEG_INFINITY
                }
                else
                {
                    f64::INFINITY
                };
            }
            Kind::Finite => (),
        }

        if self.m_mustSetRoundDir
        {
            self.m_roundDir = 0;
        }

        let digit = |digits: &[u8], index: i32| -> i32 { digits[index as usize] as i32 - '0' as i32 };
        let mut iValue: i32 = digit(&self.m_digits, 0);
        let iDigits: i32 = kDigits.min(9);
        for i in 1..iDigits
        {
            iValue = iValue.wrapping_mul(10).wrapping_add(self.m_digits[i as usize] as i32) - '0' as i32;
        }

        let mut lValue: i64 = iValue as i64;
        for i in iDigits..kDigits
        {
            lValue = lValue.wrapping_mul(10).wrapping_add(digit(&self.m_digits, i) as i64);
        }

        let mut dValue: f64 = lValue as f64;
        let mut exp: i32 = self.m_decExponent - kDigits;
        if self.m_nDigits <= 15
        {
            if exp == 0 || dValue == 0.0
            {
                return if self.m_negative { -dValue } else { dValue };
            }

            if exp >= 0
            {
                if exp <= MAX_SMALL_TEN
                {
                    let rValue: f64 = dValue * SMALL_10_POW[exp as usize];
                    if self.m_mustSetRoundDir
                    {
                        let tValue: f64 = rValue / SMALL_10_POW[exp as usize];
                        self.m_roundDir = roundDirection(tValue, dValue);
                    }
                    return if self.m_negative { -rValue } else { rValue };
                }

                let slop: i32 = 15 - kDigits;
                if exp <= MAX_SMALL_TEN + slop
                {
                    dValue *= SMALL_10_POW[slop as usize];
                    let rValue: f64 = dValue * SMALL_10_POW[(exp - slop) as usize];
                    if self.m_mustSetRoundDir
                    {
                        let tValue: f64 = rValue / SMALL_10_POW[(exp - slop) as usize];
                        self.m_roundDir = roundDirection(tValue, dValue);
                    }
                    return if self.m_negative { -rValue } else { rValue };
                }
            }
            else if exp >= -MAX_SMALL_TEN
            {
                let rValue: f64 = dValue / SMALL_10_POW[(-exp) as usize];
                let tValue: f64 = rValue * SMALL_10_POW[(-exp) as usize];
                if self.m_mustSetRoundDir
                {
                    self.m_roundDir = roundDirection(tValue, dValue);
                }
                return if self.m_negative { -rValue } else { rValue };
            }
        }

        if exp > 0
        {
            if self.m_decExponent > 309
            {
                return if self.m_negative
                {
                    f64::NEG_INFINITY
                }
                else
                {
                    f64::INFINITY
                };
            }

            if (exp & 15) != 0
            {
                dValue *= SMALL_10_POW[(exp & 15) as usize];
            }

            exp >>= 4;
            if exp != 0
            {
                let mut j: usize = 0;
                while exp > 1
                {
                    if (exp & 1) != 0
                    {
                        dValue *= BIG_10_POW[j];
                    }
                    j += 1;
                    exp >>= 1;
                }

                let mut t: f64 = dValue * BIG_10_POW[j];
                if t.is_infinite()
                {
                    t = dValue / 2.0;
                    t *= BIG_10_POW[j];
                    if t.is_infinite()
                    {
                        return if self.m_negative
                        {
                            f64::NEG_INFINITY
                        }
                        else
                        {
                            f64::INFINITY
                        };
                    }
                    t = f64::MAX;
                }
                dValue = t;
            }
        }
        else if exp < 0
        {
            exp = -exp;
            if self.m_decExponent < -325
            {
                return if self.m_negative { -0.0 } else { 0.0 };
            }

            if (exp & 15) != 0
            {
                dValue /= SMALL_10_POW[(exp & 15) as usize];
            }

            exp >>= 4;
            if exp != 0
            {
                let mut j: usize = 0;
                while exp > 1
                {
                    if (exp & 1) != 0
                    {
                        dValue *= TINY_10_POW[j];
                    }
                    j += 1;
                    exp >>= 1;
                }

                let mut t: f64 = dValue * TINY_10_POW[j];
                if t == 0.0
                {
                    t = dValue * 2.0;
                    t *= TINY_10_POW[j];
                    if t == 0.0
                    {
                        return if self.m_negative { -0.0 } else { 0.0 };
                    }
                    t = f64::from_bits(1);
                }
                dValue = t;
            }
        }

        let bigD0: FdBigInt = FdBigInt::fromDigits(lValue, &self.m_digits, kDigits, self.m_nDigits);
        exp = self.m_decExponent - self.m_nDigits;
        loop
        {
            let mut bigB: FdBigInt = self.doubleToBigInt(dValue);
            let mut B2: i32;
            let B5: i32;
            let mut D2: i32;
            let D5: i32;
            if exp >= 0
            {
                B5 = 0;
                B2 = 0;
                D5 = exp;
                D2 = exp;
            }
            else
            {
                B5 = -exp;
                B2 = B5;
                D5 = 0;
                D2 = 0;
            }

            if self.m_bigIntExp >= 0
            {
                B2 += self.m_bigIntExp;
            }
            else
            {
                D2 -= self.m_bigIntExp;
            }

            let mut Ulp2: i32 = B2;
            let hulpbias: i32 = if self.m_bigIntExp + self.m_bigIntNBits <= -1022
            {
                self.m_bigIntExp + 1023 + 52
            }
            else
            {
                54 - self.m_bigIntNBits
            };
            B2 += hulpbias;
            D2 += hulpbias;

            let common2: i32 = B2.min(D2.min(Ulp2));
            B2 -= common2;
            D2 -= common2;
            Ulp2 -= common2;

            bigB = multPow52(bigB, B5, B2);
            let bigD: FdBigInt = multPow52(FdBigInt::copy(&bigD0), D5, D2);

            let diff: FdBigInt;
            let overvalue: bool;
            let mut cmpResult: i32 = bigB.cmp(&bigD);
            if cmpResult > 0
            {
                overvalue = true;
                let mut over: FdBigInt = bigB.sub(&bigD);
                if self.m_bigIntNBits == 1 && self.m_bigIntExp > -1022
                {
                    Ulp2 -= 1;
                    if Ulp2 < 0
                    {
                        Ulp2 = 0;
                        over.lshiftMe(1);
                    }
                }
                diff = over;
            }
            else if cmpResult < 0
            {
                overvalue = false;
                diff = bigD.sub(&bigB);
            }
            else
            {
                break;
            }

            let halfUlp: FdBigInt = constructPow52(B5, Ulp2);
            cmpResult = diff.cmp(&halfUlp);
            if cmpResult < 0
            {
                if self.m_mustSetRoundDir
                {
                    self.m_roundDir = if overvalue { -1 } else { 1 };
                }
                break;
            }

            if cmpResult == 0
            {
                dValue += 0.5 * ulp(dValue, overvalue);
                if self.m_mustSetRoundDir
                {
                    self.m_roundDir = if overvalue { -1 } else { 1 };
                }
                break;
            }

            dValue += ulp(dValue, overvalue);
            if dValue == 0.0 || dValue == f64::INFINITY
            {
                break;
            }
        }
        if self.m_negative { -dValue } else { dValue }
    }

    fn floatValue(&mut self) -> f32
    {
        let kDigits: i32 = self.m_nDigits.min(8);
        match self.m_kind
        {
            Kind::NotANumber => return f32::NAN,
            Kind::Infinity =>
            {
                return if self.m_negative
                {
                    f32::NEG_INFINITY
                }
                else
                {
                    f32::INFINITY
                };
            }
            Kind::Finite => (),
        }

        let mut iValue: i32 = self.m_digits[0] as i32 - '0' as i32;
        for i in 1..kDigits
        {
            iValue = iValue.wrapping_mul(10).wrapping_add(self.m_digits[i as usize] as i32) - '0' as i32;
        }

        let mut fValue: f32 = iValue as f32;
        let mut exp: i32 = self.m_decExponent - kDigits;
        if self.m_nDigits <= 7
        {
            if exp == 0 || fValue == 0.0
            {
                return if self.m_negative { -fValue } else { fValue };
            }

            if exp >= 0
            {
                if exp <= SINGLE_MAX_SMALL_TEN
                {
                    fValue *= SINGLE_SMALL_10_POW[exp as usize];
                    return if self.m_negative { -fValue } else { fValue };
                }

                let slop: i32 = 7 - kDigits;
                if exp <= SINGLE_MAX_SMALL_TEN + slop
                {
                    fValue *= SINGLE_SMALL_10_POW[slop as usize];
                    fValue *= SINGLE_SMALL_10_POW[(exp - slop) as usize];
                    return if self.m_negative { -fValue } else { fValue };
                }
            }
            else if exp >= -SINGLE_MAX_SMALL_TEN
            {
                fValue /= SINGLE_SMALL_10_POW[(-exp) as usize];
                return if self.m_negative { -fValue } else { fValue };
            }
        }
        else if self.m_decExponent >= self.m_nDigits && self.m_nDigits + self.m_decExponent <= 15
        {
            let mut lValue: i64 = iValue as i64;
            for i in kDigits..self.m_nDigits
            {
                lValue = lValue.wrapping_mul(10).wrapping_add((self.m_digits[i as usize] as i32 - '0' as i32) as i64);
            }

            let mut dValue: f64 = lValue as f64;
            exp = self.m_decExponent - self.m_nDigits;
            dValue *= SMALL_10_POW[exp as usize];
            fValue = dValue as f32;
            return if self.m_negative { -fValue } else { fValue };
        }

        if self.m_decExponent > 39
        {
            return if self.m_negative
            {
                f32::NEG_INFINITY
            }
            else
            {
                f32::INFINITY
            };
        }

        if self.m_decExponent < -46
        {
            return if self.m_negative { -0.0 } else { 0.0 };
        }

        self.m_mustSetRoundDir = !self.m_fromHex;
        let dValue: f64 = self.doubleValue();
        self.stickyRound(dValue)
    }
}

fn roundDirection(t: f64, d: f64) -> i32
{
    if t == d
    {
        0
    }
    else if t < d
    {
        1
    }
    else
    {
        -1
    }
}

fn javaTrim(input: &str) -> Vec<u16>
{
    let chars: Vec<u16> = input.encode_utf16().collect();
    let start: usize = chars.iter().position(|&c| c > 0x20).unwrap_or(chars.len());
    let end: usize = chars.iter().rposition(|&c| c > 0x20).map_or(start, |last| last + 1);
    chars[start..end.max(start)].to_vec()
}

fn numberFormat(text: &[u16]) -> !
{
    let shown: String = String::from_utf16_lossy(text);
    throwable::throw(&NUMBER_FORMAT, &format!("For input string: \"{shown}\""))
}

fn hexDigit(c: u16) -> Option<i64>
{
    let ch: char = char::from_u32(c as u32)?;
    ch.to_digit(16).map(|digit| digit as i64)
}

// a hand matcher for HexFloatPattern ([-+])?0[xX](((hex+)\.?)|((hex*)\.(hex+)))[pP]([-+])?(digit+)[fFdD]?
fn parseHexString(text: &[u16]) -> FloatingDecimal
{
    let len: usize = text.len();
    let mut i: usize = 0;
    let mut negative: bool = false;
    if i < len && (text[i] == b'-' as u16 || text[i] == b'+' as u16)
    {
        negative = text[i] == b'-' as u16;
        i += 1;
    }
    i += 2;

    let integerStart: usize = i;
    while i < len && hexDigit(text[i]).is_some()
    {
        i += 1;
    }
    let integerPart: Vec<u16> = text[integerStart..i.min(len)].to_vec();
    let mut fractionPart: Option<Vec<u16>> = None;
    if i < len && text[i] == b'.' as u16
    {
        i += 1;
        let fractionStart: usize = i;
        while i < len && hexDigit(text[i]).is_some()
        {
            i += 1;
        }
        if i > fractionStart
        {
            fractionPart = Some(text[fractionStart..i].to_vec());
        }
    }
    if integerPart.is_empty() && fractionPart.is_none()
    {
        numberFormat(text);
    }
    if i >= len || (text[i] != b'p' as u16 && text[i] != b'P' as u16)
    {
        numberFormat(text);
    }
    i += 1;

    let mut positiveExponent: bool = true;
    if i < len && (text[i] == b'-' as u16 || text[i] == b'+' as u16)
    {
        positiveExponent = text[i] == b'+' as u16;
        i += 1;
    }
    let exponentStart: usize = i;
    while i < len && text[i] >= b'0' as u16 && text[i] <= b'9' as u16
    {
        i += 1;
    }
    if i == exponentStart
    {
        numberFormat(text);
    }
    let exponentText: String = String::from_utf16_lossy(&text[exponentStart..i]);
    if i < len && matches!(text[i], 0x66 | 0x46 | 0x64 | 0x44)
    {
        i += 1;
    }
    if i != len
    {
        numberFormat(text);
    }

    let sign: f64 = if negative { -1.0 } else { 1.0 };
    let strip = |digits: &[u16]| -> Vec<u16> { digits.iter().copied().skip_while(|&c| c == b'0' as u16).collect() };
    let leftDigits: i32;
    let mut rightDigits: i32 = 0;
    let significand: Vec<u16> = match &fractionPart
    {
        Some(fraction) =>
        {
            let left: Vec<u16> = strip(&integerPart);
            leftDigits = left.len() as i32;
            rightDigits = fraction.len() as i32;
            let mut joined: Vec<u16> = left;
            joined.extend_from_slice(fraction);
            strip(&joined)
        }
        _ =>
        {
            let stripped: Vec<u16> = strip(&integerPart);
            leftDigits = stripped.len() as i32;
            stripped
        }
    };

    let signifLength: i32 = significand.len() as i32;
    let exponentAdjust: i32 = if leftDigits >= 1
    {
        4 * (leftDigits - 1)
    }
    else
    {
        -4 * (rightDigits - signifLength + 1)
    };
    if signifLength == 0
    {
        return FloatingDecimal::fromDouble(sign * 0.0);
    }

    let unsignedRawExponent: i32 = match exponentText.parse::<i32>()
    {
        Ok(value) => value,
        Err(_) => return FloatingDecimal::fromDouble(sign * if positiveExponent { f64::INFINITY } else { 0.0 }),
    };
    let rawExponent: i64 = (if positiveExponent { 1i64 } else { -1 }) * unsignedRawExponent as i64;
    let mut exponent: i64 = rawExponent + exponentAdjust as i64;

    let mut round: bool = false;
    let mut sticky: bool = false;
    let mut nextShift: i32;
    let mut significandBits: i64 = 0;
    let digit = |index: i32| -> i64 { hexDigit(significand[index as usize]).unwrap_or_default() };
    let leadingDigit: i64 = digit(0);
    if leadingDigit == 1
    {
        significandBits |= leadingDigit << 52;
        nextShift = 48;
    }
    else if leadingDigit <= 3
    {
        significandBits |= leadingDigit << 51;
        nextShift = 47;
        exponent += 1;
    }
    else if leadingDigit <= 7
    {
        significandBits |= leadingDigit << 50;
        nextShift = 46;
        exponent += 2;
    }
    else
    {
        significandBits |= leadingDigit << 49;
        nextShift = 45;
        exponent += 3;
    }

    let mut i: i32 = 1;
    while i < signifLength && nextShift >= 0
    {
        significandBits |= digit(i) << nextShift;
        nextShift -= 4;
        i += 1;
    }

    if i < signifLength
    {
        let mut currentDigit: i64 = digit(i);
        match nextShift
        {
            -4 =>
            {
                round = (currentDigit & 8) != 0;
                sticky = (currentDigit & 7) != 0;
            }
            -3 =>
            {
                significandBits |= (currentDigit & 8) >> 3;
                round = (currentDigit & 4) != 0;
                sticky = (currentDigit & 3) != 0;
            }
            -2 =>
            {
                significandBits |= (currentDigit & 12) >> 2;
                round = (currentDigit & 2) != 0;
                sticky = (currentDigit & 1) != 0;
            }
            _ =>
            {
                significandBits |= (currentDigit & 14) >> 1;
                round = (currentDigit & 1) != 0;
            }
        }

        i += 1;
        while i < signifLength && !sticky
        {
            currentDigit = digit(i);
            sticky = sticky || currentDigit != 0;
            i += 1;
        }
    }

    if exponent > 1023
    {
        return FloatingDecimal::fromDouble(sign * f64::INFINITY);
    }

    if exponent <= 1023 && exponent >= -1022
    {
        significandBits = (((exponent + 1023) << 52) & bits::EXP_MASK) | (bits::FRACT_MASK & significandBits);
    }
    else
    {
        if exponent < -1075
        {
            return FloatingDecimal::fromDouble(sign * 0.0);
        }

        sticky = sticky || round;
        let bitsDiscarded: i32 = 53 - (exponent as i32 - -1074 + 1);
        round = (significandBits & (1i64 << (bitsDiscarded - 1))) != 0;
        if bitsDiscarded > 1
        {
            let mask: i64 = !(-1i64 << (bitsDiscarded - 1));
            sticky = sticky || (significandBits & mask) != 0;
        }
        significandBits >>= bitsDiscarded;
        significandBits &= bits::FRACT_MASK;
    }

    let leastZero: bool = (significandBits & 1) == 0;
    if (leastZero && round && sticky) || (!leastZero && round)
    {
        significandBits += 1;
    }

    let value: f64 = f64::from_bits(significandBits as u64).copysign(sign);
    let mut result: FloatingDecimal = FloatingDecimal::fromDouble(value);
    if exponent >= -150 && exponent <= 127 && (significandBits & 0x0FFFFFFF) == 0 && (round || sticky)
    {
        if leastZero
        {
            if round ^ sticky
            {
                result.m_roundDir = 1;
            }
        }
        else if round
        {
            result.m_roundDir = -1;
        }
    }
    result.m_fromHex = true;
    result
}

pub fn doubleToString(value: f64) -> String { FloatingDecimal::fromDouble(value).toJavaFormatString() }

pub fn floatToString(value: f32) -> String { FloatingDecimal::fromFloat(value).toJavaFormatString() }

pub fn parseDouble(text: &str) -> f64 { FloatingDecimal::readJavaFormatString(text).doubleValue() }

pub fn parseFloat(text: &str) -> f32 { FloatingDecimal::readJavaFormatString(text).floatValue() }
