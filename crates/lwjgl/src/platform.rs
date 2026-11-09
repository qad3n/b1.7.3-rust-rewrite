use crate::LWJGLException;
use jvm::{debug, info};
use sdl2::{EventPump, Sdl, VideoSubsystem};
use std::cell::RefCell;

// compositor window rules and desktop files match on this name
const APP_ID: &str = "beta173";

// sdl reads these at startup and the same names set in the environment win over them
const HINTS: &[Hint] = &[
    Hint { m_name: "SDL_APP_ID", m_value: APP_ID },
    Hint { m_name: "SDL_VIDEO_X11_WMCLASS", m_value: APP_ID },
    Hint { m_name: "SDL_VIDEO_WAYLAND_WMCLASS", m_value: APP_ID },
    // windows then counts window points like other desktops and gives the extra pixels to the drawable
    Hint { m_name: "SDL_WINDOWS_DPI_SCALING", m_value: "1" },
    // borderless fullscreen stays up while another window has focus
    Hint { m_name: "SDL_VIDEO_MINIMIZE_ON_FOCUS_LOSS", m_value: "0" },
];

// sdl2 and sdl3 spell the driver hint differently
const DRIVER_HINTS: [&str; 2] = ["SDL_VIDEODRIVER", "SDL_VIDEO_DRIVER"];
const WAYLAND_DRIVERS: &str = "wayland,x11";

struct Hint
{
    m_name: &'static str,
    m_value: &'static str,
}

// the sdl handles lwjgl keeps behind its static classes, all on the main thread
pub(crate) struct Platform
{
    pub m_sdl: Sdl,
    pub m_video: VideoSubsystem,
    pub m_events: EventPump,
}

thread_local! {
    static PLATFORM: RefCell<Option<Platform>> = const { RefCell::new(None) };
}

fn hint(name: &str, value: &str)
{
    if !sdl2::hint::set(name, value)
    {
        debug!("SDL hint {name} left to the environment");
    }
}

// native wayland scales sharply and locks the pointer without going through xwayland
fn preferWayland()
{
    if std::env::var_os("WAYLAND_DISPLAY").is_none()
    {
        return;
    }

    for name in DRIVER_HINTS
    {
        hint(name, WAYLAND_DRIVERS);
    }
}

fn start() -> Result<Platform, LWJGLException>
{
    for row in HINTS
    {
        hint(row.m_name, row.m_value);
    }
    preferWayland();

    let sdl: Sdl = sdl2::init().map_err(LWJGLException::new)?;
    let video: VideoSubsystem = sdl.video().map_err(LWJGLException::new)?;
    info!("Video driver {}", video.current_video_driver());

    let events: EventPump = sdl.event_pump().map_err(LWJGLException::new)?;
    Ok(Platform { m_sdl: sdl, m_video: video, m_events: events })
}

// runs body with sdl started, the first call starts it like lwjgl's Display class initializer
pub(crate) fn with<R>(body: impl FnOnce(&mut Platform) -> R) -> Result<R, LWJGLException>
{
    PLATFORM.with(|platform| {
        let mut platform = platform.borrow_mut();
        if platform.is_none()
        {
            *platform = Some(start()?);
        }
        Ok(body(platform.as_mut().expect("platform started above")))
    })
}
