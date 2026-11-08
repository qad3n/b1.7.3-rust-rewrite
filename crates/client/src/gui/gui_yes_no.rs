use crate::gui::gui_screen::{GUI_SCREEN, GuiScreenClass};
use crate::gui::{FontRenderer, GuiButton, GuiScreen};
use std::rc::Rc;

struct YesNoExt
{
    m_parentScreen: Rc<GuiScreen>,
    m_message1: String,
    m_message2: String,
    m_confirmText: String,
    m_abortText: String,
    m_worldNumber: i32,
}

fn fields(this: &GuiScreen) -> &YesNoExt { this.ext::<YesNoExt>() }

const CLASS: GuiScreenClass = GuiScreenClass {
    m_name: "GuiYesNo",
    m_parent: Some(&GUI_SCREEN),
    m_interfaces: &[],
    initGui: |this| {
        let ext: &YesNoExt = fields(this);
        let x: i32 = this.m_width.get() / 2 - 155;
        let y: i32 = this.m_height.get() / 6 + 96;
        this.m_controlList.add(GuiButton::small(0, x + 0, y, &ext.m_confirmText));
        this.m_controlList.add(GuiButton::small(1, x + 160, y, &ext.m_abortText));
    },
    actionPerformed: |this, button| {
        fields(this).m_parentScreen.deleteWorld(button.m_id.get() == 0, fields(this).m_worldNumber);
    },
    drawScreen: |this, mouseX, mouseY, tickDelta| {
        let ext: &YesNoExt = fields(this);
        this.drawDefaultBackground();
        let font: Rc<FontRenderer> = this.fontRenderer();
        this.m_gui.drawCenteredString(&font, &ext.m_message1, this.m_width.get() / 2, 70, 0xFFFFFF);
        this.m_gui.drawCenteredString(&font, &ext.m_message2, this.m_width.get() / 2, 90, 0xFFFFFF);
        (GUI_SCREEN.drawScreen)(this, mouseX, mouseY, tickDelta);
    },
    ..GUI_SCREEN
};

pub struct GuiYesNo;

impl GuiYesNo
{
    pub fn new(
        listener: &Rc<GuiScreen>,
        title: &str,
        description: &str,
        confirmText: &str,
        abortText: &str,
        id: i32,
    ) -> Rc<GuiScreen>
    {
        let ext: YesNoExt = YesNoExt {
            m_parentScreen: Rc::clone(listener),
            m_message1: title.to_string(),
            m_message2: description.to_string(),
            m_confirmText: confirmText.to_string(),
            m_abortText: abortText.to_string(),
            m_worldNumber: id,
        };
        GuiScreen::construct(&CLASS, Box::new(ext))
    }
}
