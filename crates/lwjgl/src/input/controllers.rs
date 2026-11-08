use crate::LWJGLException;

// the game only creates the controller environment, sdl joysticks are left alone
pub struct Controllers;

impl Controllers
{
    pub fn create() -> Result<(), LWJGLException> { Ok(()) }

    pub fn getControllerCount() -> i32 { 0 }

    pub fn destroy() {}
}
