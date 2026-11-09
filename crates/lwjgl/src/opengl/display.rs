use crate::input::{keyboard, mouse};
use crate::opengl::{DisplayMode, GLContext};
use crate::{LWJGLException, platform};
use jvm::{info, warning};
use sdl2::event::{Event, WindowEvent};
use sdl2::video::{FullscreenType, GLProfile, SwapInterval, Window};
use std::cell::RefCell;

const DEFAULT_TITLE: &str = "Game";

struct DisplayState
{
    m_mode: Option<DisplayMode>,
    m_title: String,
    m_fullscreen: bool,
    m_vsync: bool,
    m_closeRequested: bool,
    m_active: bool,
    m_window: Option<Window>,
    m_context: Option<sdl2::video::GLContext>,
}

// the drawable in pixels and how many pixels one window point covers
#[derive(Clone, Copy)]
pub(crate) struct Surface
{
    pub m_width: i32,
    pub m_height: i32,
    pub m_scale: f64,
}

thread_local! {
    static STATE: RefCell<DisplayState> = RefCell::new(DisplayState {
        m_mode: None,
        m_title: DEFAULT_TITLE.to_owned(),
        m_fullscreen: false,
        m_vsync: false,
        m_closeRequested: false,
        m_active: false,
        m_window: None,
        m_context: None,
    });
}

pub struct Display;

fn sdlError(error: impl ToString) -> LWJGLException { LWJGLException::new(error.to_string()) }

fn applyFullscreen(window: &mut Window, fullscreen: bool) -> Result<(), LWJGLException>
{
    let kind: FullscreenType = if fullscreen
    {
        FullscreenType::Desktop
    }
    else
    {
        FullscreenType::Off
    };
    window.set_fullscreen(kind).map_err(sdlError)
}

// drivers may refuse an interval and keep their own so a failure only warns
fn applyVSync(video: &sdl2::VideoSubsystem, vsync: bool)
{
    let interval: SwapInterval = if vsync
    {
        SwapInterval::VSync
    }
    else
    {
        SwapInterval::Immediate
    };
    if let Err(error) = video.gl_set_swap_interval(interval)
    {
        warning!("could not set the swap interval: {error}");
    }
}

impl Display
{
    // the monitor holding the window or the primary one before it exists
    pub fn getDesktopDisplayMode() -> DisplayMode
    {
        let index: i32 = match Self::withWindow(|window| window.display_index())
        {
            Some(Ok(index)) => index,
            Some(Err(error)) =>
            {
                warning!("could not find the monitor holding the window: {error}");
                0
            }
            None => 0,
        };
        let mode: Result<sdl2::video::DisplayMode, String> =
            platform::with(|platform| platform.m_video.desktop_display_mode(index))
                .unwrap_or_else(|error| jvm::throwable::throw(&jvm::throwable::RUNTIME, &error.to_string()));
        match mode
        {
            Ok(mode) =>
            {
                DisplayMode::withFormat(mode.w, mode.h, mode.format.byte_size_per_pixel() as i32 * 8, mode.refresh_rate)
            }
            Err(error) => jvm::throwable::throw(&jvm::throwable::RUNTIME, &error),
        }
    }

    // lwjgl starts out in the desktop mode until the game picks one
    pub fn getDisplayMode() -> DisplayMode
    {
        match STATE.with(|state| state.borrow().m_mode)
        {
            Some(mode) => mode,
            None => Self::getDesktopDisplayMode(),
        }
    }

    pub fn setDisplayMode(mode: DisplayMode) -> Result<(), LWJGLException>
    {
        STATE.with(|state| {
            let mut state = state.borrow_mut();
            state.m_mode = Some(mode);
            match state.m_window.as_mut()
            {
                Some(window) => window.set_size(mode.getWidth() as u32, mode.getHeight() as u32).map_err(sdlError),
                None => Ok(()),
            }
        })
    }

    pub fn setFullscreen(fullscreen: bool) -> Result<(), LWJGLException>
    {
        STATE.with(|state| {
            let mut state = state.borrow_mut();
            state.m_fullscreen = fullscreen;
            match state.m_window.as_mut()
            {
                Some(window) => applyFullscreen(window, fullscreen),
                None => Ok(()),
            }
        })
    }

    pub fn isFullscreen() -> bool { STATE.with(|state| state.borrow().m_fullscreen) }

    pub fn setVSyncEnabled(vsync: bool)
    {
        STATE.with(|state| state.borrow_mut().m_vsync = vsync);
        if !Self::isCreated()
        {
            return;
        }

        if let Err(error) = platform::with(|platform| applyVSync(&platform.m_video, vsync))
        {
            warning!("could not reach the video subsystem: {error}");
        }
    }

    // embedding into an awt canvas has no counterpart here so the window stays top level and always resizes
    pub fn setParent<T>(_parent: Option<&T>) -> Result<(), LWJGLException> { Ok(()) }

    // the drawable size in pixels an awt parent canvas would report
    pub fn getWindowSize() -> (i32, i32)
    {
        let surface: Surface = Self::surface();
        (surface.m_width, surface.m_height)
    }

    // the display mode stands in before the window exists
    pub(crate) fn surface() -> Surface
    {
        let sizes: Option<((u32, u32), u32)> = Self::withWindow(|window| (window.drawable_size(), window.size().0));
        match sizes
        {
            Some(((width, height), points)) => Surface {
                m_width: width as i32,
                m_height: height as i32,
                // minimized windows report zero sizes
                m_scale: if width == 0 || points == 0
                {
                    1.0
                }
                else
                {
                    width as f64 / points as f64
                },
            },
            None =>
            {
                let mode: DisplayMode = Self::getDisplayMode();
                Surface { m_width: mode.getWidth(), m_height: mode.getHeight(), m_scale: 1.0 }
            }
        }
    }

    pub fn setTitle(title: &str)
    {
        STATE.with(|state| {
            let mut state = state.borrow_mut();
            state.m_title = title.to_owned();
            if let Some(window) = state.m_window.as_mut()
            {
                let _ = window.set_title(title);
            }
        });
    }

    pub fn getTitle() -> String { STATE.with(|state| state.borrow().m_title.clone()) }

    pub fn isCreated() -> bool { STATE.with(|state| state.borrow().m_window.is_some()) }

    pub fn create() -> Result<(), LWJGLException>
    {
        if Self::isCreated()
        {
            return Err(LWJGLException::new("Only one LWJGL context may be instantiated at any one time."));
        }
        let mode: DisplayMode = Self::getDisplayMode();
        let (title, fullscreen, vsync): (String, bool, bool) = STATE.with(|state| {
            let state = state.borrow();
            (state.m_title.clone(), state.m_fullscreen, state.m_vsync)
        });
        let (mut window, context): (Window, sdl2::video::GLContext) = platform::with(|platform| {
            let attributes: sdl2::video::gl_attr::GLAttr = platform.m_video.gl_attr();
            attributes.set_context_profile(GLProfile::Compatibility);
            attributes.set_context_version(2, 1);
            attributes.set_double_buffer(true);
            attributes.set_depth_size(24);
            let mut builder: sdl2::video::WindowBuilder =
                platform.m_video.window(&title, mode.getWidth() as u32, mode.getHeight() as u32);
            builder.opengl().position_centered().resizable().allow_highdpi();
            let window: Window = builder.build().map_err(sdlError)?;
            let context: sdl2::video::GLContext = window.gl_create_context().map_err(sdlError)?;
            window.gl_make_current(&context).map_err(sdlError)?;
            applyVSync(&platform.m_video, vsync);
            let video: sdl2::VideoSubsystem = platform.m_video.clone();
            GLContext::load(|name| video.gl_get_proc_address(name).cast());
            Ok::<(Window, sdl2::video::GLContext), LWJGLException>((window, context))
        })??;
        applyFullscreen(&mut window, fullscreen)?;
        STATE.with(|state| {
            let mut state = state.borrow_mut();
            state.m_window = Some(window);
            state.m_context = Some(context);
            state.m_active = true;
            state.m_closeRequested = false;
        });
        info!("Display scale {:.2}", Self::surface().m_scale);
        Ok(())
    }

    pub fn destroy()
    {
        GLContext::unload();
        STATE.with(|state| {
            let mut state = state.borrow_mut();
            state.m_context = None;
            state.m_window = None;
        });
    }

    pub fn swapBuffers() -> Result<(), LWJGLException>
    {
        STATE.with(|state| match state.borrow().m_window.as_ref()
        {
            Some(window) =>
            {
                window.gl_swap_window();
                Ok(())
            }
            None => Err(LWJGLException::new("Display not created")),
        })
    }

    // swaps, then pumps the window messages into the keyboard and mouse queues
    pub fn update()
    {
        if !Self::isCreated()
        {
            jvm::throwable::throw(&jvm::throwable::ILLEGAL_STATE, "Display not created");
        }
        let _ = Self::swapBuffers();
        Self::processMessages();
    }

    pub fn processMessages()
    {
        let surface: Surface = Self::surface();
        let events: Vec<Event> = platform::with(|platform| platform.m_events.poll_iter().collect()).unwrap_or_default();
        for event in events
        {
            match &event
            {
                Event::Quit { .. } | Event::Window { win_event: WindowEvent::Close, .. } =>
                {
                    STATE.with(|state| state.borrow_mut().m_closeRequested = true);
                }
                Event::Window { win_event: WindowEvent::FocusGained, .. } =>
                {
                    STATE.with(|state| state.borrow_mut().m_active = true);
                }
                Event::Window { win_event: WindowEvent::FocusLost, .. } =>
                {
                    STATE.with(|state| state.borrow_mut().m_active = false);
                    // keys and buttons let go over another window never send their release here
                    keyboard::release();
                    mouse::release();
                }
                _ =>
                {
                    keyboard::handleEvent(&event);
                    mouse::handleEvent(&event, surface);
                }
            }
        }
    }

    // reading the request clears it, as on lwjgl's linux display
    pub fn isCloseRequested() -> bool
    {
        STATE.with(|state| std::mem::replace(&mut state.borrow_mut().m_closeRequested, false))
    }

    pub fn isActive() -> bool { STATE.with(|state| state.borrow().m_active) }

    pub(crate) fn withWindow<R>(body: impl FnOnce(&mut Window) -> R) -> Option<R>
    {
        STATE.with(|state| state.borrow_mut().m_window.as_mut().map(body))
    }
}
