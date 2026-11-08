use std::sync::OnceLock;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

// nanoTime only promises a monotonic origin, so the process start stands in for the jvm's boot clock
pub fn nanoTime() -> i64
{
    static START: OnceLock<Instant> = OnceLock::new();
    START.get_or_init(Instant::now).elapsed().as_nanos() as i64
}

pub fn currentTimeMillis() -> i64
{
    match SystemTime::now().duration_since(UNIX_EPOCH)
    {
        Ok(elapsed) => elapsed.as_millis() as i64,
        Err(error) => -(error.duration().as_millis() as i64),
    }
}

// System.out.println, java writes the platform line separator after the text
pub fn println(line: &str)
{
    use std::io::Write;
    let mut out: std::io::StdoutLock = std::io::stdout().lock();
    if let Err(error) = writeln!(out, "{line}")
    {
        eprintln!("System.out write failed: {error}");
    }
}

// System.err.println
pub fn errPrintln(line: &str)
{
    eprintln!("{line}");
}

pub fn printStackTrace(throwable: &crate::throwable::Throwable) { throwable.printStackTrace(); }

// Thread.dumpStack prints a fresh exception, the frames that follow it have no counterpart here
pub fn dumpStack()
{
    eprintln!("java.lang.Exception: Stack trace");
}
