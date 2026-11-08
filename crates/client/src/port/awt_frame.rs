use crate::client::Minecraft;
use crate::settings::Component;
use lwjgl::opengl::Display;

pub struct Canvas
{
    m_preferredWidth: i32,
    m_preferredHeight: i32,
}

impl Canvas
{
    pub fn new(preferredWidth: i32, preferredHeight: i32) -> Canvas
    {
        Canvas { m_preferredWidth: preferredWidth, m_preferredHeight: preferredHeight }
    }
}

impl Component for Canvas
{
    fn getWidth(&self) -> i32
    {
        if Display::isCreated()
        {
            Display::getWindowSize().0
        }
        else
        {
            self.m_preferredWidth
        }
    }

    fn getHeight(&self) -> i32
    {
        if Display::isCreated()
        {
            Display::getWindowSize().1
        }
        else
        {
            self.m_preferredHeight
        }
    }
}

pub struct Frame;

impl Frame
{
    pub fn dispatchEvents(&self, minecraft: &Minecraft)
    {
        if Display::isCloseRequested()
        {
            self.windowClosing(minecraft);
        }
    }

    fn windowClosing(&self, minecraft: &Minecraft) { minecraft.shutdown(); }
}
