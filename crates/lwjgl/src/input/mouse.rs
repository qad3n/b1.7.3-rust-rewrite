use crate::opengl::{Display, Surface};
use crate::{LWJGLException, platform};
use jvm::warning;
use sdl2::event::{Event, EventType};
use sdl2::mouse::{MouseButton, MouseUtil};
use std::cell::{RefCell, RefMut};
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
    // touchpads and smooth wheels scroll in fractions of a notch
    m_notches: f32,
    m_buttons: [bool; BUTTON_COUNT],
    m_queue: VecDeque<MouseEvent>,
    m_current: MouseEvent,
}

thread_local!
{
    static STATE: RefCell<MouseState> = RefCell::new(MouseState::default());
}

fn lwjglButton(button: MouseButton) -> i32
{
    BUTTONS.iter().find(|(sdlButton, _)| *sdlButton == button).map_or(-1, |(_, index)| *index)
}

fn queue(state: &mut MouseState, button: i32, buttonState: bool, dx: i32, dy: i32, dwheel: i32)
{
    let event: MouseEvent = MouseEvent
    {
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

// lwjgl positions stay inside the drawable
fn limit(value: i32, size: i32) -> i32 { value.clamp(0, (size - 1).max(0)) }

// moves the cursor from lwjgl pixels at the bottom left to window points at the top left
fn warp(x: i32, y: i32)
{
    let surface: Surface = Display::surface();
    let pointX: i32 = (x as f64 / surface.m_scale) as i32;
    let pointY: i32 = ((surface.m_height - 1 - y) as f64 / surface.m_scale) as i32;

    let warped: Result<Option<()>, LWJGLException> = platform::with(|platform|
    {
        Display::withWindow(|window| platform.m_sdl.mouse().warp_mouse_in_window(window, pointX, pointY))
    });
    if let Err(error) = warped
    {
        warning!("could not move the cursor: {error}");
    }
}

// positions are drawable pixels with their origin at the bottom left like gl, deltas grow upwards
pub(crate) fn handleEvent(event: &Event, surface: Surface)
{
    STATE.with(|state|
    {
        let mut state: RefMut<crate::input::mouse::MouseState> = state.borrow_mut();
        if !state.m_created
        {
            return;
        }

        match *event
        {
            Event::MouseMotion { x, y, xrel, yrel, .. } =>
            {
                if state.m_grabbed
                {
                    state.m_x = limit(state.m_x + xrel, surface.m_width);
                    state.m_y = limit(state.m_y - yrel, surface.m_height);
                }
                else
                {
                    state.m_x = limit((x as f64 * surface.m_scale) as i32, surface.m_width);
                    state.m_y = limit(surface.m_height - 1 - (y as f64 * surface.m_scale) as i32, surface.m_height);
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

            Event::MouseWheel { precise_y, .. } =>
            {
                state.m_notches += precise_y;
                let notches: f32 = state.m_notches.trunc();
                if notches == 0.0
                {
                    return;
                }

                state.m_notches -= notches;

                let dwheel: i32 = notches as i32 * WHEEL_DELTA;
                state.m_dwheel += dwheel;
                queue(&mut state, -1, false, 0, 0, dwheel);
            }

            _ =>
            {}
        }
    });
}

pub(crate) fn release()
{
    STATE.with(|state|
    {
        let mut state: RefMut<crate::input::mouse::MouseState> = state.borrow_mut();

        for (_, button) in BUTTONS
        {
            if !state.m_buttons[button as usize]
            {
                continue;
            }

            state.m_buttons[button as usize] = false;
            queue(&mut state, button, false, 0, 0, 0);
        }
    });
}

pub struct Mouse;

impl Mouse
{
    pub fn create() -> Result<(), LWJGLException>
    {
        STATE.with(|state|
        {
            let mut state: RefMut<crate::input::mouse::MouseState> = state.borrow_mut();
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
        STATE.with(|state|
        {
            let mut state: RefMut<crate::input::mouse::MouseState> = state.borrow_mut();
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
        let switched: Result<bool, LWJGLException> = platform::with(|platform|
        {
            let mouse: MouseUtil = platform.m_sdl.mouse();
            mouse.set_relative_mouse_mode(grab);

            // motion queued in the old mode would jolt the camera or the cursor
            match platform.m_sdl.event()
            {
                Ok(events) => events.flush_event(EventType::MouseMotion),
                Err(error) => warning!("could not drop queued mouse motion: {error}"),
            }

            mouse.relative_mouse_mode()
        });
        let grabbed: bool = match switched
        {
            Ok(grabbed) => grabbed,

            Err(error) =>
            {
                warning!("could not switch the mouse grab: {error}");
                false
            }
        };
        if grab && !grabbed
        {
            warning!("the window system refused to grab the mouse");
        }

        // lwjgl drops the deltas gathered in the old mode
        let (x, y): (i32, i32) = STATE.with(|state|
        {
            let mut state: RefMut<crate::input::mouse::MouseState> = state.borrow_mut();
            state.m_grabbed = grabbed;
            state.m_dx = 0;
            state.m_dy = 0;
            state.m_dwheel = 0;
            (state.m_x, state.m_y)
        });

        if grabbed
        {
            return;
        }

        // the released cursor shows up where the game last saw it
        warp(x, y);
    }

    pub fn setCursorPosition(x: i32, y: i32)
    {
        STATE.with(|state|
        {
            let mut state: RefMut<crate::input::mouse::MouseState> = state.borrow_mut();
            state.m_x = x;
            state.m_y = y;
        });

        warp(x, y);
    }
}
