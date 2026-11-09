// the LWJGL release Beta 1.7.3 shipped with
const VERSION: &str = "2.4.2";

// every other desktop opens urls through xdg-open
const OPENERS: &[Opener] =
    &[Opener { m_os: "windows", m_command: "explorer" }, Opener { m_os: "macos", m_command: "open" }];
const DEFAULT_OPENER: &str = "xdg-open";

struct Opener
{
    m_os: &'static str,
    m_command: &'static str,
}

pub struct Sys;

impl Sys
{
    pub fn getVersion() -> &'static str { VERSION }

    // lwjgl hands the url to the desktop's browser launcher and reports whether that started
    pub fn openURL(url: &str) -> bool
    {
        let command: &str = OPENERS
            .iter()
            .find(|opener| opener.m_os == std::env::consts::OS)
            .map_or(DEFAULT_OPENER, |opener| opener.m_command);
        std::process::Command::new(command).arg(url).spawn().is_ok()
    }
}
