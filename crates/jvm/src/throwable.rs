use std::any::Any;
use std::fmt;
use std::panic::{self, AssertUnwindSafe};

pub struct ThrowableClass
{
    pub m_name: &'static str,
    pub m_parent: Option<&'static ThrowableClass>,
}

pub static THROWABLE: ThrowableClass = ThrowableClass { m_name: "java.lang.Throwable", m_parent: None };
pub static ERROR: ThrowableClass = ThrowableClass { m_name: "java.lang.Error", m_parent: Some(&THROWABLE) };
pub static EXCEPTION: ThrowableClass = ThrowableClass { m_name: "java.lang.Exception", m_parent: Some(&THROWABLE) };
pub static RUNTIME: ThrowableClass =
    ThrowableClass { m_name: "java.lang.RuntimeException", m_parent: Some(&EXCEPTION) };
pub static ILLEGAL_ARGUMENT: ThrowableClass =
    ThrowableClass { m_name: "java.lang.IllegalArgumentException", m_parent: Some(&RUNTIME) };
pub static ILLEGAL_STATE: ThrowableClass =
    ThrowableClass { m_name: "java.lang.IllegalStateException", m_parent: Some(&RUNTIME) };
pub static NULL_POINTER: ThrowableClass =
    ThrowableClass { m_name: "java.lang.NullPointerException", m_parent: Some(&RUNTIME) };
pub static ARITHMETIC: ThrowableClass =
    ThrowableClass { m_name: "java.lang.ArithmeticException", m_parent: Some(&RUNTIME) };
pub static CLASS_CAST: ThrowableClass =
    ThrowableClass { m_name: "java.lang.ClassCastException", m_parent: Some(&RUNTIME) };
pub static INDEX_OUT_OF_BOUNDS: ThrowableClass =
    ThrowableClass { m_name: "java.lang.IndexOutOfBoundsException", m_parent: Some(&RUNTIME) };
pub static ARRAY_INDEX: ThrowableClass =
    ThrowableClass { m_name: "java.lang.ArrayIndexOutOfBoundsException", m_parent: Some(&INDEX_OUT_OF_BOUNDS) };
pub static STRING_INDEX: ThrowableClass =
    ThrowableClass { m_name: "java.lang.StringIndexOutOfBoundsException", m_parent: Some(&INDEX_OUT_OF_BOUNDS) };
pub static NEGATIVE_ARRAY_SIZE: ThrowableClass =
    ThrowableClass { m_name: "java.lang.NegativeArraySizeException", m_parent: Some(&RUNTIME) };
pub static CONCURRENT_MODIFICATION: ThrowableClass =
    ThrowableClass { m_name: "java.util.ConcurrentModificationException", m_parent: Some(&RUNTIME) };
pub static NO_SUCH_ELEMENT: ThrowableClass =
    ThrowableClass { m_name: "java.util.NoSuchElementException", m_parent: Some(&RUNTIME) };
pub static NUMBER_FORMAT: ThrowableClass =
    ThrowableClass { m_name: "java.lang.NumberFormatException", m_parent: Some(&ILLEGAL_ARGUMENT) };
pub static INSTANTIATION: ThrowableClass =
    ThrowableClass { m_name: "java.lang.InstantiationException", m_parent: Some(&EXCEPTION) };
pub static IO: ThrowableClass = ThrowableClass { m_name: "java.io.IOException", m_parent: Some(&EXCEPTION) };
pub static EOF: ThrowableClass = ThrowableClass { m_name: "java.io.EOFException", m_parent: Some(&IO) };
pub static OUT_OF_MEMORY: ThrowableClass =
    ThrowableClass { m_name: "java.lang.OutOfMemoryError", m_parent: Some(&ERROR) };
// never thrown yet: rust detects overflow only through the guard page, whose SIGSEGV handler prints and aborts on
// the faulting stack, so catch_unwind cannot turn it into this error; a catchable one needs explicit depth checks
// (remaining stack measured against the thread's base at recursive entry points) that throw it before the guard page
pub static STACK_OVERFLOW: ThrowableClass =
    ThrowableClass { m_name: "java.lang.StackOverflowError", m_parent: Some(&ERROR) };

pub struct Throwable
{
    pub m_class: &'static ThrowableClass,
    pub m_message: Option<String>,
}

impl Throwable
{
    pub fn new(class: &'static ThrowableClass, message: Option<String>) -> Self
    {
        Self { m_class: class, m_message: message }
    }

    pub fn instanceOf(&self, class: &ThrowableClass) -> bool
    {
        let mut current: Option<&ThrowableClass> = Some(self.m_class);
        while let Some(candidate) = current
        {
            if std::ptr::eq(candidate, class)
            {
                return true;
            }
            current = candidate.m_parent;
        }
        false
    }

    pub fn getMessage(&self) -> Option<&str> { self.m_message.as_deref() }

    // the jvm also prints one at line per stack frame, which has no counterpart here
    pub fn printStackTrace(&self)
    {
        eprintln!("{self}");
    }
}

impl fmt::Display for Throwable
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        match &self.m_message
        {
            Some(message) => write!(formatter, "{}: {message}", self.m_class.m_name),
            None => formatter.write_str(self.m_class.m_name),
        }
    }
}

impl fmt::Debug for Throwable
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result { fmt::Display::fmt(self, formatter) }
}

pub fn throw(class: &'static ThrowableClass, message: &str) -> !
{
    rethrow(Throwable::new(class, Some(message.to_owned())))
}

pub fn throwBare(class: &'static ThrowableClass) -> ! { rethrow(Throwable::new(class, None)) }

pub fn rethrow(throwable: Throwable) -> ! { panic::panic_any(throwable) }

pub fn npe() -> ! { throwBare(&NULL_POINTER) }

pub trait OrNpe<T>
{
    fn orNpe(self) -> T;
}

impl<T> OrNpe<T> for Option<T>
{
    fn orNpe(self) -> T
    {
        match self
        {
            Some(value) => value,
            None => npe(),
        }
    }
}

// a java try with one catch clause, rust runtime panics count as the java exception they stand for
pub fn catchThrowable<R>(class: &ThrowableClass, body: impl FnOnce() -> R) -> Result<R, Throwable>
{
    match panic::catch_unwind(AssertUnwindSafe(body))
    {
        Ok(value) => Ok(value),
        Err(payload) =>
        {
            let throwable: Throwable = fromPayload(payload);
            if throwable.instanceOf(class)
            {
                return Err(throwable);
            }
            rethrow(throwable)
        }
    }
}

pub fn fromPayload(payload: Box<dyn Any + Send>) -> Throwable
{
    let payload: Box<dyn Any + Send> = match payload.downcast::<Throwable>()
    {
        Ok(throwable) => return *throwable,
        Err(other) => other,
    };

    let text: String = if let Some(text) = payload.downcast_ref::<String>()
    {
        text.clone()
    }
    else if let Some(text) = payload.downcast_ref::<&str>()
    {
        (*text).to_owned()
    }
    else
    {
        String::new()
    };
    fromRustPanic(&text)
}

fn fromRustPanic(text: &str) -> Throwable
{
    const INDEX_PREFIX: &str = "index out of bounds: the len is ";
    if let Some(rest) = text.strip_prefix(INDEX_PREFIX)
    {
        let index: &str = rest.rsplit(' ').next().unwrap_or_default();
        return Throwable::new(&ARRAY_INDEX, Some(index.to_owned()));
    }

    if text.starts_with("attempt to divide by zero")
        || text.starts_with("attempt to calculate the remainder with a divisor of zero")
    {
        return Throwable::new(&ARITHMETIC, Some("/ by zero".to_owned()));
    }

    if text.starts_with("capacity overflow") || text.starts_with("memory allocation")
    {
        return Throwable::new(&OUT_OF_MEMORY, Some("Java heap space".to_owned()));
    }

    Throwable::new(&ERROR, Some(text.to_owned()))
}

fn standsForJava(payload: &(dyn Any + Send)) -> bool
{
    if payload.is::<Throwable>()
    {
        return true;
    }

    let text: &str = if let Some(text) = payload.downcast_ref::<String>()
    {
        text
    }
    else if let Some(text) = payload.downcast_ref::<&str>()
    {
        text
    }
    else
    {
        return false;
    };
    !std::ptr::eq(fromRustPanic(text).m_class, &ERROR)
}

// java prints nothing when it throws, so only panics with no java counterpart keep the rust message
pub fn installHook()
{
    let previous = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        if !standsForJava(info.payload())
        {
            previous(info);
        }
    }));
}
