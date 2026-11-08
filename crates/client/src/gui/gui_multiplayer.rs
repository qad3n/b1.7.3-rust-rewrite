use crate::client::Minecraft;
use crate::gui::gui_screen::{GUI_SCREEN, GuiScreenClass};
use crate::gui::{FontRenderer, GuiButton, GuiConnecting, GuiScreen, GuiTextField};
use crate::settings::GameSettings;
use game::locale::StringTranslate;
use game::port::{parseInt, text};
use jvm::field::Field;
use jvm::throwable::OrNpe;
use lwjgl::input::Keyboard;
use std::rc::Rc;

struct MultiplayerExt
{
    m_parentScreen: Rc<GuiScreen>,
    m_serverField: Field<Option<Rc<GuiTextField>>>,
}

fn fields(this: &GuiScreen) -> &MultiplayerExt { this.ext::<MultiplayerExt>() }

fn serverField(this: &GuiScreen) -> Rc<GuiTextField> { fields(this).m_serverField.get().orNpe() }

// port parseInt and trimStr give java Integer.parseInt and String.trim on rust text
fn parseIntWithDefault(s: &str, defaultValue: i32) -> i32 { parseInt(text::trimStr(s)).unwrap_or(defaultValue) }

fn splitAddress(string: &str) -> Vec<String>
{
    let mut stringArray: Vec<String> = string.split(':').map(str::to_string).collect();
    if string.contains(':')
    {
        while stringArray.last().is_some_and(String::is_empty)
        {
            stringArray.pop();
        }
    }
    if string.starts_with('[')
        && let Some(n) = string.find(']').filter(|n| *n > 0)
    {
        let string2: &str = &string[1..n];
        // port trim strips chars up to space like java String.trim
        let mut string3: &str = text::trimStr(&string[n + 1..]);
        if string3.starts_with(':') && !string3.is_empty()
        {
            string3 = &string3[1..];
            stringArray = vec![string2.to_string(), string3.to_string()];
        }
        else
        {
            stringArray = vec![string2.to_string()];
        }
    }
    if stringArray.len() > 2
    {
        stringArray = vec![string.to_string()];
    }
    stringArray
}

const CLASS: GuiScreenClass = GuiScreenClass {
    m_name: "GuiMultiplayer",
    m_parent: Some(&GUI_SCREEN),
    m_interfaces: &[],
    updateScreen: |this| serverField(this).updateCursorCounter(),
    initGui: |this| {
        let stringTranslate: Rc<StringTranslate> = StringTranslate::getInstance();
        Keyboard::enableRepeatEvents(true);
        this.m_controlList.clear();
        let x: i32 = this.m_width.get() / 2 - 100;
        let y: i32 = this.m_height.get() / 4;
        this.m_controlList.add(GuiButton::new(0, x, y + 96 + 12, &stringTranslate.translateKey("multiplayer.connect")));
        this.m_controlList.add(GuiButton::new(1, x, y + 120 + 12, &stringTranslate.translateKey("gui.cancel")));
        let string: String = this.mc().m_gameSettings.get().orNpe().m_lastServer.get().replace('_', ":");
        this.m_controlList.get(0).m_enabled.set(!string.is_empty());
        let field: GuiTextField =
            GuiTextField::new(this.rc(), this.fontRenderer(), x, y - 10 + 50 + 18, 200, 20, &string);
        field.m_isFocused.set(true);
        field.setMaxStringLength(128);
        fields(this).m_serverField.set(Some(Rc::new(field)));
    },
    onGuiClosed: |_| Keyboard::enableRepeatEvents(false),
    actionPerformed: |this, button| {
        if !button.m_enabled.get()
        {
            return;
        }
        let mc: Rc<Minecraft> = this.mc();
        match button.m_id.get()
        {
            1 => mc.displayGuiScreen(Some(Rc::clone(&fields(this).m_parentScreen))),
            0 =>
            {
                // port trim strips chars up to space like java String.trim
                let text: String = text::trim(&serverField(this).getText()).to_string();
                let string: &str = &text;
                let settings: Rc<GameSettings> = mc.m_gameSettings.get().orNpe();
                settings.m_lastServer.set(string.replace(':', "_"));
                settings.saveOptions();
                let stringArray: Vec<String> = splitAddress(string);
                let port: i32 = if stringArray.len() > 1
                {
                    parseIntWithDefault(&stringArray[1], 25565)
                }
                else
                {
                    25565
                };
                let Some(host) = stringArray.first()
                else
                {
                    jvm::throwable::throw(&jvm::throwable::ARRAY_INDEX, "0");
                };
                mc.displayGuiScreen(Some(GuiConnecting::new(&mc, host, port)));
            }
            _ =>
            {}
        }
    },
    keyTyped: |this, chr, key| {
        serverField(this).textboxKeyTyped(chr, key);
        if chr == b'\r' as u16
        {
            this.actionPerformed(&this.m_controlList.get(0));
        }
        this.m_controlList.get(0).m_enabled.set(serverField(this).getText().length() > 0);
    },
    mouseClicked: |this, mouseX, mouseY, button| {
        (GUI_SCREEN.mouseClicked)(this, mouseX, mouseY, button);
        serverField(this).mouseClicked(mouseX, mouseY, button);
    },
    drawScreen: |this, mouseX, mouseY, tickDelta| {
        let stringTranslate: Rc<StringTranslate> = StringTranslate::getInstance();
        this.drawDefaultBackground();
        let font: Rc<FontRenderer> = this.fontRenderer();
        let top: i32 = this.m_height.get() / 4 - 60;
        let x: i32 = this.m_width.get() / 2 - 140;
        this.m_gui.drawCenteredString(
            &font,
            &stringTranslate.translateKey("multiplayer.title"),
            this.m_width.get() / 2,
            top + 20,
            0xFFFFFF,
        );
        this.m_gui.drawString(&font, &stringTranslate.translateKey("multiplayer.info1"), x, top + 60 + 0, 0xA0A0A0);
        this.m_gui.drawString(&font, &stringTranslate.translateKey("multiplayer.info2"), x, top + 60 + 9, 0xA0A0A0);
        this.m_gui.drawString(&font, &stringTranslate.translateKey("multiplayer.ipinfo"), x, top + 60 + 36, 0xA0A0A0);
        serverField(this).drawTextBox();
        (GUI_SCREEN.drawScreen)(this, mouseX, mouseY, tickDelta);
    },
    ..GUI_SCREEN
};

pub struct GuiMultiplayer;

impl GuiMultiplayer
{
    pub fn new(parent: &Rc<GuiScreen>) -> Rc<GuiScreen>
    {
        GuiScreen::construct(
            &CLASS,
            Box::new(MultiplayerExt { m_parentScreen: Rc::clone(parent), m_serverField: Field::new(None) }),
        )
    }
}
