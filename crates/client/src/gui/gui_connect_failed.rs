use crate::gui::gui_screen::{GUI_SCREEN, GuiScreenClass};
use crate::gui::{FontRenderer, GuiButton, GuiMainMenu, GuiScreen};
use game::locale::StringTranslate;
use jvm::string::JString;
use std::rc::Rc;

struct ConnectFailedExt
{
    m_errorMessage: String,
    m_errorDetail: String,
}

const CLASS: GuiScreenClass = GuiScreenClass {
    m_name: "GuiConnectFailed",
    m_parent: Some(&GUI_SCREEN),
    m_interfaces: &[],
    updateScreen: |_| {},
    keyTyped: |_, _, _| {},
    initGui: |this| {
        let stringtranslate: Rc<StringTranslate> = StringTranslate::getInstance();
        this.m_controlList.clear();
        this.m_controlList.add(GuiButton::new(
            0,
            this.m_width.get() / 2 - 100,
            this.m_height.get() / 4 + 120 + 12,
            &stringtranslate.translateKey("gui.toMenu"),
        ));
    },
    actionPerformed: |this, button| {
        if button.m_id.get() == 0
        {
            this.mc().displayGuiScreen(Some(GuiMainMenu::new()));
        }
    },
    drawScreen: |this, mouseX, mouseY, tickDelta| {
        let ext: &ConnectFailedExt = this.ext::<ConnectFailedExt>();
        this.drawDefaultBackground();
        let font: Rc<FontRenderer> = this.fontRenderer();
        let center: i32 = this.m_width.get() / 2;
        this.m_gui.drawCenteredString(&font, &ext.m_errorMessage, center, this.m_height.get() / 2 - 50, 16777215);
        this.m_gui.drawCenteredString(&font, &ext.m_errorDetail, center, this.m_height.get() / 2 - 10, 16777215);
        (GUI_SCREEN.drawScreen)(this, mouseX, mouseY, tickDelta);
    },
    ..GUI_SCREEN
};

pub struct GuiConnectFailed;

impl GuiConnectFailed
{
    pub fn new(title: &str, reason: &str, reasonArgs: &[JString]) -> Rc<GuiScreen>
    {
        let stringtranslate: Rc<StringTranslate> = StringTranslate::getInstance();
        let args: Vec<String> = reasonArgs.iter().map(JString::to_string).collect();
        let ext: ConnectFailedExt = ConnectFailedExt {
            m_errorMessage: stringtranslate.translateKey(title),
            m_errorDetail: stringtranslate.translateKeyFormat(reason, &args),
        };
        GuiScreen::construct(&CLASS, Box::new(ext))
    }
}
