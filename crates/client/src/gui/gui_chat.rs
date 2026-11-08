use crate::client::Minecraft;
use crate::entity::entity_client_player_mp;
use crate::gui::GuiButton;
use crate::gui::gui_screen::{GUI_SCREEN, GuiScreen, GuiScreenClass};
use crate::gui::gui_text_field::isAllowedCharacter;
use crate::network::net_client_handler::NetClientHandler;
use game::entity::Entity;
use game::locale::StringTranslate;
use game::network::packets::packet19_entity_action::Packet19EntityAction;
use game::port::text;
use jvm::field::Field;
use jvm::string::JString;
use jvm::throwable::OrNpe;
use lwjgl::input::Keyboard;
use std::cell::Cell;
use std::rc::Rc;

pub struct GuiChat
{
    pub m_message: Field<JString>,
    m_updateCounter: Cell<i32>,
}

pub fn fields(this: &GuiScreen) -> &GuiChat { this.ext::<GuiChat>() }

fn keyTyped(this: &Rc<GuiScreen>, chr: u16, key: i32)
{
    let chat: &GuiChat = fields(this);
    let mc: Rc<Minecraft> = this.mc();
    if key == 1
    {
        mc.displayGuiScreen(None);
    }
    else if key == 28
    {
        let message: JString = chat.m_message.get();
        // port trim strips chars up to space like java String.trim
        if text::trim(&message).length() > 0
        {
            // port trim strips chars up to space like java String.trim
            let s1: String = text::trim(&message).to_string();
            if !mc.lineIsCommand(&s1)
            {
                mc.m_thePlayer.get().orNpe().sendChatMessage(&s1);
            }
        }
        mc.displayGuiScreen(None);
    }
    else
    {
        let message: JString = chat.m_message.get();
        if key == 14 && message.length() > 0
        {
            chat.m_message.set(message.substring(0, message.length() - 1));
        }
        let message: JString = chat.m_message.get();
        if isAllowedCharacter(chr) && message.length() < 100
        {
            // port appendChar appends one utf16 unit like java string concat with a char
            chat.m_message.set(text::appendChar(&message, chr));
        }
    }
}

pub const GUI_CHAT: GuiScreenClass = GuiScreenClass {
    m_name: "GuiChat",
    m_parent: Some(&GUI_SCREEN),
    m_interfaces: &[],
    initGui: |_| Keyboard::enableRepeatEvents(true),
    onGuiClosed: |_| Keyboard::enableRepeatEvents(false),
    updateScreen: |this| fields(this).m_updateCounter.set(fields(this).m_updateCounter.get() + 1),
    keyTyped,
    drawScreen: |this, mouseX, mouseY, tickDelta| {
        let chat: &GuiChat = fields(this);
        let height: i32 = this.m_height.get();
        this.m_gui.drawRect(2, height - 14, this.m_width.get() - 2, height - 2, i32::MIN);
        let cursor: &str = if chat.m_updateCounter.get() / 6 % 2 == 0
        {
            "_"
        }
        else
        {
            ""
        };
        let text: String = format!("> {}{cursor}", chat.m_message.get());
        this.m_gui.drawString(&this.fontRenderer(), &text, 4, height - 12, 14737632);
        (GUI_SCREEN.drawScreen)(this, mouseX, mouseY, tickDelta);
    },
    mouseClicked: |this, mouseX, mouseY, button| {
        if button != 0
        {
            return;
        }
        let selectedName: Option<String> = this.mc().m_ingameGUI.get().orNpe().m_selectedName.get();
        let Some(selectedName) = selectedName
        else
        {
            (GUI_SCREEN.mouseClicked)(this, mouseX, mouseY, button);
            return;
        };
        let chat: &GuiChat = fields(this);
        let mut message: JString = chat.m_message.get();
        if message.length() > 0 && message.chars().last() != Some(&(' ' as u16))
        {
            // port appendChar appends one utf16 unit like java string concat with a char
            message = text::appendChar(&message, ' ' as u16);
        }
        let mut chars: Vec<u16> = message.chars().to_vec();
        chars.extend(selectedName.encode_utf16());
        message = JString::fromUtf16(chars);
        let b0: i32 = 100;
        if message.length() > b0
        {
            message = message.substring(0, b0);
        }
        chat.m_message.set(message);
    },
    ..GUI_SCREEN
};

impl GuiChat
{
    fn ext() -> Box<GuiChat>
    {
        Box::new(GuiChat { m_message: Field::new(JString::new()), m_updateCounter: Cell::new(0) })
    }

    pub fn new() -> Rc<GuiScreen> { GuiScreen::construct(&GUI_CHAT, GuiChat::ext()) }
}

fn stopSleeping(this: &GuiScreen)
{
    let player: Rc<Entity> = this.mc().m_thePlayer.get().orNpe();
    if player.instanceOf("EntityClientPlayerMP")
    {
        let sendQueue: &Rc<NetClientHandler> = &entity_client_player_mp::fields(&player).m_sendQueue;
        sendQueue.addToSendQueue(Box::new(Packet19EntityAction::new(&player, 3)));
    }
}

pub const GUI_SLEEP_MP: GuiScreenClass = GuiScreenClass {
    m_name: "GuiSleepMP",
    m_parent: Some(&GUI_CHAT),
    m_interfaces: &[],
    initGui: |this| {
        Keyboard::enableRepeatEvents(true);
        let stringTranslate: Rc<StringTranslate> = StringTranslate::getInstance();
        this.m_controlList.add(GuiButton::new(
            1,
            this.m_width.get() / 2 - 100,
            this.m_height.get() - 40,
            &stringTranslate.translateKey("multiplayer.stopSleeping"),
        ));
    },
    onGuiClosed: |_| Keyboard::enableRepeatEvents(false),
    drawScreen: |this, mouseX, mouseY, tickDelta| (GUI_CHAT.drawScreen)(this, mouseX, mouseY, tickDelta),
    keyTyped: |this, chr, key| {
        if key == 1
        {
            stopSleeping(this);
        }
        else if key == 28
        {
            let chat: &GuiChat = fields(this);
            let message: JString = chat.m_message.get();
            // port trim strips chars up to space like java String.trim
            if text::trim(&message).length() > 0
            {
                // port trim strips chars up to space like java String.trim
                this.mc().m_thePlayer.get().orNpe().sendChatMessage(&text::trim(&message).to_string());
            }
            chat.m_message.set(JString::new());
        }
        else
        {
            (GUI_CHAT.keyTyped)(this, chr, key);
        }
    },
    actionPerformed: |this, button| {
        if button.m_id.get() == 1
        {
            stopSleeping(this);
        }
        else
        {
            (GUI_CHAT.actionPerformed)(this, button);
        }
    },
    ..GUI_CHAT
};

pub fn newSleepMP() -> Rc<GuiScreen> { GuiScreen::construct(&GUI_SLEEP_MP, GuiChat::ext()) }
