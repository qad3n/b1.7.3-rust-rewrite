use crate::string::JString;
use crate::{strict_math, system, thread, throwable};
use std::cell::Cell;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};

mod lcg
{
    pub const MULTIPLIER: i64 = 0x5DEECE66D;
    pub const ADDEND: i64 = 0xB;
    pub const MASK: i64 = (1 << 48) - 1;
}

const SEED_UNIQUIFIER_START: i64 = 8682522807148012;

static SEED_UNIQUIFIER: AtomicI64 = AtomicI64::new(SEED_UNIQUIFIER_START);
static ORACLE_SEEDING: AtomicBool = AtomicBool::new(false);

thread_local! {
    static THREAD_SEED_COUNT: Cell<i64> = const { Cell::new(0) };
}

pub struct Random
{
    m_seed: AtomicI64,
    m_gaussian: Mutex<Option<f64>>,
}

impl Random
{
    // java 6 seeds from ++seedUniquifier + nanoTime, the oracle harness from the thread name and a per thread counter
    pub fn new() -> Self
    {
        if ORACLE_SEEDING.load(Ordering::Relaxed)
        {
            return Self::withSeed(oracleSeed());
        }

        let uniquifier: i64 = SEED_UNIQUIFIER.fetch_add(1, Ordering::Relaxed).wrapping_add(1);
        Self::withSeed(uniquifier.wrapping_add(system::nanoTime()))
    }

    pub fn withSeed(seed: i64) -> Self
    {
        let random: Self = Self { m_seed: AtomicI64::new(0), m_gaussian: Mutex::new(None) };
        random.setSeed(seed);
        random
    }

    pub fn setSeed(&self, seed: i64)
    {
        let mut gaussian = self.m_gaussian.lock().expect("random gaussian lock is never poisoned");
        self.m_seed.store((seed ^ lcg::MULTIPLIER) & lcg::MASK, Ordering::Relaxed);
        *gaussian = None;
    }

    pub fn next(&self, bits: i32) -> i32
    {
        let old: i64 = self.m_seed.load(Ordering::Relaxed);
        let next: i64 = (old.wrapping_mul(lcg::MULTIPLIER).wrapping_add(lcg::ADDEND)) & lcg::MASK;
        self.m_seed.store(next, Ordering::Relaxed);
        ((next as u64) >> (48 - bits)) as i32
    }

    pub fn nextInt(&self) -> i32 { self.next(32) }

    pub fn nextIntBounded(&self, n: i32) -> i32
    {
        if n <= 0
        {
            throwable::throw(&throwable::ILLEGAL_ARGUMENT, "n must be positive");
        }

        if (n & n.wrapping_neg()) == n
        {
            return ((n as i64).wrapping_mul(self.next(31) as i64) >> 31) as i32;
        }

        loop
        {
            let bits: i32 = self.next(31);
            let val: i32 = bits % n;
            if bits.wrapping_sub(val).wrapping_add(n - 1) >= 0
            {
                return val;
            }
        }
    }

    pub fn nextLong(&self) -> i64 { ((self.next(32) as i64) << 32).wrapping_add(self.next(32) as i64) }

    pub fn nextBoolean(&self) -> bool { self.next(1) != 0 }

    pub fn nextFloat(&self) -> f32 { self.next(24) as f32 / (1 << 24) as f32 }

    pub fn nextDouble(&self) -> f64
    {
        (((self.next(26) as i64) << 27) + self.next(27) as i64) as f64 / (1i64 << 53) as f64
    }

    pub fn nextGaussian(&self) -> f64
    {
        let mut gaussian = self.m_gaussian.lock().expect("random gaussian lock is never poisoned");
        if let Some(next) = gaussian.take()
        {
            return next;
        }

        loop
        {
            let v1: f64 = 2.0 * self.nextDouble() - 1.0;
            let v2: f64 = 2.0 * self.nextDouble() - 1.0;
            let s: f64 = v1 * v1 + v2 * v2;
            if s >= 1.0 || s == 0.0
            {
                continue;
            }

            let multiplier: f64 = (-2.0 * strict_math::log(s) / s).sqrt();
            *gaussian = Some(v2 * multiplier);
            return v1 * multiplier;
        }
    }
}

impl Default for Random
{
    fn default() -> Self { Self::new() }
}

pub fn setOracleSeeding(enabled: bool) { ORACLE_SEEDING.store(enabled, Ordering::Relaxed); }

fn oracleSeed() -> i64
{
    let name: JString = thread::currentName();
    let count: i64 = THREAD_SEED_COUNT.with(|count| {
        count.set(count.get() + 1);
        count.get()
    });
    (name.hashCode() as i64).wrapping_mul(0x9E3779B97F4A7C15u64 as i64).wrapping_add(count)
}
