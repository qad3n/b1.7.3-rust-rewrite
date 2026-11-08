use crate::client::Minecraft;
use crate::gui::gui_options::{addOptionControls, toggleOption};
use crate::gui::gui_screen::{GUI_SCREEN, GuiScreenClass};
use crate::gui::{GuiButton, GuiScreen, ScaledResolution};
use crate::settings::{EnumOptions, GameSettings};
use game::locale::StringTranslate;
use jvm::field::Field;
use jvm::throwable::OrNpe;
use std::rc::Rc;

struct VideoSettingsExt
{
    m_parent: Rc<GuiScreen>,
    m_title: Field<String>,
    m_guiGameSettings: Rc<GameSettings>,
}

fn fields(this: &GuiScreen) -> &VideoSettingsExt { this.ext::<VideoSettingsExt>() }

const VIDEO_OPTIONS: [EnumOptions; 8] = [
    EnumOptions::Graphics,
    EnumOptions::RenderDistance,
    EnumOptions::AmbientOcclusion,
    EnumOptions::FramerateLimit,
    EnumOptions::Anaglyph,
    EnumOptions::ViewBobbing,
    EnumOptions::GuiScale,
    EnumOptions::AdvancedOpengl,
];

const CLASS: GuiScreenClass = GuiScreenClass {
    m_name: "GuiVideoSettings",
    m_parent: Some(&GUI_SCREEN),
    m_interfaces: &[],
    initGui: |this| {
        let ext: &VideoSettingsExt = fields(this);
        let stringTranslate: Rc<StringTranslate> = StringTranslate::getInstance();
        ext.m_title.set(stringTranslate.translateKey("options.videoTitle"));
        addOptionControls(this, &ext.m_guiGameSettings, &VIDEO_OPTIONS);
        this.m_controlList.add(GuiButton::new(
            200,
            this.m_width.get() / 2 - 100,
            this.m_height.get() / 6 + 168,
            &stringTranslate.translateKey("gui.done"),
        ));
    },
    actionPerformed: |this, button| {
        if !button.m_enabled.get()
        {
            return;
        }
        let ext: &VideoSettingsExt = fields(this);
        toggleOption(&ext.m_guiGameSettings, button);
        let mc: Rc<Minecraft> = this.mc();
        let settings: Rc<GameSettings> = mc.m_gameSettings.get().orNpe();
        if button.m_id.get() == 200
        {
            settings.saveOptions();
            mc.displayGuiScreen(Some(Rc::clone(&ext.m_parent)));
        }
        let scaledResolution: ScaledResolution =
            ScaledResolution::new(&settings, mc.m_displayWidth.get(), mc.m_displayHeight.get());
        let n: i32 = scaledResolution.getScaledWidth();
        let n2: i32 = scaledResolution.getScaledHeight();
        this.rc().setWorldAndResolution(&mc, n, n2);
    },
    drawScreen: |this, mouseX, mouseY, tickDelta| {
        this.drawDefaultBackground();
        this.m_gui.drawCenteredString(
            &this.fontRenderer(),
            &fields(this).m_title.get(),
            this.m_width.get() / 2,
            20,
            0xFFFFFF,
        );
        (GUI_SCREEN.drawScreen)(this, mouseX, mouseY, tickDelta);
    },
    ..GUI_SCREEN
};

pub struct GuiVideoSettings;

impl GuiVideoSettings
{
    pub fn new(parent: &Rc<GuiScreen>, options: &Rc<GameSettings>) -> Rc<GuiScreen>
    {
        let ext: VideoSettingsExt = VideoSettingsExt {
            m_parent: Rc::clone(parent),
            m_title: Field::new("Video Settings".to_string()),
            m_guiGameSettings: Rc::clone(options),
        };
        GuiScreen::construct(&CLASS, Box::new(ext))
    }
}
