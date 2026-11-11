use jvm::math;
use std::sync::OnceLock;

pub const POS_SCALE: f64 = 32.0;
const HALF_TURN: f64 = 180.0;
const TURN: f64 = 360.0;

fn table() -> &'static [f32]
{
    static SIN_TABLE: OnceLock<Vec<f32>> = OnceLock::new();
    SIN_TABLE.get_or_init(|| (0..65536).map(|i| math::sin(i as f64 * math::PI * 2.0 / 65536.0) as f32).collect())
}

pub fn sin(x: f32) -> f32 { table()[((x * 10430.378f32) as i32 & 0xFFFF) as usize] }

pub fn cos(x: f32) -> f32 { table()[((x * 10430.378f32 + 16384.0f32) as i32 & 0xFFFF) as usize] }

pub fn sqrt_float(x: f32) -> f32 { (x as f64).sqrt() as f32 }

pub fn sqrt_double(x: f64) -> f32 { x.sqrt() as f32 }

pub fn floor_float(x: f32) -> i32
{
    let n: i32 = x as i32;
    if x < n as f32 { n - 1 } else { n }
}

pub fn floor_double(x: f64) -> i32
{
    let n: i32 = x as i32;
    if x < n as f64 { n - 1 } else { n }
}

// entity packets carry positions in 1/32 blocks and round to nearest so neither side biases into blocks
pub fn encodePos(value: f64) -> i32 { floor_double(value * POS_SCALE + 0.5) }

pub fn wrapDegrees(angle: f64) -> f64 { (angle + HALF_TURN).rem_euclid(TURN) - HALF_TURN }

// turns the short way round so a yaw near the seam never spins a whole circle
pub fn lerpDegrees(from: f32, to: f32, t: f64) -> f32 { (from as f64 + wrapDegrees((to - from) as f64) * t) as f32 }

pub fn lerp3(from: [f64; 3], to: [f64; 3], t: f64) -> [f64; 3]
{
    std::array::from_fn(|i| from[i] + (to[i] - from[i]) * t)
}

pub fn abs(x: f32) -> f32 { if x >= 0.0 { x } else { -x } }

pub fn abs_max(a: f64, b: f64) -> f64
{
    let a: f64 = if a < 0.0 { -a } else { a };
    let b: f64 = if b < 0.0 { -b } else { b };
    if a > b { a } else { b }
}

pub fn bucketInt(a: i32, b: i32) -> i32
{
    if a < 0
    {
        return -((-a - 1).wrapping_div(b)) - 1;
    }

    a.wrapping_div(b)
}

pub fn stringNullOrLengthZero(string: Option<&jvm::string::JString>) -> bool
{
    match string
    {
        None => true,
        Some(text) => text.length() == 0,
    }
}
