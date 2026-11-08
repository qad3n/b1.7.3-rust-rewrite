use crate::client::{Canvas, Frame, Minecraft, PanelCrashReport, Session, UnexpectedThrowable};
use jvm::string::JString;
use std::rc::Rc;

mod window
{
    pub const WIDTH: i32 = 854;
    pub const HEIGHT: i32 = 480;
}

fn guestName() -> String { format!("Player{}", jvm::system::currentTimeMillis() % 1000) }

impl Minecraft
{
    pub fn displayUnexpectedThrowable(&self, report: &UnexpectedThrowable) { PanelCrashReport::new(report); }

    pub fn start(username: Option<String>, sessionId: Option<String>)
    {
        Minecraft::startMainThread(username, sessionId, None);
    }

    pub fn startMainThread(username: Option<String>, sessionId: Option<String>, server: Option<String>)
    {
        let flag: bool = false;
        let s: Option<String> = username;
        // awt frame and canvas are stand-ins over the lwjgl window and the game runs on this thread
        let frame: Rc<Frame> = Rc::new(Frame);
        let canvas: Rc<Canvas> = Rc::new(Canvas::new(window::WIDTH, window::HEIGHT));
        let minecraft: Rc<Minecraft> = Minecraft::new(Some(canvas), None, window::WIDTH, window::HEIGHT, flag);
        minecraft.m_frame.set(Some(frame));
        minecraft.m_minecraftUri.set(Some("www.minecraft.net".to_owned()));
        let session: Session = match (s, sessionId)
        {
            (Some(s), Some(sessionId)) => Session::new(Some(s), Some(sessionId)),
            _ => Session::new(Some(guestName()), Some(String::new())),
        };
        minecraft.m_session.set(Some(Rc::new(session)));
        if let Some(server) = server
        {
            let astring: Vec<&str> = server.split(':').collect();
            minecraft.setServer(
                astring[0],
                jvm::integer::parseInt(&JString::from(astring[1])).unwrap_or_else(|e| jvm::throwable::rethrow(e)),
            );
        }
        jvm::thread::setCurrentName(JString::from("Minecraft main thread"));
        minecraft.run();
    }

    pub fn main(args: &[String])
    {
        let mut string: String = guestName();
        if !args.is_empty()
        {
            string = args[0].clone();
        }
        let mut string2: String = "-".to_owned();
        if args.len() > 1
        {
            string2 = args[1].clone();
        }
        Minecraft::start(Some(string), Some(string2));
    }
}
