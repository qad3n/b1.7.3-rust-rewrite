// the LWJGL release Beta 1.7.3 shipped with
const VERSION: &str = "2.4.2";

pub struct Sys;

impl Sys
{
    pub fn getVersion() -> &'static str { VERSION }

    // lwjgl hands the url to the desktop's browser launcher and reports whether that started
    pub fn openURL(url: &str) -> bool { std::process::Command::new("xdg-open").arg(url).spawn().is_ok() }
}
