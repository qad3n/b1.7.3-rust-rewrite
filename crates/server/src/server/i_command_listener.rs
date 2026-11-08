pub trait ICommandListener
{
    fn log(&self, message: &str);

    fn getUsername(&self) -> String;
}
