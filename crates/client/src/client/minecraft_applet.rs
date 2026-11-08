use crate::client::{Canvas, Minecraft, Session};
use jvm::field::Field;
use std::cell::Cell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::atomic::Ordering;

pub struct MinecraftApplet
{
    m_mcCanvas: Field<Option<Rc<Canvas>>>,
    m_mc: Field<Option<Rc<Minecraft>>>,
    m_mcThread: Cell<bool>,
    m_parameters: HashMap<String, String>,
    m_documentHost: String,
    m_documentPort: i32,
    m_width: i32,
    m_height: i32,
    m_active: Cell<bool>,
}

impl MinecraftApplet
{
    pub fn new(
        parameters: HashMap<String, String>,
        documentHost: String,
        documentPort: i32,
        width: i32,
        height: i32,
    ) -> Rc<MinecraftApplet>
    {
        Rc::new(MinecraftApplet {
            m_mcCanvas: Field::new(None),
            m_mc: Field::new(None),
            m_mcThread: Cell::new(false),
            m_parameters: parameters,
            m_documentHost: documentHost,
            m_documentPort: documentPort,
            m_width: width,
            m_height: height,
            m_active: Cell::new(true),
        })
    }

    pub fn getParameter(&self, name: &str) -> Option<String> { self.m_parameters.get(name).cloned() }

    pub fn isActive(&self) -> bool { self.m_active.get() }

    pub fn init(self: &Rc<Self>)
    {
        let canvas: Rc<Canvas> = Rc::new(Canvas::new(self.m_width, self.m_height));
        self.m_mcCanvas.set(Some(Rc::clone(&canvas)));
        let mut bl: bool = false;
        if let Some(fullscreen) = self.getParameter("fullscreen")
        {
            bl = fullscreen.eq_ignore_ascii_case("true");
        }
        let mc: Rc<Minecraft> = Minecraft::new(Some(canvas), Some(Rc::clone(self)), self.m_width, self.m_height, bl);
        self.m_mc.set(Some(Rc::clone(&mc)));
        let mut uri: String = self.m_documentHost.clone();
        if self.m_documentPort > 0
        {
            uri += &format!(":{}", self.m_documentPort);
        }
        mc.m_minecraftUri.set(Some(uri));
        match (self.getParameter("username"), self.getParameter("sessionid"))
        {
            (Some(username), Some(sessionId)) =>
            {
                let session: Session = Session::new(Some(username), Some(sessionId));
                jvm::system::println(&format!(
                    "Setting user: {}, {}",
                    session.m_username.get().unwrap_or_default(),
                    session.m_sessionId.get().unwrap_or_default()
                ));
                if let Some(mpPass) = self.getParameter("mppass")
                {
                    session.m_mpPassParameter.set(Some(mpPass));
                }
                mc.m_session.set(Some(Rc::new(session)));
            }
            _ => mc.m_session.set(Some(Rc::new(Session::new(Some("Player".to_owned()), Some(String::new()))))),
        }
        if let (Some(server), Some(port)) = (self.getParameter("server"), self.getParameter("port"))
        {
            let port: i32 = jvm::integer::parseInt(&jvm::string::JString::from(port.as_str()))
                .unwrap_or_else(|e| jvm::throwable::rethrow(e));
            mc.setServer(&server, port);
        }
        mc.m_hideQuitButton.set(true);
    }

    pub fn startMainThread(&self)
    {
        if self.m_mcThread.get()
        {
            return;
        }
        self.m_mcThread.set(true);
        if let Some(mc) = self.m_mc.get()
        {
            // the applet runs the game on the calling thread instead of starting one
            jvm::thread::setCurrentName(jvm::string::JString::from("Minecraft main thread"));
            mc.run();
        }
    }

    pub fn start(&self)
    {
        self.m_active.set(true);
        if let Some(mc) = self.m_mc.get()
        {
            mc.m_isGamePaused.store(false, Ordering::Relaxed);
        }
    }

    pub fn stop(&self)
    {
        self.m_active.set(false);
        if let Some(mc) = self.m_mc.get()
        {
            mc.m_isGamePaused.store(true, Ordering::Relaxed);
        }
    }

    pub fn destroy(&self) { self.shutdown(); }

    pub fn shutdown(&self)
    {
        if !self.m_mcThread.get()
        {
            return;
        }
        if let Some(mc) = self.m_mc.get()
        {
            mc.shutdown();
        }
        self.m_mcThread.set(false);
    }

    pub fn clearApplet(&self)
    {
        self.m_mcCanvas.set(None);
        self.m_mc.set(None);
        self.m_mcThread.set(false);
    }
}
