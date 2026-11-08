use crate::client::Minecraft;
use crate::gui::gui_screen::{GUI_SCREEN, GuiScreenClass};
use crate::gui::{GuiButton, GuiControls, GuiScreen, GuiSlider, GuiVideoSettings};
use crate::settings::{EnumOptions, GameSettings};
use game::locale::StringTranslate;
use jvm::field::Field;
use jvm::throwable::OrNpe;
use std::rc::Rc;

pub struct OptionsExt
{
    m_parentScreen: Rc<GuiScreen>,
    m_screenTitle: Field<String>,
    m_options: Rc<GameSettings>,
}

fn fields(this: &GuiScreen) -> &OptionsExt { this.ext::<OptionsExt>() }

const RENDER_OPTIONS: [EnumOptions; 5] = [
    EnumOptions::Music,
    EnumOptions::Sound,
    EnumOptions::InvertMouse,
    EnumOptions::Sensitivity,
    EnumOptions::Difficulty,
];

pub fn addOptionControls(this: &GuiScreen, options: &GameSettings, list: &[EnumOptions])
{
    for (n, enumOptions) in list.iter().enumerate()
    {
        let n: i32 = n as i32;
        let x: i32 = this.m_width.get() / 2 - 155 + n % 2 * 160;
        let y: i32 = this.m_height.get() / 6 + 24 * (n >> 1);
        let label: String = options.getKeyBinding(*enumOptions);
        let id: i32 = enumOptions.returnEnumOrdinal();
        if !enumOptions.getEnumFloat()
        {
            this.m_controlList.add(GuiButton::smallOption(id, x, y, Some(*enumOptions), &label));
        }
        else
        {
            this.m_controlList.add(GuiSlider::new(
                id,
                x,
                y,
                Some(*enumOptions),
                &label,
                options.getOptionFloatValue(*enumOptions),
            ));
        }
    }
}

pub fn toggleOption(options: &GameSettings, button: &GuiButton)
{
    if button.m_id.get() < 100 && button.instanceOf("GuiSmallButton")
    {
        options.setOptionValue(button.returnEnumOptions().orNpe(), 1);
        button.m_displayString.set(Some(options.getKeyBinding(EnumOptions::getEnumOptions(button.m_id.get()).orNpe())));
    }
}

const CLASS: GuiScreenClass = GuiScreenClass {
    m_name: "GuiOptions",
    m_parent: Some(&GUI_SCREEN),
    m_interfaces: &[],
    initGui: |this| {
        let ext: &OptionsExt = fields(this);
        let stringTranslate: Rc<StringTranslate> = StringTranslate::getInstance();
        ext.m_screenTitle.set(stringTranslate.translateKey("options.title"));
        addOptionControls(this, &ext.m_options, &RENDER_OPTIONS);
        let x: i32 = this.m_width.get() / 2 - 100;
        let y: i32 = this.m_height.get() / 6;
        this.m_controlList.add(GuiButton::new(101, x, y + 96 + 12, &stringTranslate.translateKey("options.video")));
        this.m_controlList.add(GuiButton::new(100, x, y + 120 + 12, &stringTranslate.translateKey("options.controls")));
        this.m_controlList.add(GuiButton::new(200, x, y + 168, &stringTranslate.translateKey("gui.done")));
    },
    actionPerformed: |this, button| {
        if !button.m_enabled.get()
        {
            return;
        }
        let ext: &OptionsExt = fields(this);
        toggleOption(&ext.m_options, button);
        let mc: Rc<Minecraft> = this.mc();
        match button.m_id.get()
        {
            101 =>
            {
                mc.m_gameSettings.get().orNpe().saveOptions();
                mc.displayGuiScreen(Some(GuiVideoSettings::new(&this.rc(), &ext.m_options)));
            }
            100 =>
            {
                mc.m_gameSettings.get().orNpe().saveOptions();
                mc.displayGuiScreen(Some(GuiControls::new(&this.rc(), &ext.m_options)));
            }
            200 =>
            {
                mc.m_gameSettings.get().orNpe().saveOptions();
                mc.displayGuiScreen(Some(Rc::clone(&ext.m_parentScreen)));
            }
            _ =>
            {}
        }
    },
    drawScreen: |this, mouseX, mouseY, tickDelta| {
        this.drawDefaultBackground();
        this.m_gui.drawCenteredString(
            &this.fontRenderer(),
            &fields(this).m_screenTitle.get(),
            this.m_width.get() / 2,
            20,
            0xFFFFFF,
        );
        (GUI_SCREEN.drawScreen)(this, mouseX, mouseY, tickDelta);
    },
    ..GUI_SCREEN
};

pub struct GuiOptions;

impl GuiOptions
{
    pub fn new(parent: &Rc<GuiScreen>, options: &Rc<GameSettings>) -> Rc<GuiScreen>
    {
        let ext: OptionsExt = OptionsExt {
            m_parentScreen: Rc::clone(parent),
            m_screenTitle: Field::new("Options".to_string()),
            m_options: Rc::clone(options),
        };
        GuiScreen::construct(&CLASS, Box::new(ext))
    }
}
