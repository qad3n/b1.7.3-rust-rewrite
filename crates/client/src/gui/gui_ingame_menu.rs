use crate::client::Minecraft;
use crate::gui::GuiButton;
use crate::gui::gui_screen::{GUI_SCREEN, GuiScreen, GuiScreenClass};
use game::locale::StatCollector;
use game::math::math_helper::sin;
use game::stats::StatList;
use jvm::throwable::OrNpe;
use std::cell::Cell;
use std::f32::consts::PI;
use std::rc::Rc;

pub struct GuiIngameMenu
{
    m_updateCounter2: Cell<i32>,
    m_updateCounter: Cell<i32>,
}

fn fields(this: &GuiScreen) -> &GuiIngameMenu { this.ext::<GuiIngameMenu>() }

pub const GUI_INGAME_MENU: GuiScreenClass = GuiScreenClass {
    m_name: "GuiIngameMenu",
    m_parent: Some(&GUI_SCREEN),
    m_interfaces: &[],
    initGui: |this| {
        fields(this).m_updateCounter2.set(0);
        this.m_controlList.clear();
        let b0: i32 = -16;
        let (width, height): (i32, i32) = (this.m_width.get(), this.m_height.get());
        this.m_controlList.add(GuiButton::new(1, width / 2 - 100, height / 4 + 120 + b0, "Save and quit to title"));
        if this.mc().isMultiplayerWorld()
        {
            this.m_controlList.get(0).m_displayString.set(Some("Disconnect".to_owned()));
        }
        this.m_controlList.add(GuiButton::new(4, width / 2 - 100, height / 4 + 24 + b0, "Back to game"));
        this.m_controlList.add(GuiButton::new(0, width / 2 - 100, height / 4 + 96 + b0, "Options..."));
        this.m_controlList.add(GuiButton::withSize(
            5,
            width / 2 - 100,
            height / 4 + 48 + b0,
            98,
            20,
            &StatCollector::translateToLocal("gui.achievements"),
        ));
        this.m_controlList.add(GuiButton::withSize(
            6,
            width / 2 + 2,
            height / 4 + 48 + b0,
            98,
            20,
            &StatCollector::translateToLocal("gui.stats"),
        ));
    },
    actionPerformed: |this, button| {
        let mc: Rc<Minecraft> = this.mc();
        let id: i32 = button.m_id.get();
        if id == 0
        {
            mc.displayGuiScreen(Some(crate::gui::GuiOptions::new(&this.rc(), &mc.m_gameSettings.get().orNpe())));
        }
        if id == 1
        {
            mc.m_statFileWriter.get().orNpe().readStat(&StatList::leaveGameStat().orNpe(), 1);
            if mc.isMultiplayerWorld()
            {
                mc.m_theWorld.get().orNpe().sendQuittingDisconnectingPacket();
            }
            mc.changeWorld1(None);
            mc.displayGuiScreen(Some(crate::gui::GuiMainMenu::new()));
        }
        if id == 4
        {
            mc.displayGuiScreen(None);
            mc.setIngameFocus();
        }
        if id == 5
        {
            mc.displayGuiScreen(Some(crate::gui::GuiAchievements::new(mc.m_statFileWriter.get().orNpe())));
        }
        if id == 6
        {
            mc.displayGuiScreen(Some(crate::gui::GuiStats::new(this.rc(), mc.m_statFileWriter.get().orNpe())));
        }
    },
    updateScreen: |this| {
        (GUI_SCREEN.updateScreen)(this);
        fields(this).m_updateCounter.set(fields(this).m_updateCounter.get() + 1);
    },
    drawScreen: |this, mouseX, mouseY, tickDelta| {
        this.drawDefaultBackground();
        let menu: &GuiIngameMenu = fields(this);
        let counter2: i32 = menu.m_updateCounter2.get();
        menu.m_updateCounter2.set(counter2 + 1);
        let flag: bool = !this.mc().m_theWorld.get().orNpe().saveWhilePaused(counter2);
        if flag || menu.m_updateCounter.get() < 20
        {
            let mut f: f32 = ((menu.m_updateCounter.get() % 10) as f32 + tickDelta) / 10.0;
            f = sin(f * PI * 2.0) * 0.2 + 0.8;
            let i: i32 = (255.0 * f) as i32;
            this.m_gui.drawString(
                &this.fontRenderer(),
                "Saving level..",
                8,
                this.m_height.get() - 16,
                i << 16 | i << 8 | i,
            );
        }
        this.m_gui.drawCenteredString(&this.fontRenderer(), "Game menu", this.m_width.get() / 2, 40, 16777215);
        (GUI_SCREEN.drawScreen)(this, mouseX, mouseY, tickDelta);
    },
    ..GUI_SCREEN
};

impl GuiIngameMenu
{
    pub fn new() -> Rc<GuiScreen>
    {
        GuiScreen::construct(
            &GUI_INGAME_MENU,
            Box::new(GuiIngameMenu { m_updateCounter2: Cell::new(0), m_updateCounter: Cell::new(0) }),
        )
    }
}
