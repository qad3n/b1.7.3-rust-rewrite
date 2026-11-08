use crate::argo::ThingWithPosition;
use jvm::throwable::{EXCEPTION, Throwable, ThrowableClass};

pub static INVALID_SYNTAX_EXCEPTION: ThrowableClass =
    ThrowableClass { m_name: "net.minecraft.src.J_InvalidSyntaxException", m_parent: Some(&EXCEPTION) };

pub struct InvalidSyntaxException;

impl InvalidSyntaxException
{
    pub fn new(s: &str, thingWithPosition: &dyn ThingWithPosition) -> Throwable
    {
        Throwable::new(
            &INVALID_SYNTAX_EXCEPTION,
            Some(format!("At line {}, column {}:  {s}", thingWithPosition.getRow(), thingWithPosition.getColumn())),
        )
    }
}
