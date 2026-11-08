use crate::random::Random;
use crate::strict_math;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU8, Ordering};

pub const PI: f64 = std::f64::consts::PI;
const PI_OVER_4: f64 = f64::from_bits(0x3FE921FB54442D18);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum IntrinsicMode
{
    X87,
    Fdlibm,
}

static INTRINSIC_MODE: AtomicU8 = AtomicU8::new(0);

pub fn setIntrinsicMode(mode: IntrinsicMode)
{
    let value: u8 = match mode
    {
        IntrinsicMode::X87 => 0,
        IntrinsicMode::Fdlibm => 1,
    };
    INTRINSIC_MODE.store(value, Ordering::Relaxed);
}

pub fn intrinsicMode() -> IntrinsicMode
{
    match INTRINSIC_MODE.load(Ordering::Relaxed)
    {
        0 => IntrinsicMode::X87,
        _ => IntrinsicMode::Fdlibm,
    }
}

// hotspot 6 runs fsin and fcos inline for arguments within pi/4 and calls fdlibm for the rest
fn x87Range(x: f64) -> bool { intrinsicMode() == IntrinsicMode::X87 && !(x.abs() > PI_OVER_4) }

pub fn sin(x: f64) -> f64
{
    if x87Range(x)
    {
        return x87::sin(x);
    }
    strict_math::sin(x)
}

pub fn cos(x: f64) -> f64
{
    if x87Range(x)
    {
        return x87::cos(x);
    }
    strict_math::cos(x)
}

pub fn atan2(y: f64, x: f64) -> f64 { strict_math::atan2(y, x) }

pub fn atan(x: f64) -> f64 { strict_math::atan(x) }

pub fn pow(x: f64, y: f64) -> f64 { strict_math::pow(x, y) }

// hotspot 6 on x64 computes log with fldln2 and fyl2x
pub fn log(x: f64) -> f64
{
    if intrinsicMode() == IntrinsicMode::X87
    {
        return x87::log(x);
    }
    strict_math::log(x)
}

pub fn exp(x: f64) -> f64 { strict_math::exp(x) }

pub fn sqrt(x: f64) -> f64 { x.sqrt() }

pub fn floor(x: f64) -> f64 { x.floor() }

pub fn ceil(x: f64) -> f64 { x.ceil() }

pub fn rint(x: f64) -> f64 { x.round_ties_even() }

pub fn roundFloat(x: f32) -> i32 { (x + 0.5).floor() as f64 as i32 }

pub fn roundDouble(x: f64) -> i64 { (x + 0.5).floor() as i64 }

pub fn maxFloat(a: f32, b: f32) -> f32
{
    if a.is_nan()
    {
        return a;
    }
    if a == 0.0 && b == 0.0 && a.to_bits() == (-0.0f32).to_bits()
    {
        return b;
    }
    if a >= b { a } else { b }
}

pub fn minFloat(a: f32, b: f32) -> f32
{
    if a.is_nan()
    {
        return a;
    }
    if a == 0.0 && b == 0.0 && b.to_bits() == (-0.0f32).to_bits()
    {
        return b;
    }
    if a <= b { a } else { b }
}

pub fn maxDouble(a: f64, b: f64) -> f64
{
    if a.is_nan()
    {
        return a;
    }
    if a == 0.0 && b == 0.0 && a.to_bits() == (-0.0f64).to_bits()
    {
        return b;
    }
    if a >= b { a } else { b }
}

pub fn minDouble(a: f64, b: f64) -> f64
{
    if a.is_nan()
    {
        return a;
    }
    if a == 0.0 && b == 0.0 && b.to_bits() == (-0.0f64).to_bits()
    {
        return b;
    }
    if a <= b { a } else { b }
}

pub fn random() -> f64
{
    static GENERATOR: OnceLock<Random> = OnceLock::new();
    GENERATOR.get_or_init(Random::new).nextDouble()
}

#[cfg(target_arch = "x86_64")]
mod x87
{
    use std::arch::asm;

    pub fn sin(x: f64) -> f64
    {
        let mut value: f64 = x;
        // SAFETY: the asm only loads and stores the local through the pointer and clobbers the x87 stack it declares
        unsafe {
            asm!(
                "fld qword ptr [{value}]",
                "fsin",
                "fstp qword ptr [{value}]",
                value = in(reg) &mut value,
                out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
                out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
                options(nostack),
            );
        }
        value
    }

    pub fn cos(x: f64) -> f64
    {
        let mut value: f64 = x;
        // SAFETY: the asm only loads and stores the local through the pointer and clobbers the x87 stack it declares
        unsafe {
            asm!(
                "fld qword ptr [{value}]",
                "fcos",
                "fstp qword ptr [{value}]",
                value = in(reg) &mut value,
                out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
                out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
                options(nostack),
            );
        }
        value
    }

    pub fn log(x: f64) -> f64
    {
        let mut value: f64 = x;
        // SAFETY: the asm only loads and stores the local through the pointer and clobbers the x87 stack it declares
        unsafe {
            asm!(
                "fldln2",
                "fld qword ptr [{value}]",
                "fyl2x",
                "fstp qword ptr [{value}]",
                value = in(reg) &mut value,
                out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
                out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
                options(nostack),
            );
        }
        value
    }
}

#[cfg(not(target_arch = "x86_64"))]
mod x87
{
    pub fn sin(x: f64) -> f64 { crate::strict_math::sin(x) }

    pub fn cos(x: f64) -> f64 { crate::strict_math::cos(x) }

    pub fn log(x: f64) -> f64 { crate::strict_math::log(x) }
}
