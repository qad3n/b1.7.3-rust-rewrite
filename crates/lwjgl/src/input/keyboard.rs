use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::cell::RefCell;
use std::collections::VecDeque;

macro_rules! keys {
    ($($name:ident = $code:expr,)*) => {
        impl Keyboard
        {
            $(pub const $name: i32 = $code;)*
        }

        // Keyboard.keyName comes from the KEY_ field names
        const KEY_NAMES: &[(&str, i32)] = &[$((stringify!($name), $code),)*];
    };
}

pub struct Keyboard;

impl Keyboard
{
    pub const CHAR_NONE: u16 = 0;
    pub const KEYBOARD_SIZE: i32 = 256;
}

keys! {
    KEY_NONE = 0x00,
    KEY_ESCAPE = 0x01,
    KEY_1 = 0x02,
    KEY_2 = 0x03,
    KEY_3 = 0x04,
    KEY_4 = 0x05,
    KEY_5 = 0x06,
    KEY_6 = 0x07,
    KEY_7 = 0x08,
    KEY_8 = 0x09,
    KEY_9 = 0x0A,
    KEY_0 = 0x0B,
    KEY_MINUS = 0x0C,
    KEY_EQUALS = 0x0D,
    KEY_BACK = 0x0E,
    KEY_TAB = 0x0F,
    KEY_Q = 0x10,
    KEY_W = 0x11,
    KEY_E = 0x12,
    KEY_R = 0x13,
    KEY_T = 0x14,
    KEY_Y = 0x15,
    KEY_U = 0x16,
    KEY_I = 0x17,
    KEY_O = 0x18,
    KEY_P = 0x19,
    KEY_LBRACKET = 0x1A,
    KEY_RBRACKET = 0x1B,
    KEY_RETURN = 0x1C,
    KEY_LCONTROL = 0x1D,
    KEY_A = 0x1E,
    KEY_S = 0x1F,
    KEY_D = 0x20,
    KEY_F = 0x21,
    KEY_G = 0x22,
    KEY_H = 0x23,
    KEY_J = 0x24,
    KEY_K = 0x25,
    KEY_L = 0x26,
    KEY_SEMICOLON = 0x27,
    KEY_APOSTROPHE = 0x28,
    KEY_GRAVE = 0x29,
    KEY_LSHIFT = 0x2A,
    KEY_BACKSLASH = 0x2B,
    KEY_Z = 0x2C,
    KEY_X = 0x2D,
    KEY_C = 0x2E,
    KEY_V = 0x2F,
    KEY_B = 0x30,
    KEY_N = 0x31,
    KEY_M = 0x32,
    KEY_COMMA = 0x33,
    KEY_PERIOD = 0x34,
    KEY_SLASH = 0x35,
    KEY_RSHIFT = 0x36,
    KEY_MULTIPLY = 0x37,
    KEY_LMENU = 0x38,
    KEY_SPACE = 0x39,
    KEY_CAPITAL = 0x3A,
    KEY_F1 = 0x3B,
    KEY_F2 = 0x3C,
    KEY_F3 = 0x3D,
    KEY_F4 = 0x3E,
    KEY_F5 = 0x3F,
    KEY_F6 = 0x40,
    KEY_F7 = 0x41,
    KEY_F8 = 0x42,
    KEY_F9 = 0x43,
    KEY_F10 = 0x44,
    KEY_NUMLOCK = 0x45,
    KEY_SCROLL = 0x46,
    KEY_NUMPAD7 = 0x47,
    KEY_NUMPAD8 = 0x48,
    KEY_NUMPAD9 = 0x49,
    KEY_SUBTRACT = 0x4A,
    KEY_NUMPAD4 = 0x4B,
    KEY_NUMPAD5 = 0x4C,
    KEY_NUMPAD6 = 0x4D,
    KEY_ADD = 0x4E,
    KEY_NUMPAD1 = 0x4F,
    KEY_NUMPAD2 = 0x50,
    KEY_NUMPAD3 = 0x51,
    KEY_NUMPAD0 = 0x52,
    KEY_DECIMAL = 0x53,
    KEY_F11 = 0x57,
    KEY_F12 = 0x58,
    KEY_F13 = 0x64,
    KEY_F14 = 0x65,
    KEY_F15 = 0x66,
    KEY_KANA = 0x70,
    KEY_CONVERT = 0x79,
    KEY_NOCONVERT = 0x7B,
    KEY_YEN = 0x7D,
    KEY_NUMPADEQUALS = 0x8D,
    KEY_CIRCUMFLEX = 0x90,
    KEY_AT = 0x91,
    KEY_COLON = 0x92,
    KEY_UNDERLINE = 0x93,
    KEY_KANJI = 0x94,
    KEY_STOP = 0x95,
    KEY_AX = 0x96,
    KEY_UNLABELED = 0x97,
    KEY_NUMPADENTER = 0x9C,
    KEY_RCONTROL = 0x9D,
    KEY_NUMPADCOMMA = 0xB3,
    KEY_DIVIDE = 0xB5,
    KEY_SYSRQ = 0xB7,
    KEY_RMENU = 0xB8,
    KEY_PAUSE = 0xC5,
    KEY_HOME = 0xC7,
    KEY_UP = 0xC8,
    KEY_PRIOR = 0xC9,
    KEY_LEFT = 0xCB,
    KEY_RIGHT = 0xCD,
    KEY_END = 0xCF,
    KEY_DOWN = 0xD0,
    KEY_NEXT = 0xD1,
    KEY_INSERT = 0xD2,
    KEY_DELETE = 0xD3,
    KEY_LWIN = 0xDB,
    KEY_RWIN = 0xDC,
    KEY_APPS = 0xDD,
    KEY_POWER = 0xDE,
    KEY_SLEEP = 0xDF,
}

// layout aware like lwjgl's keysym translation on linux
const SDL_KEYS: &[(Keycode, i32)] = &[
    (Keycode::Escape, Keyboard::KEY_ESCAPE),
    (Keycode::Num1, Keyboard::KEY_1),
    (Keycode::Num2, Keyboard::KEY_2),
    (Keycode::Num3, Keyboard::KEY_3),
    (Keycode::Num4, Keyboard::KEY_4),
    (Keycode::Num5, Keyboard::KEY_5),
    (Keycode::Num6, Keyboard::KEY_6),
    (Keycode::Num7, Keyboard::KEY_7),
    (Keycode::Num8, Keyboard::KEY_8),
    (Keycode::Num9, Keyboard::KEY_9),
    (Keycode::Num0, Keyboard::KEY_0),
    (Keycode::Minus, Keyboard::KEY_MINUS),
    (Keycode::Equals, Keyboard::KEY_EQUALS),
    (Keycode::Backspace, Keyboard::KEY_BACK),
    (Keycode::Tab, Keyboard::KEY_TAB),
    (Keycode::LeftBracket, Keyboard::KEY_LBRACKET),
    (Keycode::RightBracket, Keyboard::KEY_RBRACKET),
    (Keycode::Return, Keyboard::KEY_RETURN),
    (Keycode::LCtrl, Keyboard::KEY_LCONTROL),
    (Keycode::Semicolon, Keyboard::KEY_SEMICOLON),
    (Keycode::Quote, Keyboard::KEY_APOSTROPHE),
    (Keycode::Backquote, Keyboard::KEY_GRAVE),
    (Keycode::LShift, Keyboard::KEY_LSHIFT),
    (Keycode::Backslash, Keyboard::KEY_BACKSLASH),
    (Keycode::Comma, Keyboard::KEY_COMMA),
    (Keycode::Period, Keyboard::KEY_PERIOD),
    (Keycode::Slash, Keyboard::KEY_SLASH),
    (Keycode::RShift, Keyboard::KEY_RSHIFT),
    (Keycode::KpMultiply, Keyboard::KEY_MULTIPLY),
    (Keycode::LAlt, Keyboard::KEY_LMENU),
    (Keycode::Space, Keyboard::KEY_SPACE),
    (Keycode::CapsLock, Keyboard::KEY_CAPITAL),
    (Keycode::NumLockClear, Keyboard::KEY_NUMLOCK),
    (Keycode::ScrollLock, Keyboard::KEY_SCROLL),
    (Keycode::Kp7, Keyboard::KEY_NUMPAD7),
    (Keycode::Kp8, Keyboard::KEY_NUMPAD8),
    (Keycode::Kp9, Keyboard::KEY_NUMPAD9),
    (Keycode::KpMinus, Keyboard::KEY_SUBTRACT),
    (Keycode::Kp4, Keyboard::KEY_NUMPAD4),
    (Keycode::Kp5, Keyboard::KEY_NUMPAD5),
    (Keycode::Kp6, Keyboard::KEY_NUMPAD6),
    (Keycode::KpPlus, Keyboard::KEY_ADD),
    (Keycode::Kp1, Keyboard::KEY_NUMPAD1),
    (Keycode::Kp2, Keyboard::KEY_NUMPAD2),
    (Keycode::Kp3, Keyboard::KEY_NUMPAD3),
    (Keycode::Kp0, Keyboard::KEY_NUMPAD0),
    (Keycode::KpPeriod, Keyboard::KEY_DECIMAL),
    (Keycode::KpEquals, Keyboard::KEY_NUMPADEQUALS),
    (Keycode::KpEnter, Keyboard::KEY_NUMPADENTER),
    (Keycode::RCtrl, Keyboard::KEY_RCONTROL),
    (Keycode::KpComma, Keyboard::KEY_NUMPADCOMMA),
    (Keycode::KpDivide, Keyboard::KEY_DIVIDE),
    (Keycode::Sysreq, Keyboard::KEY_SYSRQ),
    (Keycode::PrintScreen, Keyboard::KEY_SYSRQ),
    (Keycode::RAlt, Keyboard::KEY_RMENU),
    (Keycode::Pause, Keyboard::KEY_PAUSE),
    (Keycode::Home, Keyboard::KEY_HOME),
    (Keycode::Up, Keyboard::KEY_UP),
    (Keycode::PageUp, Keyboard::KEY_PRIOR),
    (Keycode::Left, Keyboard::KEY_LEFT),
    (Keycode::Right, Keyboard::KEY_RIGHT),
    (Keycode::End, Keyboard::KEY_END),
    (Keycode::Down, Keyboard::KEY_DOWN),
    (Keycode::PageDown, Keyboard::KEY_NEXT),
    (Keycode::Insert, Keyboard::KEY_INSERT),
    (Keycode::Delete, Keyboard::KEY_DELETE),
    (Keycode::LGui, Keyboard::KEY_LWIN),
    (Keycode::RGui, Keyboard::KEY_RWIN),
    (Keycode::Application, Keyboard::KEY_APPS),
    (Keycode::Power, Keyboard::KEY_POWER),
    (Keycode::Sleep, Keyboard::KEY_SLEEP),
    (Keycode::Stop, Keyboard::KEY_STOP),
    (Keycode::A, Keyboard::KEY_A),
    (Keycode::B, Keyboard::KEY_B),
    (Keycode::C, Keyboard::KEY_C),
    (Keycode::D, Keyboard::KEY_D),
    (Keycode::E, Keyboard::KEY_E),
    (Keycode::F, Keyboard::KEY_F),
    (Keycode::G, Keyboard::KEY_G),
    (Keycode::H, Keyboard::KEY_H),
    (Keycode::I, Keyboard::KEY_I),
    (Keycode::J, Keyboard::KEY_J),
    (Keycode::K, Keyboard::KEY_K),
    (Keycode::L, Keyboard::KEY_L),
    (Keycode::M, Keyboard::KEY_M),
    (Keycode::N, Keyboard::KEY_N),
    (Keycode::O, Keyboard::KEY_O),
    (Keycode::P, Keyboard::KEY_P),
    (Keycode::Q, Keyboard::KEY_Q),
    (Keycode::R, Keyboard::KEY_R),
    (Keycode::S, Keyboard::KEY_S),
    (Keycode::T, Keyboard::KEY_T),
    (Keycode::U, Keyboard::KEY_U),
    (Keycode::V, Keyboard::KEY_V),
    (Keycode::W, Keyboard::KEY_W),
    (Keycode::X, Keyboard::KEY_X),
    (Keycode::Y, Keyboard::KEY_Y),
    (Keycode::Z, Keyboard::KEY_Z),
    (Keycode::F1, Keyboard::KEY_F1),
    (Keycode::F2, Keyboard::KEY_F2),
    (Keycode::F3, Keyboard::KEY_F3),
    (Keycode::F4, Keyboard::KEY_F4),
    (Keycode::F5, Keyboard::KEY_F5),
    (Keycode::F6, Keyboard::KEY_F6),
    (Keycode::F7, Keyboard::KEY_F7),
    (Keycode::F8, Keyboard::KEY_F8),
    (Keycode::F9, Keyboard::KEY_F9),
    (Keycode::F10, Keyboard::KEY_F10),
    (Keycode::F11, Keyboard::KEY_F11),
    (Keycode::F12, Keyboard::KEY_F12),
    (Keycode::F13, Keyboard::KEY_F13),
    (Keycode::F14, Keyboard::KEY_F14),
    (Keycode::F15, Keyboard::KEY_F15),
];

// control keys lwjgl reports with their ascii character
const CONTROL_CHARACTERS: [(i32, u16); 5] = [
    (Keyboard::KEY_BACK, 8),
    (Keyboard::KEY_TAB, 9),
    (Keyboard::KEY_RETURN, 13),
    (Keyboard::KEY_ESCAPE, 27),
    (Keyboard::KEY_DELETE, 127),
];

#[derive(Clone, Copy, Default)]
struct KeyEvent
{
    m_key: i32,
    m_state: bool,
    m_character: u16,
    m_nanos: i64,
    m_repeat: bool,
    m_hasText: bool,
}

#[derive(Default)]
struct KeyboardState
{
    m_created: bool,
    m_repeatEnabled: bool,
    m_keyDown: Vec<bool>,
    m_queue: VecDeque<KeyEvent>,
    m_current: KeyEvent,
    m_droppedRepeat: bool,
}

thread_local! {
    static STATE: RefCell<KeyboardState> = RefCell::new(KeyboardState::default());
}

fn lwjglKey(keycode: Keycode) -> i32
{
    SDL_KEYS.iter().find(|(sdlKey, _)| *sdlKey == keycode).map_or(Keyboard::KEY_NONE, |(_, key)| *key)
}

fn controlCharacter(key: i32) -> u16
{
    CONTROL_CHARACTERS
        .iter()
        .find(|(control, _)| *control == key)
        .map_or(Keyboard::CHAR_NONE, |(_, character)| *character)
}

// typed text arrives after its key press, lwjgl reports both in one event and extra characters as key 0 events
fn attachText(state: &mut KeyboardState, text: &str, nanos: i64)
{
    for character in text.encode_utf16()
    {
        match state.m_queue.back_mut().filter(|last| last.m_state && !last.m_hasText)
        {
            Some(last) =>
            {
                last.m_character = character;
                last.m_hasText = true;
            }
            None => state.m_queue.push_back(KeyEvent {
                m_key: Keyboard::KEY_NONE,
                m_state: true,
                m_character: character,
                m_nanos: nanos,
                m_repeat: false,
                m_hasText: true,
            }),
        }
    }
}

pub(crate) fn handleEvent(event: &Event)
{
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        if !state.m_created
        {
            return;
        }
        let nanos: i64 = jvm::system::nanoTime();
        match event
        {
            Event::KeyDown { keycode: Some(keycode), repeat, .. } =>
            {
                let key: i32 = lwjglKey(*keycode);
                state.m_droppedRepeat = *repeat && !state.m_repeatEnabled;
                if state.m_droppedRepeat
                {
                    return;
                }
                state.m_keyDown[key as usize] = true;
                let character: u16 = controlCharacter(key);
                state.m_queue.push_back(KeyEvent {
                    m_key: key,
                    m_state: true,
                    m_character: character,
                    m_nanos: nanos,
                    m_repeat: *repeat,
                    m_hasText: character != Keyboard::CHAR_NONE,
                });
            }
            Event::KeyUp { keycode: Some(keycode), .. } =>
            {
                let key: i32 = lwjglKey(*keycode);
                state.m_droppedRepeat = false;
                state.m_keyDown[key as usize] = false;
                state.m_queue.push_back(KeyEvent {
                    m_key: key,
                    m_state: false,
                    m_character: Keyboard::CHAR_NONE,
                    m_nanos: nanos,
                    m_repeat: false,
                    m_hasText: true,
                });
            }
            Event::TextInput { text, .. } if !state.m_droppedRepeat => attachText(&mut state, text, nanos),
            _ =>
            {}
        }
    });
}

impl Keyboard
{
    pub fn create() -> Result<(), crate::LWJGLException>
    {
        STATE.with(|state| {
            let mut state = state.borrow_mut();
            state.m_created = true;
            state.m_keyDown = vec![false; Self::KEYBOARD_SIZE as usize];
            state.m_queue.clear();
        });
        Ok(())
    }

    pub fn destroy() { STATE.with(|state| *state.borrow_mut() = KeyboardState::default()); }

    pub fn isCreated() -> bool { STATE.with(|state| state.borrow().m_created) }

    pub fn enableRepeatEvents(enable: bool) { STATE.with(|state| state.borrow_mut().m_repeatEnabled = enable); }

    pub fn areRepeatEventsEnabled() -> bool { STATE.with(|state| state.borrow().m_repeatEnabled) }

    pub fn isKeyDown(key: i32) -> bool
    {
        STATE.with(|state| state.borrow().m_keyDown.get(key as usize).copied().unwrap_or(false))
    }

    pub fn getKeyName(key: i32) -> Option<String>
    {
        KEY_NAMES
            .iter()
            .find(|(_, code)| *code == key)
            .and_then(|(name, _)| name.strip_prefix("KEY_"))
            .map(str::to_owned)
    }

    pub fn getKeyIndex(name: &str) -> i32
    {
        KEY_NAMES
            .iter()
            .find(|(field, _)| field.strip_prefix("KEY_") == Some(name))
            .map_or(Self::KEY_NONE, |(_, code)| *code)
    }

    pub fn getNumKeyboardEvents() -> i32 { STATE.with(|state| state.borrow().m_queue.len() as i32) }

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

    pub fn getEventKey() -> i32 { STATE.with(|state| state.borrow().m_current.m_key) }

    pub fn getEventCharacter() -> u16 { STATE.with(|state| state.borrow().m_current.m_character) }

    pub fn getEventKeyState() -> bool { STATE.with(|state| state.borrow().m_current.m_state) }

    pub fn getEventNanoseconds() -> i64 { STATE.with(|state| state.borrow().m_current.m_nanos) }

    pub fn isRepeatEvent() -> bool { STATE.with(|state| state.borrow().m_current.m_repeat) }
}
