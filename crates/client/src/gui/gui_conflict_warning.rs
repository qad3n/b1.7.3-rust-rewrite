use crate::gui::gui_error_screen::drawWarning;
use crate::gui::gui_screen::{GUI_SCREEN, GuiScreenClass};
use crate::gui::{GuiButton, GuiMainMenu, GuiScreen};
use std::cell::Cell;
use std::rc::Rc;

const CLASS: GuiScreenClass = GuiScreenClass {
    m_name: "GuiConflictWarning",
    m_parent: Some(&GUI_SCREEN),
    m_interfaces: &[],
    updateScreen: |this| {
        let updateCounter: &Cell<i32> = this.ext::<Cell<i32>>();
        updateCounter.set(updateCounter.get() + 1);
    },
    initGui: |this| {
        this.m_controlList.clear();
        this.m_controlList.add(GuiButton::new(
            0,
            this.m_width.get() / 2 - 100,
            this.m_height.get() / 4 + 120 + 12,
            "Back to title screen",
        ));
    },
    actionPerformed: |this, button| {
        if !button.m_enabled.get()
        {
            return;
        }
        if button.m_id.get() == 0
        {
            this.mc().displayGuiScreen(Some(GuiMainMenu::new()));
        }
    },
    drawScreen: |this, mouseX, mouseY, tickDelta| {
        drawWarning(
            this,
            "Level save conflict",
            &[
                ("Minecraft detected a conflict in the level save data.", 0),
                ("This could be caused by two copies of the game", 18),
                ("accessing the same level.", 27),
                ("To prevent level corruption, the current game has quit.", 45),
            ],
        );
        (GUI_SCREEN.drawScreen)(this, mouseX, mouseY, tickDelta);
    },
    ..GUI_SCREEN
};

pub struct GuiConflictWarning;

impl GuiConflictWarning
{
    pub fn new() -> Rc<GuiScreen> { GuiScreen::construct(&CLASS, Box::new(Cell::new(0i32))) }
}
