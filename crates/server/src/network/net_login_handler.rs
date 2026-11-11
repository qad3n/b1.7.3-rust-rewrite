use crate::network::NetServerHandler;
use crate::player::entity_player_mp;
use crate::server::{MinecraftServer, ServerAuth, Ticket};
use auth::{Message, Mode};
use game::entity::{Entity, entity_player};
use game::network::net_handler::NetHandler;
use game::network::network_manager::NetworkManager;
use game::network::packet::Packet;
use game::network::packets::
{
    Packet0KeepAlive, Packet1Login, Packet2Handshake, Packet3Chat, Packet4UpdateTime, Packet6SpawnPosition,
    Packet250ModPayload, Packet255KickDisconnect,
};
use game::world::{ChunkCoordinates, World};
use jvm::field::Field;
use jvm::floating_decimal::doubleToString;
use jvm::io::IoResult;
use jvm::string::JString;
use jvm::throwable::{OrNpe, Throwable};
use jvm::{debug, info, severe, warning};
use modloader::netsync::{Outcome, ServerSession};
use std::cell::{Cell, RefCell};
use std::net::TcpStream;
use std::rc::{Rc, Weak};
use std::time::Instant;

pub struct NetLoginHandler
{
    m_self: Weak<Self>,
    pub m_netManager: Field<Option<Rc<NetworkManager>>>,
    pub m_finishedProcessing: Cell<bool>,
    m_mcServer: Rc<MinecraftServer>,
    m_loginTimer: Cell<i32>,
    m_username: Field<Option<JString>>,
    m_packet1login: Field<Option<Packet1Login>>,
    // the mod loader configuration phase of a modded client, between auth and play
    m_session: RefCell<Option<ServerSession>>,
    m_configured: Cell<bool>,
    m_configuringLogin: Field<Option<Packet1Login>>,
    // the password phase of an auth server, between the login packet and play
    m_handshakeName: Field<Option<JString>>,
    m_loginSeen: Cell<bool>,
    m_auth: RefCell<Option<auth::Handshake>>,
    m_authLogin: Field<Option<Packet1Login>>,
    m_authed: Cell<bool>,
    m_ticket: RefCell<Option<Ticket>>,
}

// a slow registry remap may take longer than the vanilla 600 tick login window
const CONFIGURATION_TIMEOUT: i32 = 6000;
// two minutes to type a password
const AUTH_TIMEOUT: i32 = 2400;
const KEEP_ALIVE_TICKS: i32 = 20;
const PROTOCOL_ERROR: &str = "Protocol error";

impl NetLoginHandler
{
    pub fn new(server: &Rc<MinecraftServer>, socket: TcpStream, name: &str) -> IoResult<Rc<Self>>
    {
        let handler: Rc<Self> = Rc::new_cyclic(|this| Self
        {
            m_self: this.clone(),
            m_netManager: Field::new(None),
            m_finishedProcessing: Cell::new(false),
            m_mcServer: Rc::clone(server),
            m_loginTimer: Cell::new(0),
            m_username: Field::new(None),
            m_packet1login: Field::new(None),
            m_session: RefCell::new(None),
            m_configured: Cell::new(false),
            m_configuringLogin: Field::new(None),
            m_handshakeName: Field::new(None),
            m_loginSeen: Cell::new(false),
            m_auth: RefCell::new(None),
            m_authLogin: Field::new(None),
            m_authed: Cell::new(false),
            m_ticket: RefCell::new(None),
        });
        let listener: Weak<dyn NetHandler> = handler.m_self.clone();
        let netManager: Rc<NetworkManager> = NetworkManager::new(socket, name, listener, true)?;
        netManager.setChunkDataSendCounter(0);
        handler.m_netManager.set(Some(netManager));
        Ok(handler)
    }

    pub fn netManager(&self) -> Rc<NetworkManager> { self.m_netManager.get().orNpe() }

    pub fn tryLogin(&self)
    {
        if let Some(packet1login) = self.m_packet1login.get()
        {
            self.doLogin(&packet1login);
            self.m_packet1login.set(None);
        }

        let loginTimer: i32 = self.m_loginTimer.get();
        self.m_loginTimer.set(loginTimer + 1);

        // the socket read timeout is 30 seconds and a player may take longer to type a password
        if self.m_auth.borrow().is_some() && loginTimer % KEEP_ALIVE_TICKS == 0
        {
            self.netManager().addToSendQueue(Box::new(Packet0KeepAlive::default()));
        }

        let timeout: i32 = if self.m_session.borrow().is_some()
        {
            CONFIGURATION_TIMEOUT
        }
        else if self.m_auth.borrow().is_some()
        {
            AUTH_TIMEOUT
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
        let result: Result<(), Throwable> = jvm::throwable::catchThrowable(&jvm::throwable::EXCEPTION, ||
        {
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

                let raw: Rc<dyn Fn(u32, Vec<u8>)> = Rc::new(move |channel: u32, body: Vec<u8>|
                {
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

    fn startAuth(&self, packet: &Packet1Login)
    {
        let service: Rc<ServerAuth> = self.m_mcServer.auth();

        if packet.m_dimension & auth::LOGIN_FLAG == 0
        {
            self.kickUser("This server needs the b1.7.3 rewrite client to log in with a password");
            return;
        }

        let Some(ip) = self.netManager().remoteIp()
        else
        {
            self.kickUser(PROTOCOL_ERROR);
            return;
        };

        match service.ticket(ip)
        {
            Ok(ticket) => *self.m_ticket.borrow_mut() = Some(ticket),

            Err(reason) =>
            {
                self.kickUser(&reason);
                return;
            }
        }

        let name: String = JString::display(packet.m_username.as_ref());
        let op: bool = self.m_mcServer.configManager().isOp(packet.m_username.as_ref());
        let joined: Result<(auth::Handshake, Message), String> =
            auth::Handshake::join(&service.vault(), &name, service.policy(), op);

        let (handshake, start): (auth::Handshake, Message) = match joined
        {
            Ok(joined) => joined,

            Err(reason) =>
            {
                self.kickUser(&reason);
                return;
            }
        };

        if let Err(reason) = Self::admit(&service, ip, handshake.name(), &start)
        {
            self.kickUser(&reason);
            return;
        }

        info!("{} is logging in with a password", self.getUserAndIPString());
        *self.m_auth.borrow_mut() = Some(handshake);
        self.m_authLogin.set(Some(packet.clone()));
        self.netManager().addToSendQueue(Box::new(Packet250ModPayload::auth(&start)));
    }

    fn admit(service: &ServerAuth, ip: std::net::IpAddr, name: &str, start: &Message) -> Result<(), String>
    {
        let now: Instant = Instant::now();
        let mut limiter: std::cell::RefMut<'_, auth::Limiter> = service.limiter();
        limiter.attempt(ip, name, now)?;

        if matches!(start, Message::Start { mode: Mode::Register, .. })
        {
            limiter.register(ip, now)?;
        }

        Ok(())
    }

    fn authPayload(&self, data: &[u8])
    {
        let message: Message = match Message::decode(data)
        {
            Ok(message) => message,

            Err(error) =>
            {
                debug!("Bad auth message from {}: {error}", self.getUserAndIPString());
                self.kickUser(PROTOCOL_ERROR);
                return;
            }
        };

        // the read thread already switched to the session cipher at this marker
        if message == Message::Secure
        {
            if !self.m_authed.get()
            {
                self.kickUser(PROTOCOL_ERROR);
            }

            return;
        }

        let service: Rc<ServerAuth> = self.m_mcServer.auth();
        let mut current: std::cell::RefMut<'_, Option<auth::Handshake>> = self.m_auth.borrow_mut();
        let Some(handshake) = current.as_mut()
        else
        {
            drop(current);
            self.kickUser(PROTOCOL_ERROR);
            return;
        };

        let name: String = handshake.name().to_string();
        let display: String = handshake.display().to_string();
        let outcome: auth::Outcome = handshake.receive(&mut service.vault(), message);
        // the outcome may finish the login which reads this cell again
        drop(current);

        match outcome
        {
            auth::Outcome::Send(reply) =>
            {
                // counted before the proof so a client that walks away after ke2 still pays the backoff
                if matches!(reply, Message::Ke2(_))
                {
                    service.limiter().failed(&name, Instant::now());
                }

                self.netManager().addToSendQueue(Box::new(Packet250ModPayload::auth(&reply)));
            }

            auth::Outcome::Wait =>
            {}
            auth::Outcome::Authed(keys) => self.finishAuth(&service, &name, &display, keys),

            auth::Outcome::Fail(reason) =>
            {
                service.limiter().failed(&name, Instant::now());
                info!("{} failed to log in as {display}: {reason}", self.getUserAndIPString());
                self.kickUser(&reason);
            }

            auth::Outcome::Refuse(reason) =>
            {
                info!("{} was refused as {display}: {reason}", self.getUserAndIPString());
                self.kickUser(&reason);
            }

            auth::Outcome::Changed =>
            {
                severe!("A join handshake reported a password change for {display}");
                self.kickUser(PROTOCOL_ERROR);
            }
        }
    }

    fn traceAddress(&self, service: &ServerAuth, name: &str, display: &str)
    {
        let Some(ip) = self.netManager().remoteIp()
        else
        {
            return;
        };

        let address: String = ip.to_string();

        match service.vault().visit(name, &address)
        {
            Ok(Some(previous)) => warning!("{display} logged in from {address}, last login was from {previous}"),
            Ok(None) =>
            {}
            Err(error) => warning!("Could not save the login address of {display}: {error}"),
        }
    }

    fn finishAuth(&self, service: &ServerAuth, name: &str, display: &str, keys: auth::Keys)
    {
        *self.m_auth.borrow_mut() = None;
        service.limiter().succeeded(name);
        self.netManager().armSecure(keys);
        self.netManager().addToSendQueue(Box::new(Packet250ModPayload::auth(&Message::Secure)));
        self.m_authed.set(true);
        info!("{} logged in with a password as {display}", self.getUserAndIPString());
        self.traceAddress(service, name, display);

        let Some(mut login) = self.m_authLogin.get()
        else
        {
            self.kickUser(PROTOCOL_ERROR);
            return;
        };

        // the stored casing is the identity from here on so ops and bans always match one account
        login.m_username = Some(JString::from(display));
        self.m_username.set(login.m_username.clone());
        self.doLogin(&login);
    }

    pub fn getUserAndIPString(&self) -> String
    {
        if let Some(username) = self.m_username.get()
        {
            return format!("{username} [{}]", self.netManager().getRemoteAddress());
        }

        self.netManager().getRemoteAddress()
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

    fn handleHandshake(&self, packet: &Packet2Handshake)
    {
        self.m_handshakeName.set(packet.m_username.clone());
        debug!("Handshake from {}", self.getUserAndIPString());
        // no session server check so players prove their name with a password after the login packet
        self.netManager().addToSendQueue(Box::new(Packet2Handshake::new(JString::from("-"))));
    }

    fn handleModPayload(&self, packet: &Packet250ModPayload)
    {
        if packet.m_channel == auth::CHANNEL
        {
            self.authPayload(&packet.m_data);
            return;
        }

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

        if self.m_loginSeen.replace(true) || packet.m_username != self.m_handshakeName.get()
        {
            self.kickUser(PROTOCOL_ERROR);
            return;
        }

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

        self.startAuth(packet);
    }
}
