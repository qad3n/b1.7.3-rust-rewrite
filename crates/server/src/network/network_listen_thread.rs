use crate::network::{NetLoginHandler, NetServerHandler};
use crate::server::MinecraftServer;
use jvm::array_list::JArrayList;
use jvm::io::IoResult;
use jvm::logging::{Level, Logger};
use jvm::string::JString;
use jvm::throwable::{self, Throwable};
use std::collections::HashMap;
use std::net::{IpAddr, Ipv6Addr, SocketAddr, TcpListener, TcpStream};
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread::JoinHandle;

type Accepted = (TcpStream, i32);

fn logger() -> &'static Logger { Logger::getLogger("Minecraft") }

pub struct NetworkListenThread
{
    pub m_serverSocket: TcpListener,
    pub m_networkAcceptThread: JoinHandle<()>,
    pub m_open: Arc<AtomicBool>,
    m_accepted: Receiver<Accepted>,
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

fn acceptLoop(serverSocket: TcpListener, open: Arc<AtomicBool>, accepted: Sender<Accepted>)
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
                let last: Option<i64> = hashmap.get(&inetaddress).copied();
                if let Some(last) = last
                    && inetaddress.to_string() != "127.0.0.1"
                    && jvm::system::currentTimeMillis() - last < 5000
                {
                    hashmap.insert(inetaddress, jvm::system::currentTimeMillis());
                    drop(socket);
                }
                else
                {
                    hashmap.insert(inetaddress, jvm::system::currentTimeMillis());
                    let counter: i32 = connectionCounter;
                    connectionCounter += 1;
                    if accepted.send((socket, counter)).is_err()
                    {
                        return;
                    }
                }
            }
            Err(error) => jvm::io::fromStd(&error).printStackTrace(),
        }
    }
}

impl NetworkListenThread
{
    pub fn new(server: &Rc<MinecraftServer>, address: Option<IpAddr>, port: i32) -> IoResult<NetworkListenThread>
    {
        let serverSocket: TcpListener = bind(address, port)?;
        let open: Arc<AtomicBool> = Arc::new(AtomicBool::new(true));
        let (sender, receiver): (Sender<Accepted>, Receiver<Accepted>) = channel();
        let listener: TcpListener = serverSocket.try_clone().map_err(|error| jvm::io::fromStd(&error))?;
        let threadOpen: Arc<AtomicBool> = Arc::clone(&open);
        let networkAcceptThread: JoinHandle<()> =
            jvm::thread::start(JString::from("Listen thread"), move || acceptLoop(listener, threadOpen, sender))
                .map_err(|error| jvm::io::fromStd(&error))?;
        Ok(NetworkListenThread {
            m_serverSocket: serverSocket,
            m_networkAcceptThread: networkAcceptThread,
            m_open: open,
            m_accepted: receiver,
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
                Err(ioexception) => ioexception.printStackTrace(),
            }
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
                logger().log(Level::WARNING, &format!("Failed to handle packet: {exception1}"), Some(exception1));
            }
            if netloginhandler.m_finishedProcessing.get()
            {
                self.m_pendingConnections.removeAt(i);
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
                logger().log(Level::WARNING, &format!("Failed to handle packet: {exception}"), Some(exception));
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
