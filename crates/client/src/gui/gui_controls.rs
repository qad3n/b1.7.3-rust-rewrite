use crate::gui::gui_screen::{GUI_SCREEN, GuiScreenClass};
use crate::gui::{FontRenderer, GuiButton, GuiScreen};
use crate::settings::GameSettings;
use game::locale::StringTranslate;
use jvm::field::Field;
use std::cell::Cell;
use std::rc::Rc;

struct ControlsExt
{
    m_parentScreen: Rc<GuiScreen>,
    m_screenTitle: Field<String>,
    m_options: Rc<GameSettings>,
    m_buttonId: Cell<i32>,
}

fn fields(this: &GuiScreen) -> &ControlsExt { this.ext::<ControlsExt>() }

fn getControlsListX(this: &GuiScreen) -> i32 { this.m_width.get() / 2 - 155 }

fn addKeyButton(this: &GuiScreen, options: &GameSettings, i: i32, x: i32)
{
    let button: Rc<GuiButton> =
        GuiButton::smallWithSize(i, x + i % 2 * 160, this.m_height.get() / 6 + 24 * (i >> 1), 70, 20, "");
    button.m_displayString.set(options.getOptionDisplayString(i));
    this.m_controlList.add(button);
}

const CLASS: GuiScreenClass = GuiScreenClass {
    m_name: "GuiControls",
    m_parent: Some(&GUI_SCREEN),
    m_interfaces: &[],
    initGui: |this| {
        let ext: &ControlsExt = fields(this);
        let stringTranslate: Rc<StringTranslate> = StringTranslate::getInstance();
        let n: i32 = getControlsListX(this);
        for i in 0..ext.m_options.m_keyBindings.len() as i32
        {
            addKeyButton(this, &ext.m_options, i, n);
        }
        this.m_controlList.add(GuiButton::new(
            200,
            this.m_width.get() / 2 - 100,
            this.m_height.get() / 6 + 168,
            &stringTranslate.translateKey("gui.done"),
        ));
        ext.m_screenTitle.set(stringTranslate.translateKey("controls.title"));
    },
    actionPerformed: |this, button| {
        let ext: &ControlsExt = fields(this);
        for i in 0..ext.m_options.m_keyBindings.len() as i32
        {
            this.m_controlList.get(i).m_displayString.set(ext.m_options.getOptionDisplayString(i));
        }
        if button.m_id.get() == 200
        {
            this.mc().displayGuiScreen(Some(Rc::clone(&ext.m_parentScreen)));
        }
        else
        {
            ext.m_buttonId.set(button.m_id.get());
            let name: Option<String> = ext.m_options.getOptionDisplayString(button.m_id.get());
            button.m_displayString.set(Some(format!("> {} <", name.as_deref().unwrap_or("null"))));
        }
    },
    keyTyped: |this, chr, key| {
        let ext: &ControlsExt = fields(this);
        if ext.m_buttonId.get() >= 0
        {
            ext.m_options.setKeyBinding(ext.m_buttonId.get(), key);
            this.m_controlList
                .get(ext.m_buttonId.get())
                .m_displayString
                .set(ext.m_options.getOptionDisplayString(ext.m_buttonId.get()));
            ext.m_buttonId.set(-1);
        }
        else
        {
            (GUI_SCREEN.keyTyped)(this, chr, key);
        }
    },
    drawScreen: |this, mouseX, mouseY, tickDelta| {
        let ext: &ControlsExt = fields(this);
        this.drawDefaultBackground();
        let font: Rc<FontRenderer> = this.fontRenderer();
        this.m_gui.drawCenteredString(&font, &ext.m_screenTitle.get(), this.m_width.get() / 2, 20, 0xFFFFFF);
        let n: i32 = getControlsListX(this);
        for i in 0..ext.m_options.m_keyBindings.len() as i32
        {
            this.m_gui.drawString(
                &font,
                &ext.m_options.getKeyBindingDescription(i),
                n + i % 2 * 160 + 70 + 6,
                this.m_height.get() / 6 + 24 * (i >> 1) + 7,
                -1,
            );
        }
        (GUI_SCREEN.drawScreen)(this, mouseX, mouseY, tickDelta);
    },
    ..GUI_SCREEN
};

pub struct GuiControls;

impl GuiControls
{
    pub fn new(parent: &Rc<GuiScreen>, options: &Rc<GameSettings>) -> Rc<GuiScreen>
    {
        let ext: ControlsExt = ControlsExt {
            m_parentScreen: Rc::clone(parent),
            m_screenTitle: Field::new("Controls".to_string()),
            m_options: Rc::clone(options),
            m_buttonId: Cell::new(-1),
        };
        GuiScreen::construct(&CLASS, Box::new(ext))
    }
}
