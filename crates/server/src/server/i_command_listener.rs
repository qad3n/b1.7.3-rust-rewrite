pub trait ICommandListener
{
    fn log(&self, message: &str);

    fn getUsername(&self) -> String;

    // by type and never by name since a player may register the name console
    fn console(&self) -> bool { false }
}
