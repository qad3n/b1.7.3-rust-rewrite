use std::any::Any;
use std::backtrace::Backtrace;
use std::cell::RefCell;
use std::fmt;
use std::panic::{self, AssertUnwindSafe};

pub struct ThrowableClass
{
    pub m_name: &'static str,
    pub m_parent: Option<&'static ThrowableClass>,
}

pub static THROWABLE: ThrowableClass = ThrowableClass { m_name: "java.lang.Throwable", m_parent: None };
pub static ERROR: ThrowableClass = ThrowableClass { m_name: "java.lang.Error", m_parent: Some(&THROWABLE) };
// rewrite: a rust panic with no java counterpart keeps its own name instead of posing as java.lang.Error
pub static RUST_PANIC: ThrowableClass = ThrowableClass { m_name: "rust panic", m_parent: Some(&ERROR) };
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

// crate paths whose frames belong to the game, everything else is std, the runtime or a library
// release builds keep only line tables, so frames are told apart by source path and not by module path
const GAME_SOURCES: [&str; 2] = ["crates/", "vendor/"];
// the throw machinery itself, above the code that threw
const THROW_SOURCE: &str = "crates/jvm/src/throwable.rs";
const MAX_FRAMES: usize = 48;

thread_local! {
    // the trace of the panic now unwinding, taken by whoever catches it
    static THROWN: RefCell<Option<Trace>> = const { RefCell::new(None) };
}

// where a throwable came from, the rust frames and what the code it unwound through was doing
pub struct Trace
{
    m_location: String,
    m_frames: Backtrace,
    m_context: Vec<String>,
}

impl Trace
{
    // java style at lines for the game's own frames, innermost first
    fn frames(&self) -> Vec<String>
    {
        let text: String = self.m_frames.to_string();
        let mut frames: Vec<String> = Vec::new();
        let mut name: &str = "";
        for line in text.lines().map(str::trim)
        {
            let Some(location) = line.strip_prefix("at ")
            else
            {
                name = line.split_once(": ").map_or("", |(_, symbol)| symbol);
                continue;
            };

            let source: &str = relative(location);
            if GAME_SOURCES.iter().any(|root| source.starts_with(root)) && !source.starts_with(THROW_SOURCE)
            {
                frames.push(format!("{name} ({source})"));
            }
        }
        frames.truncate(MAX_FRAMES);
        frames
    }
}

// source paths from the workspace down so a shared report carries no home folder
fn relative(location: &str) -> &str
{
    ["/crates/", "/vendor/"]
        .iter()
        .filter_map(|root| location.find(root))
        .min()
        .map_or(location, |start| &location[start + 1..])
}

pub struct Throwable
{
    pub m_class: &'static ThrowableClass,
    pub m_message: Option<String>,
    pub m_trace: Option<Box<Trace>>,
}

impl Throwable
{
    pub fn new(class: &'static ThrowableClass, message: Option<String>) -> Self
    {
        Self { m_class: class, m_message: message, m_trace: None }
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
    pub fn printStackTrace(&self) { crate::system::errPrintln(&self.to_string()); }

    // the throwable, the rust frames that threw it and what the code it unwound through was doing
    pub fn stackTrace(&self) -> String
    {
        let mut text: String = format!("{self}\n");
        let Some(trace) = &self.m_trace
        else
        {
            return text;
        };

        text += &format!("\tthrown at {}\n", relative(&trace.m_location));
        for frame in trace.frames()
        {
            text += &format!("\tat {frame}\n");
        }
        if trace.m_context.is_empty()
        {
            return text;
        }

        text += "while\n";
        for context in &trace.m_context
        {
            text += &format!("\t{context}\n");
        }
        text
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

#[track_caller]
pub fn throw(class: &'static ThrowableClass, message: &str) -> !
{
    rethrow(Throwable::new(class, Some(message.to_owned())))
}

#[track_caller]
pub fn throwBare(class: &'static ThrowableClass) -> ! { rethrow(Throwable::new(class, None)) }

#[track_caller]
pub fn rethrow(throwable: Throwable) -> ! { panic::panic_any(throwable) }

#[track_caller]
pub fn npe() -> ! { throwBare(&NULL_POINTER) }

pub trait OrNpe<T>
{
    fn orNpe(self) -> T;
}

impl<T> OrNpe<T> for Option<T>
{
    #[track_caller]
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

// a rethrown throwable keeps the trace of its first throw
pub fn fromPayload(payload: Box<dyn Any + Send>) -> Throwable
{
    let mut throwable: Throwable = match payload.downcast::<Throwable>()
    {
        Ok(throwable) => *throwable,
        Err(other) => fromRustPanic(panicText(other.as_ref()).unwrap_or_default()),
    };
    let thrown: Option<Trace> = THROWN.with(RefCell::take);
    if throwable.m_trace.is_none()
    {
        throwable.m_trace = thrown.map(Box::new);
    }
    throwable
}

// records what unwinding code was doing, call from a drop guard while std::thread::panicking
pub fn noteUnwind(context: impl FnOnce() -> String)
{
    THROWN.with(|slot| {
        if let Ok(mut thrown) = slot.try_borrow_mut()
            && let Some(trace) = thrown.as_mut()
        {
            trace.m_context.push(context());
        }
    });
}

fn panicText(payload: &(dyn Any + Send)) -> Option<&str>
{
    if let Some(text) = payload.downcast_ref::<String>()
    {
        return Some(text);
    }

    payload.downcast_ref::<&str>().copied()
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

    Throwable::new(&RUST_PANIC, Some(text.to_owned()))
}

pub(crate) fn standsForJava(payload: &(dyn Any + Send)) -> bool
{
    if payload.is::<Throwable>()
    {
        return true;
    }

    let Some(text) = panicText(payload)
    else
    {
        return false;
    };

    !std::ptr::eq(fromRustPanic(text).m_class, &RUST_PANIC)
}

// java prints nothing when it throws, so only panics with no java counterpart keep the rust message
pub fn installHook()
{
    let previous = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        let location: String = info.location().map_or_else(String::new, ToString::to_string);
        let trace: Trace = Trace { m_location: location, m_frames: Backtrace::force_capture(), m_context: Vec::new() };
        THROWN.with(|slot| {
            if let Ok(mut thrown) = slot.try_borrow_mut()
            {
                *thrown = Some(trace);
            }
        });
        if !standsForJava(info.payload())
        {
            previous(info);
        }
    }));
}
