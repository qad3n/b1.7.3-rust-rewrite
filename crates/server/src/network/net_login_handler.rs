use crate::network::NetServerHandler;
use crate::player::entity_player_mp;
use crate::server::MinecraftServer;
use game::entity::{Entity, entity_player};
use game::network::net_handler::NetHandler;
use game::network::network_manager::NetworkManager;
use game::network::packet::Packet;
use game::network::packets::{
    Packet1Login, Packet2Handshake, Packet3Chat, Packet4UpdateTime, Packet6SpawnPosition, Packet250ModPayload,
    Packet255KickDisconnect,
};
use game::world::{ChunkCoordinates, World};
use jvm::field::Field;
use jvm::floating_decimal::doubleToString;
use jvm::io::IoResult;
use jvm::random::Random;
use jvm::string::JString;
use jvm::throwable::{OrNpe, Throwable};
use jvm::{debug, info};
use modloader::netsync::{Outcome, ServerSession};
use std::cell::{Cell, RefCell};
use std::net::TcpStream;
use std::rc::{Rc, Weak};
use std::sync::{Arc, Mutex};

thread_local! {
    static RAND: Random = Random::new();
}

enum VerifyOutcome
{
    Verified,
    Kick(String, Option<Throwable>),
}

pub struct NetLoginHandler
{
    m_self: Weak<NetLoginHandler>,
    pub m_netManager: Field<Option<Rc<NetworkManager>>>,
    pub m_finishedProcessing: Cell<bool>,
    m_mcServer: Rc<MinecraftServer>,
    m_loginTimer: Cell<i32>,
    m_username: Field<Option<JString>>,
    m_packet1login: Field<Option<Packet1Login>>,
    m_serverId: Field<String>,
    m_verification: Arc<Mutex<Option<VerifyOutcome>>>,
    m_pendingLogin: Field<Option<Packet1Login>>,
    // the mod loader configuration phase of a modded client, between auth and play
    m_session: RefCell<Option<ServerSession>>,
    m_configured: Cell<bool>,
    m_configuringLogin: Field<Option<Packet1Login>>,
}

// a slow registry remap may take longer than the vanilla 600 tick login window
const CONFIGURATION_TIMEOUT: i32 = 6000;

impl NetLoginHandler
{
    pub fn new(server: &Rc<MinecraftServer>, socket: TcpStream, name: &str) -> IoResult<Rc<NetLoginHandler>>
    {
        let handler: Rc<NetLoginHandler> = Rc::new_cyclic(|this| NetLoginHandler {
            m_self: this.clone(),
            m_netManager: Field::new(None),
            m_finishedProcessing: Cell::new(false),
            m_mcServer: Rc::clone(server),
            m_loginTimer: Cell::new(0),
            m_username: Field::new(None),
            m_packet1login: Field::new(None),
            m_serverId: Field::new(String::new()),
            m_verification: Arc::new(Mutex::new(None)),
            m_pendingLogin: Field::new(None),
            m_session: RefCell::new(None),
            m_configured: Cell::new(false),
            m_configuringLogin: Field::new(None),
        });
        let listener: Weak<dyn NetHandler> = handler.m_self.clone();
        let netManager: Rc<NetworkManager> = NetworkManager::new(socket, name, listener, true)?;
        netManager.setChunkDataSendCounter(0);
        handler.m_netManager.set(Some(netManager));
        Ok(handler)
    }

    pub fn netManager(&self) -> Rc<NetworkManager> { self.m_netManager.get().orNpe() }

    fn pollVerification(&self)
    {
        let outcome: Option<VerifyOutcome> = game::port::network::lock(&self.m_verification).take();
        match outcome
        {
            Some(VerifyOutcome::Verified) => self.m_packet1login.set(self.m_pendingLogin.get()),
            Some(VerifyOutcome::Kick(reason, exception)) =>
            {
                self.kickUser(&reason);
                if let Some(exception) = exception
                {
                    exception.printStackTrace();
                }
            }
            None =>
            {}
        }
    }

    pub fn tryLogin(&self)
    {
        // the verifier thread cannot touch rc game state so its result is applied here on the server thread
        self.pollVerification();
        if let Some(packet1login) = self.m_packet1login.get()
        {
            self.doLogin(&packet1login);
            self.m_packet1login.set(None);
        }
        let loginTimer: i32 = self.m_loginTimer.get();
        self.m_loginTimer.set(loginTimer + 1);
        let timeout: i32 = if self.m_session.borrow().is_some()
        {
            CONFIGURATION_TIMEOUT
        }
        else
        {
            600
        };
        if loginTimer == timeout
        {
            self.kickUser("Took too long to log in");
        }
        else
        {
            self.netManager().processReadPackets();
        }
    }

    pub fn kickUser(&self, reason: &str)
    {
        let result: Result<(), Throwable> = jvm::throwable::catchThrowable(&jvm::throwable::EXCEPTION, || {
            info!("Disconnecting {}: {reason}", self.getUserAndIPString());
            self.netManager().addToSendQueue(Box::new(Packet255KickDisconnect::new(JString::from(reason))));
            self.netManager().close();
            self.m_finishedProcessing.set(true);
        });
        if let Err(exception) = result
        {
            exception.printStackTrace();
        }
    }

    fn rawSender(&self) -> impl Fn(u32, Vec<u8>) + use<>
    {
        let netManager: Rc<NetworkManager> = self.netManager();
        move |channel: u32, body: Vec<u8>| netManager.addToSendQueue(Box::new(Packet250ModPayload::new(channel, body)))
    }

    // true when the mod loader holds the login back for its configuration phase or refused it
    fn configure(&self, packet: &Packet1Login) -> bool
    {
        if !modloader::play::active() || self.m_configured.get()
        {
            return false;
        }
        if packet.m_mapSeed != modloader::netsync::LOADER_MAGIC
        {
            let who: String = self.getUserAndIPString();
            modloader::log::info(modloader::log::LOADER, &format!("{who} has no mod loader"));
            let problems: Vec<String> = modloader::netsync::vanillaClientProblems();
            if problems.is_empty()
            {
                return false;
            }
            self.kickUser(&modloader::netsync::kickMessage(&problems));
            return true;
        }
        *self.m_session.borrow_mut() = Some(ServerSession::start(&self.rawSender(), self.getUserAndIPString()));
        self.m_configuringLogin.set(Some(packet.clone()));
        true
    }

    pub fn doLogin(&self, packet: &Packet1Login)
    {
        if self.configure(packet)
        {
            return;
        }
        let entityPlayerMP: Option<Rc<Entity>> =
            self.m_mcServer.configManager().login(self, packet.m_username.as_ref());
        if let Some(entityPlayerMP) = entityPlayerMP
        {
            self.m_mcServer.configManager().readPlayerDataFromFile(&entityPlayerMP);
            let dimension: i32 = entity_player::fields(&entityPlayerMP).m_dimension.get();
            entityPlayerMP.setWorld(Some(self.m_mcServer.getWorldManager(dimension)));
            info!(
                "{} logged in with entity id {} at ({}, {}, {})",
                self.getUserAndIPString(),
                entityPlayerMP.m_entityId.get(),
                doubleToString(entityPlayerMP.m_posX.get()),
                doubleToString(entityPlayerMP.m_posY.get()),
                doubleToString(entityPlayerMP.m_posZ.get())
            );
            let dimension: i32 = entity_player::fields(&entityPlayerMP).m_dimension.get();
            let worldServer: Rc<World> = self.m_mcServer.getWorldManager(dimension);
            let chunkCoordinates: ChunkCoordinates = worldServer.getSpawnPoint();
            let netServerHandler: Rc<NetServerHandler> =
                NetServerHandler::new(&self.m_mcServer, &self.netManager(), &entityPlayerMP);
            netServerHandler.sendPacket(Box::new(Packet1Login::withSeed(
                Some(JString::from("")),
                entityPlayerMP.m_entityId.get(),
                worldServer.getRandomSeed(),
                worldServer.worldProvider().m_worldType.get() as i8,
            )));
            netServerHandler.sendPacket(Box::new(Packet6SpawnPosition::new(
                chunkCoordinates.m_x,
                chunkCoordinates.m_y,
                chunkCoordinates.m_z,
            )));
            self.m_mcServer.configManager().sendWorldInfo(&entityPlayerMP, &worldServer);
            self.m_mcServer.configManager().sendPacketToAllPlayers(Box::new(Packet3Chat::new(JString::from(
                format!(
                    "\u{a7}e{} joined the game.",
                    JString::display(entity_player::fields(&entityPlayerMP).m_username.get().as_ref())
                )
                .as_str(),
            ))));
            self.m_mcServer.configManager().playerLoggedIn(&entityPlayerMP);
            netServerHandler.teleportTo(
                entityPlayerMP.m_posX.get(),
                entityPlayerMP.m_posY.get(),
                entityPlayerMP.m_posZ.get(),
                entityPlayerMP.m_rotationYaw.get(),
                entityPlayerMP.m_rotationPitch.get(),
            );
            self.m_mcServer.networkServer().addPlayer(&netServerHandler);
            // a configured modded client gets its mod channels for play
            if let Some(session) = self.m_session.borrow().as_ref()
            {
                let handler: Weak<NetServerHandler> = Rc::downgrade(&netServerHandler);
                let raw: Rc<dyn Fn(u32, Vec<u8>)> = Rc::new(move |channel: u32, body: Vec<u8>| {
                    if let Some(handler) = handler.upgrade()
                    {
                        handler.sendPacket(Box::new(Packet250ModPayload::new(channel, body)));
                    }
                });
                modloader::netsync::serverPlayerJoined(entityPlayerMP.m_entityId.get(), session, raw);
            }
            netServerHandler.sendPacket(Box::new(Packet4UpdateTime::new(worldServer.getWorldTime())));
            entity_player_mp::initMenu(&entityPlayerMP);
        }
        self.m_finishedProcessing.set(true);
    }

    pub fn getUserAndIPString(&self) -> String
    {
        if let Some(username) = self.m_username.get()
        {
            return format!("{username} [{}]", self.netManager().getRemoteAddress());
        }
        self.netManager().getRemoteAddress()
    }

    fn verify(username: &str, serverId: &str) -> Result<bool, Throwable>
    {
        let url: String = format!(
            "http://www.minecraft.net/game/checkserver.jsp?user={}&serverId={}",
            jvm::net::urlEncode(username),
            jvm::net::urlEncode(serverId)
        );
        let bytes: Vec<u8> = jvm::net::openStream(&url)?;
        let line: String =
            game::port::text::decodePlatform(&bytes).split(['\n', '\r']).next().unwrap_or_default().to_string();
        Ok(line == "YES")
    }
}

impl NetHandler for NetLoginHandler
{
    fn isServerHandler(&self) -> bool { true }

    fn registerPacket(&self, _packet: &dyn Packet) { self.kickUser("Protocol error"); }

    fn handleErrorMessage(&self, _reason: &JString, _args: &[JString])
    {
        info!("{} lost connection", self.getUserAndIPString());
        self.m_finishedProcessing.set(true);
    }

    fn handleHandshake(&self, _packet: &Packet2Handshake)
    {
        debug!("Handshake from {}", self.getUserAndIPString());
        if self.m_mcServer.m_onlineMode.get()
        {
            let serverId: String = format!("{:x}", RAND.with(Random::nextLong) as u64);
            self.m_serverId.set(serverId.clone());
            self.netManager().addToSendQueue(Box::new(Packet2Handshake::new(JString::from(serverId.as_str()))));
        }
        else
        {
            self.netManager().addToSendQueue(Box::new(Packet2Handshake::new(JString::from("-"))));
        }
    }

    fn handleModPayload(&self, packet: &Packet250ModPayload)
    {
        let sender = self.rawSender();
        let outcome: Option<Outcome> = self
            .m_session
            .borrow_mut()
            .as_mut()
            .map(|session| session.receive(packet.m_channel, &packet.m_data, &sender));
        match outcome
        {
            None => self.kickUser("Protocol error"),
            Some(Outcome::Continue) =>
            {}
            Some(Outcome::Kick(reason)) => self.kickUser(&reason),
            Some(Outcome::Done) =>
            {
                self.m_configured.set(true);
                if let Some(login) = self.m_configuringLogin.get()
                {
                    self.doLogin(&login);
                }
            }
        }
    }

    fn handleLogin(&self, packet: &Packet1Login)
    {
        self.m_username.set(packet.m_username.clone());
        if packet.m_protocolVersion != 14
        {
            if packet.m_protocolVersion > 14
            {
                self.kickUser("Outdated server!");
            }
            else
            {
                self.kickUser("Outdated client!");
            }
            return;
        }
        if !self.m_mcServer.m_onlineMode.get()
        {
            self.doLogin(packet);
        }
        else
        {
            self.m_pendingLogin.set(Some(packet.clone()));
            let username: String = JString::display(packet.m_username.as_ref());
            let serverId: String = self.m_serverId.get();
            let outcome: Arc<Mutex<Option<VerifyOutcome>>> = Arc::clone(&self.m_verification);
            let started: std::io::Result<std::thread::JoinHandle<()>> =
                jvm::thread::start(jvm::thread::nextName(), move || {
                    let result: VerifyOutcome = match NetLoginHandler::verify(&username, &serverId)
                    {
                        Ok(true) => VerifyOutcome::Verified,
                        Ok(false) => VerifyOutcome::Kick(String::from("Failed to verify username!"), None),
                        Err(exception) => VerifyOutcome::Kick(
                            format!("Failed to verify username! [internal error {exception}]"),
                            Some(exception),
                        ),
                    };
                    *game::port::network::lock(&outcome) = Some(result);
                });
            if let Err(error) = started
            {
                jvm::io::fromStd(&error).printStackTrace();
            }
        }
    }
}
