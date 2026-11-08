use crate::LWJGLException;
use sdl2::{EventPump, Sdl, VideoSubsystem};
use std::cell::RefCell;

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

fn start() -> Result<Platform, LWJGLException>
{
    let sdl: Sdl = sdl2::init().map_err(LWJGLException::new)?;
    let video: VideoSubsystem = sdl.video().map_err(LWJGLException::new)?;
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
