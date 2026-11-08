use jvm::string::JString;
use jvm::throwable::Throwable;
use std::sync::{Mutex, MutexGuard, PoisonError};

pub fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> { mutex.lock().unwrap_or_else(PoisonError::into_inner) }

pub fn length(text: &Option<JString>) -> i32
{
    match text
    {
        Some(text) => text.length(),
        None => jvm::throwable::npe(),
    }
}

pub fn npeError() -> Throwable { Throwable::new(&jvm::throwable::NULL_POINTER, None) }
