use crate::client::{Minecraft, Session};
use crate::gui::gui_screen::{GUI_SCREEN, GuiScreenClass};
use crate::gui::{FontRenderer, GuiButton, GuiConnecting, GuiScreen, GuiTextField};
use crate::settings::GameSettings;
use game::port::text;
use jvm::field::Field;
use jvm::throwable::OrNpe;
use lwjgl::input::Keyboard;
use std::rc::Rc;

mod layout
{
    pub const FIELD_WIDTH: i32 = 200;
    pub const FIELD_HEIGHT: i32 = 20;
    pub const FIELD_TOP: i32 = 50;
    pub const LABEL_GAP: i32 = 11;
    pub const BUTTON_TOP: i32 = 108;
    pub const BUTTON_GAP: i32 = 24;
    pub const MAX_LENGTH: i32 = 16;
}

mod buttons
{
    pub const CONNECT: i32 = 0;
    pub const CANCEL: i32 = 1;
}

const TEXT_COLOR: i32 = 0xFFFFFF;
const LABEL_COLOR: i32 = 0xA0A0A0;
const ERROR_COLOR: i32 = 0xFF5555;
const INVALID_NAME: &str = "Use 3 to 16 letters, digits or underscores";

struct NameExt
{
    m_parent: Rc<GuiScreen>,
    m_host: String,
    m_port: i32,
    m_field: Field<Option<Rc<GuiTextField>>>,
    m_error: Field<Option<String>>,
}

fn fields(this: &GuiScreen) -> &NameExt { this.ext::<NameExt>() }

fn nameField(this: &GuiScreen) -> Rc<GuiTextField> { fields(this).m_field.get().orNpe() }

fn top(this: &GuiScreen) -> i32 { this.m_height.get() / 4 }

// the remembered name, or the launcher name before any server was joined
fn lastName(mc: &Minecraft) -> String
{
    let last: String = mc.m_gameSettings.get().orNpe().m_lastName.get();
    if !last.is_empty()
    {
        return last;
    }

    mc.m_session.get().and_then(|session| session.m_username.get()).unwrap_or_default()
}

fn connect(this: &GuiScreen)
{
    let ext: &NameExt = fields(this);
    let name: String = text::trim(&nameField(this).getText()).to_string();

    if auth::canonical(&name).is_none()
    {
        ext.m_error.set(Some(String::from(INVALID_NAME)));
        return;
    }

    let mc: Rc<Minecraft> = this.mc();
    let settings: Rc<GameSettings> = mc.m_gameSettings.get().orNpe();
    settings.m_lastName.set(name.clone());
    settings.saveOptions();

    let session: Rc<Session> = mc.m_session.get().orNpe();
    session.m_username.set(Some(name));

    mc.displayGuiScreen(Some(GuiConnecting::new(&mc, &ext.m_host, ext.m_port)));
}

const CLASS: GuiScreenClass = GuiScreenClass
{
    m_name: "GuiName",
    m_parent: Some(&GUI_SCREEN),
    m_interfaces: &[],
    updateScreen: |this| nameField(this).updateCursorCounter(),
    initGui: |this|
    {
        Keyboard::enableRepeatEvents(true);

        let x: i32 = this.m_width.get() / 2 - layout::FIELD_WIDTH / 2;
        let y: i32 = top(this);

        this.m_controlList.clear();
        this.m_controlList.add(GuiButton::new(buttons::CONNECT, x, y + layout::BUTTON_TOP, "Connect"));
        this.m_controlList.add(GuiButton::new(
            buttons::CANCEL,
            x,
            y + layout::BUTTON_TOP + layout::BUTTON_GAP,
            "Cancel",
        ));

        let name: String = lastName(&this.mc());
        this.m_controlList.get(0).m_enabled.set(!name.is_empty());

        let field: GuiTextField = GuiTextField::new(
            this.rc(),
            this.fontRenderer(),
            x,
            y + layout::FIELD_TOP,
            layout::FIELD_WIDTH,
            layout::FIELD_HEIGHT,
            &name,
        );
        field.m_isFocused.set(true);
        field.setMaxStringLength(layout::MAX_LENGTH);
        fields(this).m_field.set(Some(Rc::new(field)));
    },
    onGuiClosed: |_| Keyboard::enableRepeatEvents(false),
    actionPerformed: |this, button|
    {
        if !button.m_enabled.get()
        {
            return;
        }

        match button.m_id.get()
        {
            buttons::CONNECT => connect(this),
            buttons::CANCEL => this.mc().displayGuiScreen(Some(Rc::clone(&fields(this).m_parent))),
            _ =>
            {}
        }
    },

    keyTyped: |this, chr, key|
    {
        nameField(this).textboxKeyTyped(chr, key);

        if chr == b'\r' as u16
        {
            this.actionPerformed(&this.m_controlList.get(0));
        }

        this.m_controlList.get(0).m_enabled.set(nameField(this).getText().length() > 0);
    },

    mouseClicked: |this, mouseX, mouseY, button|
    {
        (GUI_SCREEN.mouseClicked)(this, mouseX, mouseY, button);
        nameField(this).mouseClicked(mouseX, mouseY, button);
    },

    drawScreen: |this, mouseX, mouseY, tickDelta|
    {
        this.drawDefaultBackground();

        let font: Rc<FontRenderer> = this.fontRenderer();
        let center: i32 = this.m_width.get() / 2;
        let x: i32 = center - layout::FIELD_WIDTH / 2;
        let y: i32 = top(this);

        this.m_gui.drawCenteredString(&font, "Choose a name", center, y, TEXT_COLOR);

        let server: String = format!("for {}", fields(this).m_host);
        this.m_gui.drawCenteredString(&font, &server, center, y + layout::LABEL_GAP + 3, LABEL_COLOR);
        this.m_gui.drawString(&font, "Player name", x, y + layout::FIELD_TOP - layout::LABEL_GAP, LABEL_COLOR);
        nameField(this).drawTextBox();

        if let Some(error) = fields(this).m_error.get()
        {
            let errorY: i32 = y + layout::FIELD_TOP + layout::FIELD_HEIGHT + layout::LABEL_GAP / 2;
            this.m_gui.drawCenteredString(&font, &error, center, errorY, ERROR_COLOR);
        }

        (GUI_SCREEN.drawScreen)(this, mouseX, mouseY, tickDelta);
    },
    ..GUI_SCREEN
};

pub struct GuiName;

impl GuiName
{
    pub fn new(parent: &Rc<GuiScreen>, host: &str, port: i32) -> Rc<GuiScreen>
    {
        let ext: NameExt = NameExt
        {
            m_parent: Rc::clone(parent),
            m_host: host.to_string(),
            m_port: port,
            m_field: Field::new(None),
            m_error: Field::new(None),
        };
        GuiScreen::construct(&CLASS, Box::new(ext))
    }
}
