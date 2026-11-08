use crate::string::JString;
use std::cell::Cell;
use std::sync::Mutex;

// equals and hashCode as the java class defines them, identity for classes that inherit Object's
pub trait JavaObject
{
    fn javaEquals(&self, other: &Self) -> bool;
    fn javaHash(&self) -> i32;
}

impl JavaObject for i32
{
    fn javaEquals(&self, other: &Self) -> bool { self == other }

    fn javaHash(&self) -> i32 { *self }
}

impl JavaObject for i64
{
    fn javaEquals(&self, other: &Self) -> bool { self == other }

    fn javaHash(&self) -> i32 { (*self ^ ((*self as u64) >> 32) as i64) as i32 }
}

impl JavaObject for bool
{
    fn javaEquals(&self, other: &Self) -> bool { self == other }

    fn javaHash(&self) -> i32 { if *self { 1231 } else { 1237 } }
}

impl JavaObject for JString
{
    fn javaEquals(&self, other: &Self) -> bool { self == other }

    fn javaHash(&self) -> i32 { self.hashCode() }
}

impl<T: JavaObject + ?Sized> JavaObject for std::rc::Rc<T>
{
    fn javaEquals(&self, other: &Self) -> bool { (**self).javaEquals(other) }

    fn javaHash(&self) -> i32 { (**self).javaHash() }
}

// hotspot 6 hashCode mode 0 draws identity hashes from one global park miller generator
static IDENTITY_SEED: Mutex<u64> = Mutex::new(1234567);

fn nextIdentityHash() -> i32
{
    const MODULUS: u64 = 2147483647;
    const MULTIPLIER: u64 = 16807;
    let mut seed = IDENTITY_SEED.lock().expect("identity hash lock is never poisoned");
    *seed = (*seed * MULTIPLIER) % MODULUS;
    *seed as i32
}

#[derive(Default)]
pub struct IdentityHash
{
    m_value: Cell<Option<i32>>,
}

impl IdentityHash
{
    pub fn new() -> Self { Self { m_value: Cell::new(None) } }

    pub fn get(&self) -> i32
    {
        if let Some(value) = self.m_value.get()
        {
            return value;
        }

        let value: i32 = nextIdentityHash();
        self.m_value.set(Some(value));
        value
    }
}
