use crate::input::{keyboard, mouse};
use crate::opengl::{DisplayMode, GLContext};
use crate::{LWJGLException, platform};
use sdl2::event::{Event, WindowEvent};
use sdl2::video::{FullscreenType, GLProfile, SwapInterval, Window};
use std::cell::RefCell;

const DEFAULT_TITLE: &str = "Game";

struct DisplayState
{
    m_mode: Option<DisplayMode>,
    m_title: String,
    m_fullscreen: bool,
    m_parented: bool,
    m_closeRequested: bool,
    m_active: bool,
    m_window: Option<Window>,
    m_context: Option<sdl2::video::GLContext>,
}

thread_local! {
    static STATE: RefCell<DisplayState> = RefCell::new(DisplayState {
        m_mode: None,
        m_title: DEFAULT_TITLE.to_owned(),
        m_fullscreen: false,
        m_parented: false,
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

impl Display
{
    pub fn getDesktopDisplayMode() -> DisplayMode
    {
        let mode: Result<sdl2::video::DisplayMode, String> =
            platform::with(|platform| platform.m_video.desktop_display_mode(0))
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

    // embedding into an awt canvas has no counterpart here, the window stays top level and resizes like the frame
    pub fn setParent<T>(parent: Option<&T>) -> Result<(), LWJGLException>
    {
        STATE.with(|state| state.borrow_mut().m_parented = parent.is_some());
        Ok(())
    }

    // the drawable size an awt parent canvas would report, the display mode before the window exists
    pub fn getWindowSize() -> (i32, i32)
    {
        match Self::withWindow(|window| window.size())
        {
            Some((width, height)) => (width as i32, height as i32),
            None => (Self::getDisplayMode().getWidth(), Self::getDisplayMode().getHeight()),
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
        let (title, fullscreen, parented): (String, bool, bool) = STATE.with(|state| {
            let state = state.borrow();
            (state.m_title.clone(), state.m_fullscreen, state.m_parented)
        });
        let (mut window, context): (Window, sdl2::video::GLContext) = platform::with(|platform| {
            let attributes: sdl2::video::gl_attr::GLAttr = platform.m_video.gl_attr();
            attributes.set_context_profile(GLProfile::Compatibility);
            attributes.set_context_version(2, 1);
            attributes.set_double_buffer(true);
            attributes.set_depth_size(24);
            let mut builder: sdl2::video::WindowBuilder =
                platform.m_video.window(&title, mode.getWidth() as u32, mode.getHeight() as u32);
            builder.opengl().position_centered();
            if parented
            {
                builder.resizable();
            }
            let window: Window = builder.build().map_err(sdlError)?;
            let context: sdl2::video::GLContext = window.gl_create_context().map_err(sdlError)?;
            window.gl_make_current(&context).map_err(sdlError)?;
            let _ = platform.m_video.gl_set_swap_interval(SwapInterval::Immediate);
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
        let height: i32 = Self::getWindowSize().1;
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
                }
                _ =>
                {
                    keyboard::handleEvent(&event);
                    mouse::handleEvent(&event, height);
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
