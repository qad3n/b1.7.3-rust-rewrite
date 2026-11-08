use crate::client::Minecraft;
use crate::gui::gui_screen::{GUI_SCREEN, GuiScreenClass};
use crate::gui::{FontRenderer, GuiButton, GuiConnectFailed, GuiMainMenu, GuiScreen};
use crate::network::net_client_handler::{CONNECT, NetClientHandler, UNKNOWN_HOST};
use game::locale::StringTranslate;
use game::network::packets::Packet2Handshake;
use jvm::field::Field;
use jvm::string::JString;
use jvm::throwable::{OrNpe, Throwable};
use std::net::TcpStream;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, PoisonError};
use std::thread::JoinHandle;

type Connection = Result<TcpStream, Throwable>;
type HandlerResult = Result<Rc<NetClientHandler>, Throwable>;

struct ConnectingExt
{
    m_clientHandler: Field<Option<Rc<NetClientHandler>>>,
    m_cancelled: Arc<AtomicBool>,
    m_connection: Arc<Mutex<Option<Connection>>>,
    m_minecraft: Rc<Minecraft>,
    m_address: String,
}

fn fields(this: &GuiScreen) -> &ConnectingExt { this.ext::<ConnectingExt>() }

fn finishConnecting(ext: &ConnectingExt, connection: Connection)
{
    let minecraft: &Rc<Minecraft> = &ext.m_minecraft;
    let handler: HandlerResult = connection.and_then(|socket| NetClientHandler::fromSocket(minecraft, socket));
    let error: Throwable = match handler
    {
        Ok(handler) =>
        {
            ext.m_clientHandler.set(Some(Rc::clone(&handler)));
            if ext.m_cancelled.load(Ordering::SeqCst)
            {
                return;
            }
            let username: Option<String> = minecraft.m_session.get().orNpe().m_username.get();
            handler.addToSendQueue(Box::new(Packet2Handshake::new(JString::from(username.orNpe().as_str()))));
            return;
        }
        Err(error) => error,
    };
    if ext.m_cancelled.load(Ordering::SeqCst)
    {
        return;
    }
    let detail: String = if error.instanceOf(&UNKNOWN_HOST)
    {
        format!("Unknown host '{}'", ext.m_address)
    }
    else if error.instanceOf(&CONNECT)
    {
        error.getMessage().unwrap_or("null").to_string()
    }
    else
    {
        error.printStackTrace();
        error.to_string()
    };
    minecraft.displayGuiScreen(Some(GuiConnectFailed::new(
        "connect.failed",
        "disconnect.genericReason",
        &[JString::from(detail.as_str())],
    )));
}

const CLASS: GuiScreenClass = GuiScreenClass {
    m_name: "GuiConnecting",
    m_parent: Some(&GUI_SCREEN),
    m_interfaces: &[],
    updateScreen: |this| {
        let ext: &ConnectingExt = fields(this);
        let connection: Option<Connection> = ext.m_connection.lock().unwrap_or_else(PoisonError::into_inner).take();
        if let Some(connection) = connection
        {
            // builds the handler on the main thread where java builds it on the connect thread
            finishConnecting(ext, connection);
        }
        if let Some(handler) = ext.m_clientHandler.get()
        {
            handler.processReadPackets();
        }
    },
    keyTyped: |_, _, _| {},
    initGui: |this| {
        let stringTranslate: Rc<StringTranslate> = StringTranslate::getInstance();
        this.m_controlList.clear();
        this.m_controlList.add(GuiButton::new(
            0,
            this.m_width.get() / 2 - 100,
            this.m_height.get() / 4 + 120 + 12,
            &stringTranslate.translateKey("gui.cancel"),
        ));
    },
    actionPerformed: |this, button| {
        if button.m_id.get() == 0
        {
            let ext: &ConnectingExt = fields(this);
            ext.m_cancelled.store(true, Ordering::SeqCst);
            if let Some(handler) = ext.m_clientHandler.get()
            {
                handler.disconnect();
            }
            this.mc().displayGuiScreen(Some(GuiMainMenu::new()));
        }
    },
    drawScreen: |this, mouseX, mouseY, tickDelta| {
        this.drawDefaultBackground();
        let stringTranslate: Rc<StringTranslate> = StringTranslate::getInstance();
        let font: Rc<FontRenderer> = this.fontRenderer();
        let center: i32 = this.m_width.get() / 2;
        let y: i32 = this.m_height.get() / 2;
        match fields(this).m_clientHandler.get()
        {
            None =>
            {
                this.m_gui.drawCenteredString(
                    &font,
                    &stringTranslate.translateKey("connect.connecting"),
                    center,
                    y - 50,
                    0xFFFFFF,
                );
                this.m_gui.drawCenteredString(&font, "", center, y - 10, 0xFFFFFF);
            }
            Some(handler) =>
            {
                this.m_gui.drawCenteredString(
                    &font,
                    &stringTranslate.translateKey("connect.authorizing"),
                    center,
                    y - 50,
                    0xFFFFFF,
                );
                if let Some(message) = handler.m_message.get()
                {
                    this.m_gui.drawCenteredString(&font, &message.to_string(), center, y - 10, 0xFFFFFF);
                }
            }
        }
        (GUI_SCREEN.drawScreen)(this, mouseX, mouseY, tickDelta);
    },
    ..GUI_SCREEN
};

pub struct GuiConnecting;

impl GuiConnecting
{
    pub fn new(minecraft: &Rc<Minecraft>, address: &str, port: i32) -> Rc<GuiScreen>
    {
        jvm::system::println(&format!("Connecting to {address}, {port}"));
        minecraft.changeWorld1(None);
        let ext: ConnectingExt = ConnectingExt {
            m_clientHandler: Field::new(None),
            m_cancelled: Arc::new(AtomicBool::new(false)),
            m_connection: Arc::new(Mutex::new(None)),
            m_minecraft: Rc::clone(minecraft),
            m_address: address.to_string(),
        };
        let connection: Arc<Mutex<Option<Connection>>> = Arc::clone(&ext.m_connection);
        let host: String = address.to_string();
        // the thread only opens the socket since Rc game state cannot cross threads
        let started: std::io::Result<JoinHandle<()>> = jvm::thread::start(jvm::thread::nextName(), move || {
            let result: Connection = NetClientHandler::connect(&host, port);
            *connection.lock().unwrap_or_else(PoisonError::into_inner) = Some(result);
        });
        if let Err(error) = started
        {
            jvm::io::fromStd(&error).printStackTrace();
        }
        GuiScreen::construct(&CLASS, Box::new(ext))
    }
}
