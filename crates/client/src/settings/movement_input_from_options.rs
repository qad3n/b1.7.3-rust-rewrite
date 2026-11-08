use crate::settings::GameSettings;
use crate::settings::movement_input::{FromOptions, MovementInput};
use game::entity::Entity;
use std::cell::RefCell;
use std::rc::Rc;

impl MovementInput
{
    pub fn fromOptions(options: Rc<GameSettings>) -> MovementInput
    {
        MovementInput::alloc(Some(FromOptions {
            m_movementKeyStates: RefCell::new([false; 10]),
            m_gameSettings: options,
        }))
    }
}

impl FromOptions
{
    pub(super) fn checkKeyForMovementInput(&self, key: i32, state: bool)
    {
        let settings: &GameSettings = &self.m_gameSettings;
        let mut n: i32 = -1;
        if key == settings.m_keyBindForward.m_keyCode.get()
        {
            n = 0;
        }
        if key == settings.m_keyBindBack.m_keyCode.get()
        {
            n = 1;
        }
        if key == settings.m_keyBindLeft.m_keyCode.get()
        {
            n = 2;
        }
        if key == settings.m_keyBindRight.m_keyCode.get()
        {
            n = 3;
        }
        if key == settings.m_keyBindJump.m_keyCode.get()
        {
            n = 4;
        }
        if key == settings.m_keyBindSneak.m_keyCode.get()
        {
            n = 5;
        }
        if n >= 0
        {
            self.m_movementKeyStates.borrow_mut()[n as usize] = state;
        }
    }

    pub(super) fn resetKeyState(&self) { *self.m_movementKeyStates.borrow_mut() = [false; 10]; }

    pub(super) fn updatePlayerMoveState(&self, input: &MovementInput, _player: &Rc<Entity>)
    {
        let states: [bool; 10] = *self.m_movementKeyStates.borrow();
        input.m_moveStrafe.set(0.0);
        input.m_moveForward.set(0.0);
        if states[0]
        {
            input.m_moveForward.set(input.m_moveForward.get() + 1.0);
        }
        if states[1]
        {
            input.m_moveForward.set(input.m_moveForward.get() - 1.0);
        }
        if states[2]
        {
            input.m_moveStrafe.set(input.m_moveStrafe.get() + 1.0);
        }
        if states[3]
        {
            input.m_moveStrafe.set(input.m_moveStrafe.get() - 1.0);
        }
        input.m_jump.set(states[4]);
        input.m_sneak.set(states[5]);
        if input.m_sneak.get()
        {
            input.m_moveStrafe.set((input.m_moveStrafe.get() as f64 * 0.3) as f32);
            input.m_moveForward.set((input.m_moveForward.get() as f64 * 0.3) as f32);
        }
    }
}
