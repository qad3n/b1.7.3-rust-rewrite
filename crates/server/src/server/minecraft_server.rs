use crate::network::NetworkListenThread;
use crate::player::EntityTracker;
use crate::server::{
    ConsoleCommandHandler, ICommandListener, PropertyManager, ServerCommand, ServerConfigurationManager,
};
use crate::world::{IUpdatePlayerListBox, WorldManager, world_server, world_server_multi};
use common::world::IProgressUpdate;
use game::math::{AxisAlignedBB, Vec3D};
use game::network::packets::Packet4UpdateTime;
use game::storage::{ISaveFormat, SaveConverterMcRegion, SaveOldDir};
use game::world::{ChunkCoordinates, ISaveHandler, World};
use jvm::array_list::JArrayList;
use jvm::field::Field;
use jvm::hash_map::JHashMap;
use jvm::logging::{Level, logger, took};
use jvm::random::Random;
use jvm::string::JString;
use jvm::throwable::{self, OrNpe, Throwable, ThrowableClass};
use std::cell::{Cell, RefCell};
use std::net::{IpAddr, ToSocketAddrs};
use std::path::{Path, PathBuf};
use std::rc::{Rc, Weak};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

thread_local! {
    pub static GIVE_COMMANDS_COOLDOWNS: RefCell<JHashMap<JString, i32>> = RefCell::new(JHashMap::new());
}

pub struct MinecraftServer
{
    m_self: Weak<MinecraftServer>,
    pub m_networkServer: Field<Option<Rc<NetworkListenThread>>>,
    pub m_propertyManagerObj: Field<Option<Rc<PropertyManager>>>,
    pub m_worldMngr: Field<Option<Vec<Option<Rc<World>>>>>,
    pub m_configManager: Field<Option<Rc<ServerConfigurationManager>>>,
    m_commandHandler: Field<Option<Rc<ConsoleCommandHandler>>>,
    m_serverRunning: Arc<AtomicBool>,
    pub m_serverStopped: Arc<AtomicBool>,
    m_deathTime: Cell<i32>,
    pub m_currentTask: Field<Option<String>>,
    pub m_percentDone: Cell<i32>,
    m_tickables: JArrayList<Rc<dyn IUpdatePlayerListBox>>,
    m_commands: RefCell<Vec<ServerCommand>>,
    m_consoleLines: Arc<Mutex<Vec<String>>>,
    // one tracker per dimension, vanilla dimensions first and mod dimensions after them
    pub m_entityTracker: Field<Vec<Option<Rc<EntityTracker>>>>,
    pub m_onlineMode: Cell<bool>,
    pub m_spawnPeacefulMobs: Cell<bool>,
    pub m_pvpOn: Cell<bool>,
    pub m_allowFlight: Cell<bool>,
}

struct ConvertProgress
{
    m_lastTimeMillis: Cell<i64>,
}

impl IProgressUpdate for ConvertProgress
{
    fn progressStartNoAbort(&self, _title: &str) {}

    fn displayLoadingString(&self, _stage: &str) {}

    fn setLoadingProgress(&self, percentage: i32)
    {
        if jvm::system::currentTimeMillis() - self.m_lastTimeMillis.get() >= 1000
        {
            self.m_lastTimeMillis.set(jvm::system::currentTimeMillis());
            logger().info(&format!("Converting... {percentage}%"));
        }
    }
}

impl MinecraftServer
{
    pub fn new() -> Rc<MinecraftServer>
    {
        let server: Rc<MinecraftServer> = Rc::new_cyclic(|this| MinecraftServer {
            m_self: this.clone(),
            m_networkServer: Field::new(None),
            m_propertyManagerObj: Field::new(None),
            m_worldMngr: Field::new(None),
            m_configManager: Field::new(None),
            m_commandHandler: Field::new(None),
            m_serverRunning: Arc::new(AtomicBool::new(true)),
            m_serverStopped: Arc::new(AtomicBool::new(false)),
            m_deathTime: Cell::new(0),
            m_currentTask: Field::new(None),
            m_percentDone: Cell::new(0),
            m_tickables: JArrayList::new(),
            m_commands: RefCell::new(Vec::new()),
            m_consoleLines: Arc::new(Mutex::new(Vec::new())),
            m_entityTracker: Field::new(Vec::new()),
            m_onlineMode: Cell::new(false),
            m_spawnPeacefulMobs: Cell::new(false),
            m_pvpOn: Cell::new(false),
            m_allowFlight: Cell::new(false),
        });
        let sleeper: std::io::Result<std::thread::JoinHandle<()>> = jvm::thread::start(jvm::thread::nextName(), || {
            loop
            {
                std::thread::sleep(Duration::from_millis(2147483647));
            }
        });
        if let Err(error) = sleeper
        {
            jvm::io::fromStd(&error).printStackTrace();
        }
        server
    }

    fn this(&self) -> Rc<MinecraftServer> { self.m_self.upgrade().expect("the server outlives its own calls") }

    pub fn networkServer(&self) -> Rc<NetworkListenThread> { self.m_networkServer.get().orNpe() }

    pub fn propertyManager(&self) -> Rc<PropertyManager> { self.m_propertyManagerObj.get().orNpe() }

    pub fn configManager(&self) -> Rc<ServerConfigurationManager> { self.m_configManager.get().orNpe() }

    pub fn worldMngr(&self) -> Vec<Rc<World>> { self.m_worldMngr.get().orNpe().into_iter().map(OrNpe::orNpe).collect() }

    fn commandHandler(&self) -> Rc<ConsoleCommandHandler> { self.m_commandHandler.get().orNpe() }

    fn startServer(&self) -> Result<bool, Throwable>
    {
        let startup: Instant = Instant::now();
        let this: Rc<MinecraftServer> = self.this();
        self.m_commandHandler.set(Some(Rc::new(ConsoleCommandHandler::new(&this))));
        let consoleLines: Arc<Mutex<Vec<String>>> = Arc::clone(&self.m_consoleLines);
        let serverStopped: Arc<AtomicBool> = Arc::clone(&self.m_serverStopped);
        let serverRunning: Arc<AtomicBool> = Arc::clone(&self.m_serverRunning);
        // the console thread only reads lines and the server thread turns them into commands
        let started: std::io::Result<std::thread::JoinHandle<()>> =
            jvm::thread::start(jvm::thread::nextName(), move || {
                readConsole(consoleLines, serverStopped, serverRunning)
            });
        if let Err(error) = started
        {
            logger().severe(&format!("Could not start the console thread: {error}"));
        }
        logger().info("Starting minecraft server version Beta 1.7.3");
        logger().info("Loading properties");
        self.m_propertyManagerObj.set(Some(Rc::new(PropertyManager::new(Path::new("server.properties")))));
        // the built in mod loader reads its server.properties switches and loads mods next to that file
        crate::modded::start(&self.propertyManager());
        let s: String = self.propertyManager().getStringProperty("server-ip", "");
        self.m_onlineMode.set(self.propertyManager().getBooleanProperty("online-mode", true));
        self.m_spawnPeacefulMobs.set(self.propertyManager().getBooleanProperty("spawn-animals", true));
        self.m_pvpOn.set(self.propertyManager().getBooleanProperty("pvp", true));
        self.m_allowFlight.set(self.propertyManager().getBooleanProperty("allow-flight", false));
        let mut inetaddress: Option<IpAddr> = None;
        if !s.is_empty()
        {
            inetaddress = Some(getByName(&s)?);
        }
        let i: i32 = self.propertyManager().getIntProperty("server-port", 25565);
        logger().info(&format!("Starting Minecraft server on {}:{i}", if s.is_empty() { "*" } else { s.as_str() }));
        match NetworkListenThread::new(&this, inetaddress, i)
        {
            Ok(networkServer) => self.m_networkServer.set(Some(Rc::new(networkServer))),
            Err(ioexception) =>
            {
                logger().warning("**** FAILED TO BIND TO PORT!");
                logger().log(Level::WARNING, &format!("The exception was: {ioexception}"), None);
                logger().warning("Perhaps a server is already running on that port?");
                return Ok(false);
            }
        }
        if !self.m_onlineMode.get()
        {
            logger().warning("**** SERVER IS RUNNING IN OFFLINE/INSECURE MODE!");
            logger().warning("The server will make no attempt to authenticate usernames. Beware.");
            logger().warning(
                "While this makes the game possible to play without internet access, it also opens up the ability for hackers to connect with any username they choose.",
            );
            logger().warning("To change this, set \"online-mode\" to \"true\" in the server.settings file.");
        }
        // rewrite: the level's saved mod id map applies before the per dimension trackers and player managers exist
        modloader::remap::preload(Path::new(&self.propertyManager().getStringProperty("level-name", "world")));
        self.m_configManager.set(Some(ServerConfigurationManager::new(&this)));
        let trackers: Vec<Option<Rc<EntityTracker>>> = modloader::dimensions::serverDimensions()
            .iter()
            .map(|dimension| Some(EntityTracker::new(&this, *dimension)))
            .collect();
        self.m_entityTracker.set(trackers);
        let s1: String = self.propertyManager().getStringProperty("level-name", "world");
        let s2: String = self.propertyManager().getStringProperty("level-seed", "");
        let mut k: i64 = Random::new().nextLong();
        if !s2.is_empty()
        {
            match jvm::integer::parseLong(&JString::from(s2.as_str()))
            {
                Ok(seed) => k = seed,
                Err(exception) if exception.instanceOf(&throwable::NUMBER_FORMAT) =>
                {
                    k = JString::from(s2.as_str()).hashCode() as i64
                }
                Err(exception) => throwable::rethrow(exception),
            }
        }
        logger().info(&format!("Preparing level \"{s1}\""));
        let preparing: Instant = Instant::now();
        self.initWorld(&SaveConverterMcRegion::new(Path::new(".")), &s1, k);
        logger().info(&format!("Level \"{s1}\" prepared in {}", took(preparing)));
        logger().info(&format!("Done ({})! For help, type \"help\" or \"?\"", took(startup)));
        Ok(true)
    }

    fn initWorld(&self, storageSource: &dyn ISaveFormat, saveName: &str, seed: i64)
    {
        let this: Rc<MinecraftServer> = self.this();
        if storageSource.isOldMapFormat(saveName)
        {
            logger().info("Converting map!");
            let converting: Instant = Instant::now();
            storageSource.convertMapFormat(
                saveName,
                &ConvertProgress { m_lastTimeMillis: Cell::new(jvm::system::currentTimeMillis()) },
            );
            logger().info(&format!("Converted the map in {}", took(converting)));
        }
        // mod dimensions get worlds after the overworld and the nether
        let dimensions: Vec<i32> = modloader::dimensions::serverDimensions();
        self.m_worldMngr.set(Some(vec![None; dimensions.len()]));
        let saveolddir: Rc<dyn ISaveHandler> = Rc::new(SaveOldDir::new(Path::new("."), saveName, true));
        for (i, dimension) in dimensions.iter().copied().enumerate()
        {
            let world: Rc<World> = if i == 0
            {
                world_server::new(&this, Rc::clone(&saveolddir), saveName, dimension, seed)
            }
            else
            {
                let delegate: Rc<World> = self.m_worldMngr.get().orNpe()[0].clone().orNpe();
                world_server_multi::new(&this, Rc::clone(&saveolddir), saveName, dimension, seed, &delegate)
            };
            self.m_worldMngr.update(|worlds| worlds.as_mut().orNpe()[i] = Some(Rc::clone(&world)));
            logger().debug(&format!("Created world for dimension {dimension}"));
            // mods apply the saved id map and load world data before the start region loads
            modloader::play::worldLoaded(&world);
            world.addWorldAccess(Rc::new(WorldManager::new(&this, &world)));
            world.m_difficultySetting.set(
                if self.propertyManager().getBooleanProperty("spawn-monsters", true)
                {
                    1
                }
                else
                {
                    0
                },
            );
            world.setAllowedMobSpawns(
                self.propertyManager().getBooleanProperty("spawn-monsters", true),
                self.m_spawnPeacefulMobs.get(),
            );
            self.configManager().setPlayerManager(&self.m_worldMngr.get().orNpe());
        }
        let short1: i16 = 196;
        let mut j: i64 = jvm::system::currentTimeMillis();
        for k in 0..2
        {
            logger().info(&format!("Preparing start region for level {k}"));
            if k == 0 || self.propertyManager().getBooleanProperty("allow-nether", true)
            {
                let preparing: Instant = Instant::now();
                let worldserver: Rc<World> = self.worldMngr()[k].clone();
                let chunkcoordinates: ChunkCoordinates = worldserver.getSpawnPoint();
                let short1: i32 = short1 as i32;
                let mut l: i32 = -short1;
                while l <= short1 && self.m_serverRunning.load(Ordering::Relaxed)
                {
                    let mut i1: i32 = -short1;
                    while i1 <= short1 && self.m_serverRunning.load(Ordering::Relaxed)
                    {
                        let j1: i64 = jvm::system::currentTimeMillis();
                        if j1 < j
                        {
                            j = j1;
                        }
                        if j1 > j + 1000
                        {
                            let k1: i32 = (short1 * 2 + 1) * (short1 * 2 + 1);
                            let l1: i32 = (l + short1) * (short1 * 2 + 1) + i1 + 1;
                            self.outputPercentRemaining("Preparing spawn area", l1 * 100 / k1);
                            j = j1;
                        }
                        world_server::chunkProviderServer(&worldserver)
                            .loadChunk(chunkcoordinates.m_x + l >> 4, chunkcoordinates.m_z + i1 >> 4);
                        while worldserver.updatingLighting() && self.m_serverRunning.load(Ordering::Relaxed)
                        {}
                        i1 += 16;
                    }
                    l += 16;
                }
                logger().debug(&format!("Start region for level {k} ready in {}", took(preparing)));
            }
        }
        self.clearCurrentTask();
    }

    fn outputPercentRemaining(&self, progressType: &str, progress: i32)
    {
        self.m_currentTask.set(Some(progressType.to_string()));
        self.m_percentDone.set(progress);
        logger().info(&format!("{progressType}: {progress}%"));
    }

    fn clearCurrentTask(&self)
    {
        self.m_currentTask.set(None);
        self.m_percentDone.set(0);
    }

    fn saveServerWorld(&self)
    {
        logger().info("Saving chunks");
        let saving: Instant = Instant::now();
        for worldserver in self.worldMngr()
        {
            worldserver.saveWorld(true, None);
            world_server::forceSave(&worldserver);
        }
        logger().debug(&format!("Saved chunks in {}", took(saving)));
    }

    fn stopServer(&self)
    {
        logger().info("Stopping server");
        let stopping: Instant = Instant::now();
        modloader::play::lifecycle(modloader::EventKind::ServerStopping);
        if let Some(configManager) = self.m_configManager.get()
        {
            let saving: Instant = Instant::now();
            configManager.savePlayerStates();
            logger().debug(&format!("Saved players in {}", took(saving)));
        }
        for worldserver in self.m_worldMngr.get().orNpe()
        {
            if worldserver.is_some()
            {
                self.saveServerWorld();
            }
        }
        logger().info(&format!("Server stopped in {}", took(stopping)));
    }

    pub fn initiateShutdown(&self) { self.m_serverRunning.store(false, Ordering::Relaxed); }

    fn runLoop(&self) -> Result<(), Throwable>
    {
        if self.startServer()?
        {
            modloader::play::lifecycle(modloader::EventKind::ServerStarted);
            let mut i: i64 = jvm::system::currentTimeMillis();
            let mut j: i64 = 0;
            while self.m_serverRunning.load(Ordering::Relaxed)
            {
                let k: i64 = jvm::system::currentTimeMillis();
                let mut l: i64 = k - i;
                if l > 2000
                {
                    logger().warning(&format!(
                        "Can't keep up! Did the system time change, or is the server overloaded? Running {l}ms behind"
                    ));
                    l = 2000;
                }
                if l < 0
                {
                    logger().warning("Time ran backwards! Did the system time change?");
                    l = 0;
                }
                j += l;
                i = k;
                if self.worldMngr()[0].isAllPlayersFullyAsleep()
                {
                    self.doTick();
                    j = 0;
                }
                else
                {
                    while j > 50
                    {
                        j -= 50;
                        self.doTick();
                    }
                }
                std::thread::sleep(Duration::from_millis(1));
            }
        }
        else
        {
            while self.m_serverRunning.load(Ordering::Relaxed)
            {
                self.commandLineParser();
                std::thread::sleep(Duration::from_millis(10));
            }
        }
        Ok(())
    }

    pub fn run(&self)
    {
        let result: Result<Result<(), Throwable>, Throwable> =
            throwable::catchThrowable(&throwable::THROWABLE, || self.runLoop());
        if let Err(throwable1) = result.and_then(|inner| inner)
        {
            throwable1.printStackTrace();
            logger().log(Level::SEVERE, "Unexpected exception", Some(throwable1));
            while self.m_serverRunning.load(Ordering::Relaxed)
            {
                self.commandLineParser();
                std::thread::sleep(Duration::from_millis(10));
            }
        }
        let stopped: Result<(), Throwable> = throwable::catchThrowable(&throwable::THROWABLE, || {
            self.stopServer();
            self.m_serverStopped.store(true, Ordering::Relaxed);
        });
        if let Err(throwable) = stopped
        {
            throwable.printStackTrace();
        }
        std::process::exit(0);
    }

    fn doTick(&self)
    {
        // mod tick listeners run around the server tick, serverbound mod traffic budgets reset each tick
        modloader::netsync::resetBudgets();
        modloader::play::tick(true);
        let mut arraylist: Vec<JString> = Vec::new();
        GIVE_COMMANDS_COOLDOWNS.with(|cooldowns| {
            let mut cooldowns: std::cell::RefMut<'_, JHashMap<JString, i32>> = cooldowns.borrow_mut();
            for s in cooldowns.keys()
            {
                let i: i32 = cooldowns.get(&s).orNpe();
                if i > 0
                {
                    cooldowns.put(s, i - 1);
                }
                else
                {
                    arraylist.push(s);
                }
            }
            for s in &arraylist
            {
                cooldowns.remove(s);
            }
        });
        AxisAlignedBB::clearBoundingBoxPool();
        Vec3D::initialize();
        self.m_deathTime.set(self.m_deathTime.get() + 1);
        for k in 0..self.worldMngr().len()
        {
            if k != 1 || self.propertyManager().getBooleanProperty("allow-nether", true)
            {
                let worldserver: Rc<World> = self.worldMngr()[k].clone();
                if self.m_deathTime.get() % 20 == 0
                {
                    self.configManager().sendPacketToAllPlayersInDimension(
                        Box::new(Packet4UpdateTime::new(worldserver.getWorldTime())),
                        worldserver.worldProvider().m_worldType.get(),
                    );
                }
                worldserver.tick();
                while worldserver.updatingLighting()
                {}
                worldserver.updateEntities();
            }
        }
        self.networkServer().handleNetworkListenThread();
        self.configManager().onTick();
        for tracker in self.m_entityTracker.get()
        {
            tracker.orNpe().updateTrackedEntities();
        }
        self.networkServer().flush();
        let mut i1: i32 = 0;
        while i1 < self.m_tickables.size()
        {
            self.m_tickables.get(i1).update();
            i1 += 1;
        }
        let result: Result<(), Throwable> =
            throwable::catchThrowable(&throwable::EXCEPTION, || self.commandLineParser());
        if let Err(exception) = result
        {
            logger().log(Level::WARNING, "Unexpected exception while parsing console command", Some(exception));
        }
        // the mod watchdog may stop the server when a mod keeps blowing its tick budget
        if modloader::play::tick(false)
        {
            self.initiateShutdown();
        }
    }

    fn takeConsoleLines(&self)
    {
        let lines: Vec<String> = std::mem::take(&mut *game::port::network::lock(&self.m_consoleLines));
        for line in lines
        {
            let listener: Rc<dyn ICommandListener> = self.this();
            self.m_commands.borrow_mut().push(ServerCommand::new(line, listener));
        }
    }

    pub fn addCommand(&self, command: &str, source: Rc<dyn ICommandListener>)
    {
        // console lines arrive through a channel so they are queued first to keep arrival order
        self.takeConsoleLines();
        self.m_commands.borrow_mut().push(ServerCommand::new(command.to_string(), source));
    }

    pub fn commandLineParser(&self)
    {
        self.takeConsoleLines();
        while !self.m_commands.borrow().is_empty()
        {
            let servercommand: ServerCommand = self.m_commands.borrow_mut().remove(0);
            self.commandHandler().handleCommand(&servercommand);
        }
    }

    pub fn addTickable(&self, tickable: Rc<dyn IUpdatePlayerListBox>) { self.m_tickables.add(tickable); }

    pub fn main(_args: &[String])
    {
        game::stats::StatList::init();
        let result: Result<(), Throwable> = throwable::catchThrowable(&throwable::EXCEPTION, || {
            let minecraftserver: Rc<MinecraftServer> = MinecraftServer::new();
            // the awt gui is not ported so the server always starts as if java were headless
            minecraftserver.run();
        });
        if let Err(exception) = result
        {
            logger().log(Level::SEVERE, "Failed to start the minecraft server", Some(exception));
        }
    }

    pub fn getFile(&self, path: &str) -> PathBuf { PathBuf::from(path) }

    pub fn logWarning(&self, message: &str) { logger().warning(message); }

    // dimensions are keyed through the loader's map, without mods it is overworld then nether
    pub fn getWorldManager(&self, dimension: i32) -> Rc<World>
    {
        self.worldMngr()[modloader::dimensions::serverSlot(dimension)].clone()
    }

    pub fn getEntityTracker(&self, dimension: i32) -> Rc<EntityTracker>
    {
        let trackers: Vec<Option<Rc<EntityTracker>>> = self.m_entityTracker.get();
        trackers[modloader::dimensions::serverSlot(dimension)].clone().orNpe()
    }
}

impl ICommandListener for MinecraftServer
{
    fn log(&self, message: &str) { logger().info(message); }

    fn getUsername(&self) -> String { String::from("CONSOLE") }
}

static UNKNOWN_HOST: ThrowableClass =
    ThrowableClass { m_name: "java.net.UnknownHostException", m_parent: Some(&throwable::IO) };

fn getByName(host: &str) -> Result<IpAddr, Throwable>
{
    match (host, 0).to_socket_addrs().map(|mut addresses| addresses.next())
    {
        Ok(Some(address)) => Ok(address.ip()),
        _ => Err(jvm::io::ioError(&UNKNOWN_HOST, host)),
    }
}

fn readConsole(consoleLines: Arc<Mutex<Vec<String>>>, serverStopped: Arc<AtomicBool>, serverRunning: Arc<AtomicBool>)
{
    let stdin: std::io::Stdin = std::io::stdin();
    let mut line: Vec<u8> = Vec::new();
    while !serverStopped.load(Ordering::Relaxed) && serverRunning.load(Ordering::Relaxed)
    {
        line.clear();
        match std::io::BufRead::read_until(&mut stdin.lock(), b'\n', &mut line)
        {
            Ok(0) => return,
            Ok(_) =>
            {
                let s3: String = game::port::text::decodePlatform(&line).trim_end_matches(['\n', '\r']).to_string();
                game::port::network::lock(&consoleLines).push(s3);
            }
            Err(error) =>
            {
                jvm::io::fromStd(&error).printStackTrace();
                return;
            }
        }
    }
}
