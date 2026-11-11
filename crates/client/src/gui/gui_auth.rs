use crate::gui::gui_screen::{GUI_SCREEN, GuiScreenClass};
use crate::gui::{FontRenderer, GuiButton, GuiScreen, GuiTextField};
use crate::network::net_client_handler::NetClientHandler;
use game::network::packets::Packet0KeepAlive;
use jvm::field::Field;
use lwjgl::input::Keyboard;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AuthScreen
{
    Login,
    Register
    {
        invite: bool,
    },
    Change,
}

struct Entry
{
    m_label: &'static str,
    m_masked: bool,
}

const PASSWORD: Entry = Entry { m_label: "Password", m_masked: true };
const CONFIRM: Entry = Entry { m_label: "Confirm password", m_masked: true };
const INVITE: Entry = Entry { m_label: "Invite code from an op", m_masked: false };
const CURRENT: Entry = Entry { m_label: "Current password", m_masked: true };
const NEW: Entry = Entry { m_label: "New password", m_masked: true };
const CONFIRM_NEW: Entry = Entry { m_label: "Confirm new password", m_masked: true };

mod layout
{
    pub const FIELD_WIDTH: i32 = 200;
    pub const FIELD_HEIGHT: i32 = 20;
    pub const ROW: i32 = 36;
    pub const LABEL_GAP: i32 = 11;
    pub const TOP: i32 = 24;
    pub const MAX_LENGTH: i32 = 64;
}

mod buttons
{
    pub const DONE: i32 = 0;
    pub const CANCEL: i32 = 1;
}

const KEEP_ALIVE_TICKS: i32 = 20;
const LABEL_COLOR: i32 = 0xA0A0A0;
const TEXT_COLOR: i32 = 0xFFFFFF;
const ERROR_COLOR: i32 = 0xFF5555;

impl AuthScreen
{
    fn entries(self) -> &'static [Entry]
    {
        match self
        {
            Self::Login => &[PASSWORD],
            Self::Register { invite: false } => &[PASSWORD, CONFIRM],
            Self::Register { invite: true } => &[PASSWORD, CONFIRM, INVITE],
            Self::Change => &[CURRENT, NEW, CONFIRM_NEW],
        }
    }

    fn title(self) -> &'static str
    {
        match self
        {
            Self::Login => "Log in",
            Self::Register { .. } => "Create your account",
            Self::Change => "Change password",
        }
    }

    // the entry index a confirm field must match, if any
    fn confirms(self) -> Option<(usize, usize)>
    {
        match self
        {
            Self::Login => None,
            Self::Register { .. } => Some((0, 1)),
            Self::Change => Some((1, 2)),
        }
    }
}

struct AuthExt
{
    m_handler: Rc<NetClientHandler>,
    m_kind: AuthScreen,
    m_fingerprint: String,
    m_fields: RefCell<Vec<Rc<GuiTextField>>>,
    m_status: Field<Option<(String, bool)>>,
    m_sent: Cell<bool>,
    m_ticks: Cell<i32>,
}

fn fields(this: &GuiScreen) -> &AuthExt { this.ext::<AuthExt>() }

fn top(this: &GuiScreen) -> i32 { this.m_height.get() / 4 - layout::TOP }

fn submit(this: &GuiScreen)
{
    let ext: &AuthExt = fields(this);
    if ext.m_sent.get()
    {
        return;
    }

    let values: Vec<String> = ext.m_fields.borrow().iter().map(|field| field.getText().to_string()).collect();
    if let Some((first, second)) = ext.m_kind.confirms()
        && values[first] != values[second]
    {
        ext.m_status.set(Some((String::from("The passwords do not match"), true)));
        return;
    }

    match ext.m_handler.submitAuth(ext.m_kind, &values)
    {
        Ok(()) =>
        {
            for field in ext.m_fields.borrow().iter()
            {
                field.setText("");
                field.m_isEnabled.set(false);
            }

            ext.m_sent.set(true);
            this.m_controlList.get(0).m_enabled.set(false);
            ext.m_status.set(Some((String::from("Checking..."), false)));
        }

        Err(reason) => ext.m_status.set(Some((reason, true))),
    }
}

fn cancel(this: &GuiScreen)
{
    let ext: &AuthExt = fields(this);
    if ext.m_kind == AuthScreen::Change
    {
        this.mc().displayGuiScreen(None);
        return;
    }

    ext.m_handler.disconnect();
    this.mc().displayGuiScreen(Some(crate::gui::GuiMainMenu::new()));
}

fn focusNext(this: &GuiScreen)
{
    let ext: &AuthExt = fields(this);
    let list: std::cell::Ref<'_, Vec<Rc<GuiTextField>>> = ext.m_fields.borrow();
    let current: usize = list.iter().position(|field| field.m_isFocused.get()).unwrap_or(list.len() - 1);

    for (i, field) in list.iter().enumerate()
    {
        field.setFocused(i == (current + 1) % list.len());
    }
}

const CLASS: GuiScreenClass = GuiScreenClass
{
    m_name: "GuiAuth",
    m_parent: Some(&GUI_SCREEN),
    m_interfaces: &[],
    updateScreen: |this|
    {
        let ext: &AuthExt = fields(this);
        for field in ext.m_fields.borrow().iter()
        {
            field.updateCursorCounter();
        }

        // before play nothing else drives the connection and the socket times out after 30 idle seconds
        if !ext.m_handler.playing()
        {
            ext.m_ticks.set(ext.m_ticks.get() + 1);

            if ext.m_ticks.get() % KEEP_ALIVE_TICKS == 0
            {
                ext.m_handler.addToSendQueue(Box::new(Packet0KeepAlive::default()));
            }

            ext.m_handler.processReadPackets();
        }
    },

    initGui: |this|
    {
        Keyboard::enableRepeatEvents(true);

        let ext: &AuthExt = fields(this);
        let x: i32 = this.m_width.get() / 2 - layout::FIELD_WIDTH / 2;
        let entries: &[Entry] = ext.m_kind.entries();
        let mut built: Vec<Rc<GuiTextField>> = Vec::new();

        for (i, entry) in entries.iter().enumerate()
        {
            let y: i32 = top(this) + i as i32 * layout::ROW + layout::LABEL_GAP;
            let field: GuiTextField =
                GuiTextField::new(this.rc(), this.fontRenderer(), x, y, layout::FIELD_WIDTH, layout::FIELD_HEIGHT, "");
            field.setMaxStringLength(layout::MAX_LENGTH);
            field.m_masked.set(entry.m_masked);
            field.m_isFocused.set(i == 0);
            built.push(Rc::new(field));
        }
        *ext.m_fields.borrow_mut() = built;

        let buttonY: i32 = top(this) + entries.len() as i32 * layout::ROW + layout::LABEL_GAP + layout::FIELD_HEIGHT;

        this.m_controlList.clear();
        this.m_controlList.add(GuiButton::new(buttons::DONE, x, buttonY, "Done"));
        this.m_controlList.add(GuiButton::new(buttons::CANCEL, x, buttonY + 24, "Cancel"));
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
            buttons::DONE => submit(this),
            buttons::CANCEL => cancel(this),
            _ =>
            {}
        }
    },

    keyTyped: |this, chr, key| match key
    {
        Keyboard::KEY_ESCAPE => cancel(this),
        Keyboard::KEY_TAB => focusNext(this),
        Keyboard::KEY_RETURN => submit(this),

        _ =>
        {
            for field in fields(this).m_fields.borrow().iter()
            {
                field.textboxKeyTyped(chr, key);
            }
        }
    },

    mouseClicked: |this, mouseX, mouseY, button|
    {
        (GUI_SCREEN.mouseClicked)(this, mouseX, mouseY, button);

        for field in fields(this).m_fields.borrow().iter()
        {
            field.mouseClicked(mouseX, mouseY, button);
        }
    },

    drawScreen: |this, mouseX, mouseY, tickDelta|
    {
        this.drawDefaultBackground();

        let ext: &AuthExt = fields(this);
        let font: Rc<FontRenderer> = this.fontRenderer();
        let center: i32 = this.m_width.get() / 2;
        let x: i32 = center - layout::FIELD_WIDTH / 2;

        this.m_gui.drawCenteredString(&font, ext.m_kind.title(), center, top(this) - 40, TEXT_COLOR);

        let key: String = format!("Server key {}", ext.m_fingerprint);
        this.m_gui.drawCenteredString(&font, &key, center, top(this) - 26, LABEL_COLOR);

        for (i, entry) in ext.m_kind.entries().iter().enumerate()
        {
            this.m_gui.drawString(&font, entry.m_label, x, top(this) + i as i32 * layout::ROW, LABEL_COLOR);
        }

        for field in ext.m_fields.borrow().iter()
        {
            field.drawTextBox();
        }

        if let Some((status, error)) = ext.m_status.get()
        {
            let color: i32 = if error { ERROR_COLOR } else { TEXT_COLOR };
            this.m_gui.drawCenteredString(&font, &status, center, top(this) - 12, color);
        }

        (GUI_SCREEN.drawScreen)(this, mouseX, mouseY, tickDelta);
    },
    ..GUI_SCREEN
};

pub struct GuiAuth;

impl GuiAuth
{
    pub fn new(handler: &Rc<NetClientHandler>, kind: AuthScreen, fingerprint: &str) -> Rc<GuiScreen>
    {
        let ext: AuthExt = AuthExt
        {
            m_handler: Rc::clone(handler),
            m_kind: kind,
            m_fingerprint: fingerprint.to_string(),
            m_fields: RefCell::new(Vec::new()),
            m_status: Field::new(None),
            m_sent: Cell::new(false),
            m_ticks: Cell::new(0),
        };
        GuiScreen::construct(&CLASS, Box::new(ext))
    }

    pub fn showing(screen: &GuiScreen) -> bool { screen.m_class.m_name == CLASS.m_name }
}
