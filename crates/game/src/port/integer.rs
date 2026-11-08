use jvm::string::JString;
use jvm::throwable::Throwable;

pub fn parseInt(string: &str) -> Result<i32, Throwable> { jvm::integer::parseInt(&JString::from(string)) }
