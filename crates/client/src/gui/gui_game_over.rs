use crate::client::Minecraft;
use crate::gui::GuiButton;
use crate::gui::gui_screen::{GUI_SCREEN, GuiScreen, GuiScreenClass};
use game::entity::entity_player;
use jvm::throwable::OrNpe;
use lwjgl::opengl::GL11;
use std::rc::Rc;

pub struct GuiGameOver;

pub const GUI_GAME_OVER: GuiScreenClass = GuiScreenClass {
    m_name: "GuiGameOver",
    m_parent: Some(&GUI_SCREEN),
    m_interfaces: &[],
    initGui: |this| {
        this.m_controlList.clear();
        let (width, height): (i32, i32) = (this.m_width.get(), this.m_height.get());
        this.m_controlList.add(GuiButton::new(1, width / 2 - 100, height / 4 + 72, "Respawn"));
        this.m_controlList.add(GuiButton::new(2, width / 2 - 100, height / 4 + 96, "Title menu"));
        if this.mc().m_session.get().is_none()
        {
            this.m_controlList.get(1).m_enabled.set(false);
        }
    },
    keyTyped: |_, _, _| {},
    actionPerformed: |this, button| {
        let mc: Rc<Minecraft> = this.mc();
        if button.m_id.get() == 1
        {
            mc.m_thePlayer.get().orNpe().respawnPlayer();
            mc.displayGuiScreen(None);
        }
        if button.m_id.get() == 2
        {
            mc.changeWorld1(None);
            mc.displayGuiScreen(Some(crate::gui::GuiMainMenu::new()));
        }
    },
    drawScreen: |this, mouseX, mouseY, tickDelta| {
        let (width, height): (i32, i32) = (this.m_width.get(), this.m_height.get());
        this.m_gui.drawGradientRect(0, 0, width, height, 0x60500000, -1602211792);
        GL11::glPushMatrix();
        GL11::glScalef(2.0, 2.0, 2.0);
        this.m_gui.drawCenteredString(&this.fontRenderer(), "Game over!", width / 2 / 2, 30, 0xFFFFFF);
        GL11::glPopMatrix();
        let score: i32 = entity_player::getScore(&this.mc().m_thePlayer.get().orNpe());
        this.m_gui.drawCenteredString(&this.fontRenderer(), &format!("Score: &e{score}"), width / 2, 100, 0xFFFFFF);
        (GUI_SCREEN.drawScreen)(this, mouseX, mouseY, tickDelta);
    },
    doesGuiPauseGame: |_| false,
    ..GUI_SCREEN
};

impl GuiGameOver
{
    pub fn new() -> Rc<GuiScreen> { GuiScreen::construct(&GUI_GAME_OVER, Box::new(())) }
}
