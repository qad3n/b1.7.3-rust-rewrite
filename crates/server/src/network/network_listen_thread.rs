use crate::network::{NetLoginHandler, NetServerHandler};
use crate::server::MinecraftServer;
use jvm::array_list::JArrayList;
use jvm::io::IoResult;
use jvm::logging::Level;
use jvm::string::JString;
use jvm::throwable::{self, Throwable};
use jvm::{debug, info, logThrown, warning};
use std::collections::HashMap;
use std::net::{IpAddr, Ipv6Addr, SocketAddr, TcpListener, TcpStream};
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread::JoinHandle;

type Accepted = (TcpStream, i32);

mod limits
{
    use std::time::Duration;

    pub const RECONNECT_MILLIS: i64 = 5000;
    // rewrite: caps on top of the vanilla reconnect throttle so login floods cannot exhaust threads or memory
    pub const PENDING_MAX: usize = 64;
    pub const TRACKED_MAX: usize = 4096;
    pub const ACCEPT_RETRY: Duration = Duration::from_millis(100);
    pub const IPV6_PREFIX_BITS: u32 = 64;
}

pub struct NetworkListenThread
{
    pub m_serverSocket: TcpListener,
    pub m_networkAcceptThread: JoinHandle<()>,
    pub m_open: Arc<AtomicBool>,
    m_accepted: Receiver<Accepted>,
    m_pending: Arc<AtomicUsize>,
    m_pendingConnections: JArrayList<Rc<NetLoginHandler>>,
    m_playerList: JArrayList<Rc<NetServerHandler>>,
    pub m_mcServer: Rc<MinecraftServer>,
}

fn bind(address: Option<IpAddr>, port: i32) -> IoResult<TcpListener>
{
    let port: u16 = port as u16;

    let result: std::io::Result<TcpListener> = match address
    {
        Some(address) => TcpListener::bind(SocketAddr::new(address, port)),
        None => TcpListener::bind(SocketAddr::new(IpAddr::V6(Ipv6Addr::UNSPECIFIED), port))
            .or_else(|_| TcpListener::bind(SocketAddr::new(IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED), port))),
    };
    result.map_err(|error| jvm::io::fromStd(&error))
}

// one address per ipv6 /64 since a single host usually owns the whole prefix
fn throttleKey(ip: IpAddr) -> IpAddr
{
    match ip
    {
        IpAddr::V4(_) => ip,
        IpAddr::V6(ip) =>
        {
            let mask: u128 = u128::MAX << (u128::BITS - limits::IPV6_PREFIX_BITS);
            IpAddr::V6(Ipv6Addr::from(ip.to_bits() & mask))
        }
    }
}

fn acceptLoop(serverSocket: TcpListener, open: Arc<AtomicBool>, accepted: Sender<Accepted>, pending: Arc<AtomicUsize>)
{
    let mut hashmap: HashMap<IpAddr, i64> = HashMap::new();
    let mut connectionCounter: i32 = 0;

    while open.load(Ordering::Relaxed)
    {
        match serverSocket.accept()
        {
            Ok((socket, address)) =>
            {
                let inetaddress: IpAddr = address.ip().to_canonical();
                let shown: String = auth::tag(inetaddress);
                let key: IpAddr = throttleKey(inetaddress);
                let now: i64 = jvm::system::currentTimeMillis();

                if hashmap.len() > limits::TRACKED_MAX
                {
                    hashmap.retain(|_, last| now - *last < limits::RECONNECT_MILLIS);
                }

                let last: Option<i64> = hashmap.get(&key).copied();
                if let Some(last) = last
                    && inetaddress.to_string() != "127.0.0.1"
                    && now - last < limits::RECONNECT_MILLIS
                {
                    info!("Refused {shown}, it reconnected within 5 seconds");
                    hashmap.insert(key, now);
                    drop(socket);
                }
                else if pending.load(Ordering::Relaxed) >= limits::PENDING_MAX
                {
                    info!("Refused {shown}, too many connections are logging in");
                    drop(socket);
                }
                else
                {
                    hashmap.insert(key, now);
                    pending.fetch_add(1, Ordering::Relaxed);

                    let counter: i32 = connectionCounter;

                    connectionCounter += 1;
                    info!("Connection #{counter} from {shown}");

                    if accepted.send((socket, counter)).is_err()
                    {
                        return;
                    }
                }
            }

            Err(error) =>
            {
                warning!("Could not accept a connection: {error}");
                // errors such as running out of file handles repeat at once so wait before trying again
                std::thread::sleep(limits::ACCEPT_RETRY);
            }
        }
    }
}

impl NetworkListenThread
{
    pub fn new(server: &Rc<MinecraftServer>, address: Option<IpAddr>, port: i32) -> IoResult<Self>
    {
        let serverSocket: TcpListener = bind(address, port)?;
        if let Ok(local) = serverSocket.local_addr()
        {
            debug!("Listening on {local}");
        }

        let open: Arc<AtomicBool> = Arc::new(AtomicBool::new(true));
        let (sender, receiver): (Sender<Accepted>, Receiver<Accepted>) = channel();
        let listener: TcpListener = serverSocket.try_clone().map_err(|error| jvm::io::fromStd(&error))?;
        let threadOpen: Arc<AtomicBool> = Arc::clone(&open);
        let pending: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));
        let threadPending: Arc<AtomicUsize> = Arc::clone(&pending);
        let networkAcceptThread: JoinHandle<()> = jvm::thread::start(JString::from("Listen thread"), move ||
        {
            acceptLoop(listener, threadOpen, sender, threadPending)
        })
        .map_err(|error| jvm::io::fromStd(&error))?;

        Ok(Self
        {
            m_serverSocket: serverSocket,
            m_networkAcceptThread: networkAcceptThread,
            m_open: open,
            m_accepted: receiver,
            m_pending: pending,
            m_pendingConnections: JArrayList::new(),
            m_playerList: JArrayList::new(),
            m_mcServer: Rc::clone(server),
        })
    }

    pub fn addPlayer(&self, connection: &Rc<NetServerHandler>) { self.m_playerList.add(Rc::clone(connection)); }

    fn addPendingConnection(&self, connection: Rc<NetLoginHandler>) { self.m_pendingConnections.add(connection); }

    fn takeAccepted(&self)
    {
        // the listen thread only accepts sockets and the login handlers holding rc state are built here on the server thread
        while let Ok((socket, counter)) = self.m_accepted.try_recv()
        {
            match NetLoginHandler::new(&self.m_mcServer, socket, &format!("Connection #{counter}"))
            {
                Ok(netloginhandler) => self.addPendingConnection(netloginhandler),
                Err(ioexception) =>
                {
                    self.m_pending.fetch_sub(1, Ordering::Relaxed);
                    warning!("Could not set up connection #{counter}: {ioexception}");
                }
            }
        }
    }

    // tracker packets queue after handleNetworkListenThread so they would otherwise wait a whole tick
    pub fn flush(&self)
    {
        for i in 0..self.m_playerList.size()
        {
            self.m_playerList.get(i).m_netManager.wakeThreads();
        }
    }

    pub fn handleNetworkListenThread(&self)
    {
        self.takeAccepted();

        let mut i: i32 = 0;
        while i < self.m_pendingConnections.size()
        {
            let netloginhandler: Rc<NetLoginHandler> = self.m_pendingConnections.get(i);
            let result: Result<(), Throwable> =
                throwable::catchThrowable(&throwable::EXCEPTION, || netloginhandler.tryLogin());

            if let Err(exception1) = result
            {
                netloginhandler.kickUser("Internal server error");
                logThrown!(Level::WARNING, Some(exception1), "Failed to handle packet: {exception1}");
            }

            if netloginhandler.m_finishedProcessing.get()
            {
                self.m_pendingConnections.removeAt(i);
                self.m_pending.fetch_sub(1, Ordering::Relaxed);
                i -= 1;
            }

            netloginhandler.netManager().wakeThreads();
            i += 1;
        }

        let mut j: i32 = 0;
        while j < self.m_playerList.size()
        {
            let netserverhandler: Rc<NetServerHandler> = self.m_playerList.get(j);
            let result: Result<(), Throwable> =
                throwable::catchThrowable(&throwable::EXCEPTION, || netserverhandler.handlePackets());

            if let Err(exception) = result
            {
                logThrown!(Level::WARNING, Some(exception), "Failed to handle packet: {exception}");
                netserverhandler.kickPlayer("Internal server error");
            }

            if netserverhandler.m_connectionClosed.get()
            {
                self.m_playerList.removeAt(j);
                j -= 1;
            }

            netserverhandler.m_netManager.wakeThreads();
            j += 1;
        }
    }
}
