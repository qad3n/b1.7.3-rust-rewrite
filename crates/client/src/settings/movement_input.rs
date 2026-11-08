use crate::settings::GameSettings;
use game::entity::Entity;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

pub(super) struct FromOptions
{
    pub(super) m_movementKeyStates: RefCell<[bool; 10]>,
    pub(super) m_gameSettings: Rc<GameSettings>,
}

pub struct MovementInput
{
    pub m_moveStrafe: Cell<f32>,
    pub m_moveForward: Cell<f32>,
    pub m_wasJumping: Cell<bool>,
    pub m_jump: Cell<bool>,
    pub m_sneak: Cell<bool>,
    pub(super) m_fromOptions: Option<FromOptions>,
}

impl MovementInput
{
    pub(super) fn alloc(fromOptions: Option<FromOptions>) -> MovementInput
    {
        MovementInput {
            m_moveStrafe: Cell::new(0.0),
            m_moveForward: Cell::new(0.0),
            m_wasJumping: Cell::new(false),
            m_jump: Cell::new(false),
            m_sneak: Cell::new(false),
            m_fromOptions: fromOptions,
        }
    }

    pub fn new() -> MovementInput { MovementInput::alloc(None) }

    pub fn updatePlayerMoveState(&self, player: &Rc<Entity>)
    {
        if let Some(fromOptions) = &self.m_fromOptions
        {
            fromOptions.updatePlayerMoveState(self, player);
        }
    }

    pub fn resetKeyState(&self)
    {
        if let Some(fromOptions) = &self.m_fromOptions
        {
            fromOptions.resetKeyState();
        }
    }

    pub fn checkKeyForMovementInput(&self, key: i32, state: bool)
    {
        if let Some(fromOptions) = &self.m_fromOptions
        {
            fromOptions.checkKeyForMovementInput(key, state);
        }
    }
}

impl Default for MovementInput
{
    fn default() -> Self { Self::new() }
}
