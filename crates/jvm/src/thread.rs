use crate::string::JString;
use std::cell::RefCell;
use std::sync::atomic::{AtomicI32, Ordering};
use std::thread::JoinHandle;

static THREAD_INIT_NUMBER: AtomicI32 = AtomicI32::new(0);

thread_local! {
    static NAME: RefCell<Option<JString>> = const { RefCell::new(None) };
}

// threads built without a name get Thread-N from one counter like java.lang.Thread
pub fn nextName() -> JString
{
    let number: i32 = THREAD_INIT_NUMBER.fetch_add(1, Ordering::Relaxed);
    JString::from(format!("Thread-{number}").as_str())
}

pub fn currentName() -> JString
{
    if let Some(name) = NAME.with(|name| name.borrow().clone())
    {
        return name;
    }

    match std::thread::current().name()
    {
        Some(name) => JString::from(name),
        None => JString::from("main"),
    }
}

pub fn setCurrentName(name: JString) { NAME.with(|current| *current.borrow_mut() = Some(name)); }

pub fn start<F>(name: JString, body: F) -> std::io::Result<JoinHandle<()>>
where F: FnOnce() + Send + 'static
{
    let label: String = name.to_string();
    std::thread::Builder::new().name(label).spawn(move || {
        setCurrentName(name);
        body();
    })
}
