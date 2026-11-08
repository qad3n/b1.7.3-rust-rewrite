use crate::opengl::Display;
use crate::{LWJGLException, platform};
use sdl2::event::Event;
use sdl2::mouse::MouseButton;
use std::cell::RefCell;
use std::collections::VecDeque;

// lwjgl numbers buttons left 0, right 1, middle 2
const BUTTONS: [(MouseButton, i32); 5] = [
    (MouseButton::Left, 0),
    (MouseButton::Right, 1),
    (MouseButton::Middle, 2),
    (MouseButton::X1, 3),
    (MouseButton::X2, 4),
];

const BUTTON_COUNT: usize = 8;

// windows reports 120 per wheel notch, the game only reads the sign
const WHEEL_DELTA: i32 = 120;

#[derive(Clone, Copy, Default)]
struct MouseEvent
{
    m_button: i32,
    m_state: bool,
    m_dx: i32,
    m_dy: i32,
    m_dwheel: i32,
    m_x: i32,
    m_y: i32,
    m_nanos: i64,
}

#[derive(Default)]
struct MouseState
{
    m_created: bool,
    m_grabbed: bool,
    m_x: i32,
    m_y: i32,
    m_dx: i32,
    m_dy: i32,
    m_dwheel: i32,
    m_buttons: [bool; BUTTON_COUNT],
    m_queue: VecDeque<MouseEvent>,
    m_current: MouseEvent,
}

thread_local! {
    static STATE: RefCell<MouseState> = RefCell::new(MouseState::default());
}

fn lwjglButton(button: MouseButton) -> i32
{
    BUTTONS.iter().find(|(sdlButton, _)| *sdlButton == button).map_or(-1, |(_, index)| *index)
}

fn queue(state: &mut MouseState, button: i32, buttonState: bool, dx: i32, dy: i32, dwheel: i32)
{
    let event: MouseEvent = MouseEvent {
        m_button: button,
        m_state: buttonState,
        m_dx: dx,
        m_dy: dy,
        m_dwheel: dwheel,
        m_x: state.m_x,
        m_y: state.m_y,
        m_nanos: jvm::system::nanoTime(),
    };
    state.m_queue.push_back(event);
}

// positions have their origin at the bottom left like gl, deltas grow upwards
pub(crate) fn handleEvent(event: &Event, height: i32)
{
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        if !state.m_created
        {
            return;
        }
        let width: i32 = Display::getDisplayMode().getWidth();
        match *event
        {
            Event::MouseMotion { x, y, xrel, yrel, .. } =>
            {
                if state.m_grabbed
                {
                    state.m_x = (state.m_x + xrel).clamp(0, width - 1);
                    state.m_y = (state.m_y - yrel).clamp(0, height - 1);
                }
                else
                {
                    state.m_x = x;
                    state.m_y = height - 1 - y;
                }
                state.m_dx += xrel;
                state.m_dy -= yrel;
                queue(&mut state, -1, false, xrel, -yrel, 0);
            }
            Event::MouseButtonDown { mouse_btn, .. } | Event::MouseButtonUp { mouse_btn, .. } =>
            {
                let button: i32 = lwjglButton(mouse_btn);
                if button < 0
                {
                    return;
                }
                let pressed: bool = matches!(event, Event::MouseButtonDown { .. });
                state.m_buttons[button as usize] = pressed;
                queue(&mut state, button, pressed, 0, 0, 0);
            }
            Event::MouseWheel { y, .. } =>
            {
                let dwheel: i32 = y * WHEEL_DELTA;
                state.m_dwheel += dwheel;
                queue(&mut state, -1, false, 0, 0, dwheel);
            }
            _ =>
            {}
        }
    });
}

pub struct Mouse;

impl Mouse
{
    pub fn create() -> Result<(), LWJGLException>
    {
        STATE.with(|state| {
            let mut state = state.borrow_mut();
            *state = MouseState::default();
            state.m_created = true;
        });
        Ok(())
    }

    pub fn destroy()
    {
        Self::setGrabbed(false);
        STATE.with(|state| *state.borrow_mut() = MouseState::default());
    }

    pub fn isCreated() -> bool { STATE.with(|state| state.borrow().m_created) }

    pub fn next() -> bool
    {
        STATE.with(|state| {
            let mut state = state.borrow_mut();
            match state.m_queue.pop_front()
            {
                Some(event) =>
                {
                    state.m_current = event;
                    true
                }
                None => false,
            }
        })
    }

    pub fn getEventButton() -> i32 { STATE.with(|state| state.borrow().m_current.m_button) }

    pub fn getEventButtonState() -> bool { STATE.with(|state| state.borrow().m_current.m_state) }

    pub fn getEventDX() -> i32 { STATE.with(|state| state.borrow().m_current.m_dx) }

    pub fn getEventDY() -> i32 { STATE.with(|state| state.borrow().m_current.m_dy) }

    pub fn getEventX() -> i32 { STATE.with(|state| state.borrow().m_current.m_x) }

    pub fn getEventY() -> i32 { STATE.with(|state| state.borrow().m_current.m_y) }

    pub fn getEventDWheel() -> i32 { STATE.with(|state| state.borrow().m_current.m_dwheel) }

    pub fn getEventNanoseconds() -> i64 { STATE.with(|state| state.borrow().m_current.m_nanos) }

    pub fn getX() -> i32 { STATE.with(|state| state.borrow().m_x) }

    pub fn getY() -> i32 { STATE.with(|state| state.borrow().m_y) }

    // the deltas accumulate since the previous call
    pub fn getDX() -> i32 { STATE.with(|state| std::mem::take(&mut state.borrow_mut().m_dx)) }

    pub fn getDY() -> i32 { STATE.with(|state| std::mem::take(&mut state.borrow_mut().m_dy)) }

    pub fn getDWheel() -> i32 { STATE.with(|state| std::mem::take(&mut state.borrow_mut().m_dwheel)) }

    pub fn isButtonDown(button: i32) -> bool
    {
        STATE.with(|state| state.borrow().m_buttons.get(button as usize).copied().unwrap_or(false))
    }

    pub fn getButtonCount() -> i32 { BUTTONS.len() as i32 }

    pub fn isGrabbed() -> bool { STATE.with(|state| state.borrow().m_grabbed) }

    pub fn setGrabbed(grab: bool)
    {
        STATE.with(|state| state.borrow_mut().m_grabbed = grab);
        let _ = platform::with(|platform| platform.m_sdl.mouse().set_relative_mouse_mode(grab));
    }

    pub fn setCursorPosition(x: i32, y: i32)
    {
        STATE.with(|state| {
            let mut state = state.borrow_mut();
            state.m_x = x;
            state.m_y = y;
        });
        let height: i32 = Display::getDisplayMode().getHeight();
        let _ = platform::with(|platform| {
            Display::withWindow(|window| platform.m_sdl.mouse().warp_mouse_in_window(window, x, height - 1 - y))
        });
    }
}
