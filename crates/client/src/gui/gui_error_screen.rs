use crate::gui::gui_screen::{GUI_SCREEN, GuiScreenClass};
use crate::gui::{FontRenderer, GuiScreen};
use std::cell::Cell;
use std::rc::Rc;

pub fn drawWarning(this: &GuiScreen, title: &str, lines: &[(&str, i32)])
{
    this.drawDefaultBackground();
    let font: Rc<FontRenderer> = this.fontRenderer();
    let top: i32 = this.m_height.get() / 4 - 60;
    this.m_gui.drawCenteredString(&font, title, this.m_width.get() / 2, top + 20, 0xFFFFFF);
    for (line, offset) in lines
    {
        this.m_gui.drawString(&font, line, this.m_width.get() / 2 - 140, top + 60 + offset, 0xA0A0A0);
    }
}

const CLASS: GuiScreenClass = GuiScreenClass {
    m_name: "GuiErrorScreen",
    m_parent: Some(&GUI_SCREEN),
    m_interfaces: &[],
    updateScreen: |this| {
        let ticks: &Cell<i32> = this.ext::<Cell<i32>>();
        ticks.set(ticks.get() + 1);
    },
    initGui: |_| {},
    actionPerformed: |_, _| {},
    keyTyped: |_, _, _| {},
    drawScreen: |this, mouseX, mouseY, tickDelta| {
        drawWarning(
            this,
            "Out of memory!",
            &[
                ("Minecraft has run out of memory.", 0),
                ("This could be caused by a bug in the game or by the", 18),
                ("Java Virtual Machine not being allocated enough", 27),
                ("memory. If you are playing in a web browser, try", 36),
                ("downloading the game and playing it offline.", 45),
                ("To prevent level corruption, the current game has quit.", 63),
                ("Please restart the game.", 81),
            ],
        );
        (GUI_SCREEN.drawScreen)(this, mouseX, mouseY, tickDelta);
    },
    ..GUI_SCREEN
};

pub struct GuiErrorScreen;

impl GuiErrorScreen
{
    pub fn new() -> Rc<GuiScreen> { GuiScreen::construct(&CLASS, Box::new(Cell::new(0i32))) }
}
