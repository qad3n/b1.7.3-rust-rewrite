mod legacy;
mod location;
mod pack;
mod source;
mod stack;

pub(crate) use location::{Location, MINECRAFT};
pub(crate) use pack::Pack;
pub use source::Source;
pub(crate) use stack::Stack;

thread_local!
{
    static STACK: Stack = vanilla();
}

// the user's extracted b1.7.3 jar read in place
fn vanilla() -> Stack { Stack::new(vec![Pack::new(Source::Folder(crate::port::assetRoot()))]) }

pub(crate) fn read(location: &Location) -> Option<Vec<u8>> { STACK.with(|stack| stack.read(location)) }

// a jar path such as /lang/en_US.lang
pub fn readBeta(path: &str) -> Option<Vec<u8>> { read(&legacy::location(path)) }
