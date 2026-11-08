use crate::gui::gui_screen::{GUI_SCREEN, GuiScreenClass};
use crate::gui::{FontRenderer, GuiScreen};
use std::rc::Rc;

struct UnusedExt
{
    m_message1: Option<String>,
    m_message2: Option<String>,
}

const CLASS: GuiScreenClass = GuiScreenClass {
    m_name: "GuiUnused",
    m_parent: Some(&GUI_SCREEN),
    m_interfaces: &[],
    initGui: |_| {},
    drawScreen: |this, mouseX, mouseY, tickDelta| {
        let ext: &UnusedExt = this.ext::<UnusedExt>();
        this.m_gui.drawGradientRect(0, 0, this.m_width.get(), this.m_height.get(), -12574688, -11530224);
        let font: Rc<FontRenderer> = this.fontRenderer();
        let center: i32 = this.m_width.get() / 2;
        if let Some(message) = &ext.m_message1
        {
            this.m_gui.drawCenteredString(&font, message, center, 90, 0xFFFFFF);
        }
        if let Some(message) = &ext.m_message2
        {
            this.m_gui.drawCenteredString(&font, message, center, 110, 0xFFFFFF);
        }
        (GUI_SCREEN.drawScreen)(this, mouseX, mouseY, tickDelta);
    },
    keyTyped: |_, _, _| {},
    ..GUI_SCREEN
};

pub struct GuiUnused;

impl GuiUnused
{
    pub fn new() -> Rc<GuiScreen>
    {
        GuiScreen::construct(&CLASS, Box::new(UnusedExt { m_message1: None, m_message2: None }))
    }
}
