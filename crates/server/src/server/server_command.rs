use crate::server::ICommandListener;
use std::rc::Rc;

pub struct ServerCommand
{
    pub m_command: String,
    pub m_commandListener: Rc<dyn ICommandListener>,
}

impl ServerCommand
{
    pub fn new(command: String, source: Rc<dyn ICommandListener>) -> ServerCommand
    {
        ServerCommand { m_command: command, m_commandListener: source }
    }
}
