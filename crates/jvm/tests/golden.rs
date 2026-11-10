#![cfg(feature = "golden")]

use jvm::random::Random;
use jvm::{math, strict_math};
use std::path::PathBuf;

fn golden(name: &str) -> String
{
    let dir: PathBuf = match std::env::var("B173_GOLDEN")
    {
        Ok(path) => PathBuf::from(path),
        Err(_) => PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../tools/reconstruct/golden"),
    };
    let path: PathBuf = dir.join(name);
    std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("golden file {} unreadable: {error}", path.display()))
}

fn bits(text: &str) -> f64 { f64::from_bits(u64::from_str_radix(text, 16).expect("golden bits are hex")) }

fn same(expected: f64, actual: f64) -> bool
{
    expected.to_bits() == actual.to_bits() || (expected.is_nan() && actual.is_nan())
}

#[test]
fn math_matches_oracle()
{
    let text: String = golden("math.txt");
    let mut failures: Vec<String> = Vec::new();
    for line in text.lines()
    {
        let fields: Vec<&str> = line.split(' ').collect();
        let values: Vec<f64> = fields[1..].iter().map(|field| bits(field)).collect();
        let checks: Vec<(&str, f64, f64)> = if fields[0] == "u"
        {
            let x: f64 = values[0];
            vec![
                ("StrictMath.sin", values[1], strict_math::sin(x)),
                ("StrictMath.cos", values[2], strict_math::cos(x)),
                ("StrictMath.atan", values[3], strict_math::atan(x)),
                ("StrictMath.log", values[4], strict_math::log(x)),
                ("StrictMath.exp", values[5], strict_math::exp(x)),
                ("Math.sin", values[6], math::sin(x)),
                ("Math.cos", values[7], math::cos(x)),
                ("Math.atan", values[8], math::atan(x)),
                ("Math.log", values[9], math::log(x)),
                ("Math.exp", values[10], math::exp(x)),
            ]
        }
        else
        {
            let (x, y): (f64, f64) = (values[0], values[1]);
            vec![
                ("StrictMath.atan2", values[2], strict_math::atan2(x, y)),
                ("StrictMath.pow", values[3], strict_math::pow(x, y)),
                ("Math.atan2", values[4], math::atan2(x, y)),
                ("Math.pow", values[5], math::pow(x, y)),
            ]
        };
        for (name, expected, actual) in checks
        {
            if !same(expected, actual)
            {
                failures.push(format!("{name}({line}) expected {:x} got {:x}", expected.to_bits(), actual.to_bits()));
            }
        }
    }
    assert!(failures.is_empty(), "{} mismatches, first: {:?}", failures.len(), &failures[..failures.len().min(10)]);
}

#[test]
fn random_matches_oracle()
{
    let text: String = golden("random.txt");
    let mut random: Random = Random::withSeed(0);
    for (number, line) in text.lines().enumerate()
    {
        let fields: Vec<&str> = line.split(' ').collect();
        let ok: bool = match fields[0]
        {
            "seed" =>
            {
                random = Random::withSeed(fields[1].parse().expect("seed"));
                true
            }
            "s" =>
            {
                random.setSeed(fields[1].parse().expect("seed"));
                true
            }
            "i" => random.nextInt() == fields[1].parse::<i32>().expect("int"),
            "n" => random.nextIntBounded(fields[1].parse().expect("bound")) == fields[2].parse::<i32>().expect("int"),
            "l" => random.nextLong() == fields[1].parse::<i64>().expect("long"),
            "f" => random.nextFloat().to_bits() as i32 == fields[1].parse::<i32>().expect("float"),
            "d" => random.nextDouble().to_bits() as i64 == fields[1].parse::<i64>().expect("double"),
            "b" => random.nextBoolean() == (fields[1] == "true"),
            "g" => random.nextGaussian().to_bits() as i64 == fields[1].parse::<i64>().expect("gaussian"),
            other => panic!("unknown golden op {other}"),
        };
        assert!(ok, "random.txt line {} ({line}) differs", number + 1);
    }
}

fn parsed<T>(parse: impl FnOnce() -> T, show: impl Fn(T) -> String) -> String
{
    match jvm::throwable::catchThrowable(&jvm::throwable::NUMBER_FORMAT, parse)
    {
        Ok(value) => show(value),
        Err(error) => format!("NFE:{}", error.getMessage().unwrap_or("null")),
    }
}

#[test]
fn floating_decimal_matches_oracle()
{
    jvm::throwable::installHook();
    let text: String = golden("decimal.txt");
    let mut failures: Vec<String> = Vec::new();
    for line in text.lines()
    {
        let (kind, rest): (&str, &str) = line.split_at(2);
        let (expected, actual): (String, String) = match kind
        {
            "d " =>
            {
                let (hex, shown): (&str, &str) = rest.split_once(' ').expect("d line");
                let value: f64 = f64::from_bits(u64::from_str_radix(hex, 16).expect("hex"));
                (shown.to_owned(), jvm::floating_decimal::doubleToString(value))
            }
            "f " =>
            {
                let (hex, shown): (&str, &str) = rest.split_once(' ').expect("f line");
                let value: f32 = f32::from_bits(u32::from_str_radix(hex, 16).expect("hex"));
                (shown.to_owned(), jvm::floating_decimal::floatToString(value))
            }
            _ =>
            {
                let fields: Vec<&str> = rest.splitn(3, '\t').collect();
                let input: &str = fields[2];
                let double: String =
                    parsed(|| jvm::floating_decimal::parseDouble(input), |value: f64| format!("{:x}", value.to_bits()));
                let float: String =
                    parsed(|| jvm::floating_decimal::parseFloat(input), |value: f32| format!("{:x}", value.to_bits()));
                (format!("{}\t{}", fields[0], fields[1]), format!("{double}\t{float}"))
            }
        };
        if expected != actual
        {
            failures.push(format!("{line} -> {actual}"));
        }
    }
    assert!(failures.is_empty(), "{} mismatches, first: {:#?}", failures.len(), &failures[..failures.len().min(10)]);
}

fn joinInts(values: &[i32]) -> String { values.iter().map(i32::to_string).collect::<Vec<String>>().join(",") }

#[test]
fn collections_match_oracle()
{
    use jvm::hash_map::{JHashMap, JHashSet};
    use jvm::string::JString;

    let text: String = golden("collections.txt");
    let mut map: JHashMap<i32, i32> = JHashMap::new();
    let mut set: JHashSet<JString> = JHashSet::new();
    let mut longs: JHashMap<i64, i32> = JHashMap::new();
    let mut original: Vec<i32> = Vec::new();
    for (number, line) in text.lines().enumerate()
    {
        let fields: Vec<&str> = line.split(' ').collect();
        let fail = |actual: String| panic!("collections.txt line {} ({line}) got {actual}", number + 1);
        match fields[0]
        {
            "round" =>
            {
                map = JHashMap::new();
                set = JHashSet::new();
                longs = JHashMap::new();
            }
            "p" =>
            {
                map.put(fields[1].parse().expect("key"), fields[4].parse().expect("value"));
                set.add(JString::from(fields[2]));
                longs.put(fields[3].parse().expect("long"), fields[4].parse().expect("value"));
            }
            "r" =>
            {
                map.remove(&fields[1].parse().expect("key"));
                set.remove(&JString::from(fields[2]));
                longs.remove(&fields[3].parse().expect("long"));
            }
            "m" =>
            {
                let actual: String =
                    map.entries().iter().map(|(key, value)| format!(" {key}={value}")).collect::<String>();
                if format!("m{actual}") != line
                {
                    fail(actual);
                }
            }
            "s" =>
            {
                let actual: String = set.items().iter().map(|key| format!(" {key}")).collect::<String>();
                if format!("s{actual}") != line
                {
                    fail(actual);
                }
            }
            "l" =>
            {
                let actual: String = longs.keys().iter().map(|key| format!(" {key}")).collect::<String>();
                if format!("l{actual}") != line
                {
                    fail(actual);
                }
            }
            "q" =>
            {
                original = fields
                    .get(2)
                    .filter(|list| !list.is_empty())
                    .map_or(Vec::new(), |list| list.split(',').map(|value| value.parse().expect("int")).collect());
                let mut chaotic: Vec<i32> = original.clone();
                let chaos: Random = Random::withSeed(fields[1].parse().expect("seed"));
                jvm::sort::sort(&mut chaotic, &mut |_: &i32, _: &i32| chaos.nextIntBounded(3) - 1);
                original.push(i32::MIN);
                original.extend(chaotic);
            }
            "n" =>
            {
                let split: usize = original.iter().position(|value| *value == i32::MIN).expect("marker");
                let mut natural: Vec<i32> = original[..split].to_vec();
                jvm::sort::sort(&mut natural, &mut |a: &i32, b: &i32| {
                    if a < b
                    {
                        -1
                    }
                    else if a == b
                    {
                        0
                    }
                    else
                    {
                        1
                    }
                });
                if fields.get(1).copied().unwrap_or_default() != joinInts(&natural)
                {
                    fail(joinInts(&natural));
                }
            }
            "c" =>
            {
                let split: usize = original.iter().position(|value| *value == i32::MIN).expect("marker");
                let chaotic: String = joinInts(&original[split + 1..]);
                if fields.get(1).copied().unwrap_or_default() != chaotic
                {
                    fail(chaotic);
                }
            }
            other => panic!("unknown op {other}"),
        }
    }
}

fn utf16(units: &[u16]) -> jvm::string::JString { jvm::string::JString::fromUtf16(units.to_vec()) }

// expected values are java 6 behavior: Character.digit accepts every unicode 4.0 Nd digit and parseInt rejects '+'
#[test]
fn parse_int_matches_java6()
{
    use jvm::integer::{parseInt, parseLong};
    assert_eq!(parseInt(&utf16(&[0x0661, 0x0662, 0x0663])).ok(), Some(123));
    assert_eq!(parseLong(&utf16(&[0xFF11, 0xFF12])).ok(), Some(12));
    assert_eq!(parseInt(&"-2147483648".into()).ok(), Some(i32::MIN));
    let overflow: jvm::throwable::Throwable = parseInt(&"2147483648".into()).expect_err("out of int range");
    assert_eq!(overflow.getMessage(), Some("For input string: \"2147483648\""));
    let plus: jvm::throwable::Throwable = parseInt(&"+1".into()).expect_err("java 6 rejects a plus sign");
    assert_eq!(plus.getMessage(), Some("For input string: \"+1\""));
    assert!(parseInt(&"-".into()).is_err());
    assert!(parseInt(&"".into()).is_err());
}

#[test]
fn case_mapping_matches_java6()
{
    let text = |units: &[u16]| utf16(units);
    assert_eq!(jvm::string::JString::from("TITLE").toLowerCaseIn("tr"), text(&[0x74, 0x0131, 0x74, 0x6C, 0x65]));
    assert_eq!(jvm::string::JString::from("title").toUpperCaseIn("tr"), text(&[0x54, 0x0130, 0x54, 0x4C, 0x45]));
    assert_eq!(jvm::string::JString::from("TITLE").toLowerCaseIn("en"), jvm::string::JString::from("title"));
    assert_eq!(text(&[0xDF]).toUpperCaseIn("en"), jvm::string::JString::from("SS"));
    assert_eq!(text(&[0x039F, 0x0394, 0x039F, 0x03A3]).toLowerCaseIn("en"), text(&[0x03BF, 0x03B4, 0x03BF, 0x03C2]));
    assert_eq!(text(&[0x03A3, 0x0391]).toLowerCaseIn("en"), text(&[0x03C3, 0x03B1]));
}

#[test]
fn lone_surrogate_encodes_as_question_mark()
{
    assert_eq!(utf16(&[0x61, 0xD800, 0x62]).getBytes(), b"a?b".to_vec());
}
