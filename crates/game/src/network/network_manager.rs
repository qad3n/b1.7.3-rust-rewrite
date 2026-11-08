use crate::network::net_handler::NetHandler;
use crate::network::packet::{self, Packet, PacketInput, PacketOutput};
// java monitors and null stream errors come from port
use crate::port::network::{lock, npeError};
use jvm::io::{
    BufferedOutputStream, DataInputStream, DataOutputStream, InputStream, IoResult, OutputStream, fromStd, ioError,
};
use jvm::string::JString;
use jvm::throwable::{EXCEPTION, IO, Throwable, ThrowableClass, catchThrowable};
use std::cell::{Cell, RefCell};
use std::io::{ErrorKind, Read, Write};
use std::net::{Shutdown, TcpStream};
use std::rc::{Rc, Weak};
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread::JoinHandle;
use std::time::Duration;

static SOCKET: ThrowableClass = ThrowableClass { m_name: "java.net.SocketException", m_parent: Some(&IO) };
static SOCKET_TIMEOUT: ThrowableClass =
    ThrowableClass { m_name: "java.net.SocketTimeoutException", m_parent: Some(&IO) };

pub static NUM_READ_THREADS: AtomicI32 = AtomicI32::new(0);
pub static NUM_WRITE_THREADS: AtomicI32 = AtomicI32::new(0);
pub static TOTAL_READ_SIZE: [AtomicI32; 256] = [const { AtomicI32::new(0) }; 256];
pub static TOTAL_SEND_SIZE: [AtomicI32; 256] = [const { AtomicI32::new(0) }; 256];

fn socketError(error: &std::io::Error) -> Throwable
{
    match error.kind()
    {
        ErrorKind::WouldBlock | ErrorKind::TimedOut => ioError(&SOCKET_TIMEOUT, "Read timed out"),
        ErrorKind::ConnectionReset => ioError(&SOCKET, "Connection reset"),
        ErrorKind::BrokenPipe => ioError(&SOCKET, "Broken pipe"),
        _ => fromStd(error),
    }
}

struct SocketInputStream
{
    m_socket: TcpStream,
}

impl InputStream for SocketInputStream
{
    fn read(&mut self) -> IoResult<i32>
    {
        let mut buf: [u8; 1] = [0];
        match self.readInto(&mut buf, 0, 1)?
        {
            -1 => Ok(-1),
            _ => Ok(buf[0] as i32),
        }
    }

    fn readInto(&mut self, buf: &mut [u8], off: i32, len: i32) -> IoResult<i32>
    {
        if len == 0
        {
            return Ok(0);
        }
        loop
        {
            match self.m_socket.read(&mut buf[off as usize..(off + len) as usize])
            {
                Ok(0) => return Ok(-1),
                Ok(count) => return Ok(count as i32),
                Err(error) if error.kind() == ErrorKind::Interrupted => continue,
                Err(error) => return Err(socketError(&error)),
            }
        }
    }
}

struct SocketOutputStream
{
    m_socket: TcpStream,
}

impl OutputStream for SocketOutputStream
{
    fn write(&mut self, value: i32) -> IoResult<()> { self.writeFrom(&[value as u8], 0, 1) }

    fn writeFrom(&mut self, buf: &[u8], off: i32, len: i32) -> IoResult<()>
    {
        self.m_socket.write_all(&buf[off as usize..(off + len) as usize]).map_err(|error| socketError(&error))
    }

    fn flush(&mut self) -> IoResult<()> { self.m_socket.flush().map_err(|error| socketError(&error)) }

    fn close(&mut self) -> IoResult<()>
    {
        match self.m_socket.shutdown(Shutdown::Both)
        {
            Err(error) if error.kind() != ErrorKind::NotConnected => Err(socketError(&error)),
            _ => Ok(()),
        }
    }
}

struct ThreadCount(&'static AtomicI32);

impl ThreadCount
{
    fn enter(counter: &'static AtomicI32) -> ThreadCount
    {
        counter.fetch_add(1, Ordering::Relaxed);
        ThreadCount(counter)
    }
}

impl Drop for ThreadCount
{
    fn drop(&mut self) { self.0.fetch_sub(1, Ordering::Relaxed); }
}

struct Shared
{
    m_sendQueueLock: Mutex<()>,
    m_networkSocket: Mutex<Option<TcpStream>>,
    m_socketInputStream: Mutex<Option<PacketInput>>,
    m_socketOutputStream: Mutex<Option<PacketOutput>>,
    m_isRunning: AtomicBool,
    m_readPackets: Mutex<Vec<Box<dyn Packet>>>,
    m_dataPackets: Mutex<Vec<Box<dyn Packet>>>,
    m_chunkDataPackets: Mutex<Vec<Box<dyn Packet>>>,
    m_isServerHandler: bool,
    m_isServerTerminating: AtomicBool,
    m_writeThread: OnceLock<JoinHandle<()>>,
    m_readThread: OnceLock<JoinHandle<()>>,
    m_isTerminating: AtomicBool,
    m_terminationReason: Mutex<JString>,
    m_disconnectReasonArgs: Mutex<Vec<JString>>,
    m_sendQueueByteLength: AtomicI32,
    m_chunkDataSendCounter: AtomicI32,
    m_delay: AtomicI32,
}

impl Shared
{
    fn readLoop(&self)
    {
        let _count: ThreadCount = ThreadCount::enter(&NUM_READ_THREADS);
        while self.m_isRunning.load(Ordering::Relaxed) && !self.m_isServerTerminating.load(Ordering::Relaxed)
        {
            while self.readPacket()
            {}
            std::thread::park_timeout(Duration::from_millis(100));
        }
    }

    fn writeLoop(&self)
    {
        let _count: ThreadCount = ThreadCount::enter(&NUM_WRITE_THREADS);
        while self.m_isRunning.load(Ordering::Relaxed)
        {
            while self.sendPacket()
            {}
            std::thread::park_timeout(Duration::from_millis(100));
            let flushed: IoResult<()> = match lock(&self.m_socketOutputStream).as_mut()
            {
                None => continue,
                Some(output) => output.flush(),
            };
            if let Err(ex2) = flushed
            {
                if !self.m_isTerminating.load(Ordering::Relaxed)
                {
                    self.onNetworkError(&ex2);
                }
                ex2.printStackTrace();
            }
        }
    }

    fn isDue(&self, queue: &Mutex<Vec<Box<dyn Packet>>>) -> bool
    {
        let counter: i32 = self.m_chunkDataSendCounter.load(Ordering::Relaxed);
        !lock(queue).is_empty()
            && (counter == 0
                || jvm::system::currentTimeMillis() - lock(queue)[0].base().m_creationTimeMillis >= counter as i64)
    }

    fn dequeue(&self, queue: &Mutex<Vec<Box<dyn Packet>>>) -> Box<dyn Packet>
    {
        let _guard: std::sync::MutexGuard<'_, ()> = lock(&self.m_sendQueueLock);
        let packet: Box<dyn Packet> = lock(queue).remove(0);
        self.m_sendQueueByteLength.fetch_sub(packet.getPacketSize() + 1, Ordering::Relaxed);
        packet
    }

    fn writeQueued(&self, packet: &dyn Packet) -> IoResult<()>
    {
        {
            let mut guard: std::sync::MutexGuard<'_, Option<PacketOutput>> = lock(&self.m_socketOutputStream);
            let output: &mut PacketOutput = guard.as_mut().ok_or_else(npeError)?;
            packet::writePacket(packet, output)?;
        }
        TOTAL_SEND_SIZE[packet.getPacketId() as usize].fetch_add(packet.getPacketSize() + 1, Ordering::Relaxed);
        Ok(())
    }

    fn sendQueuedPackets(&self) -> IoResult<bool>
    {
        let mut b: bool = false;
        if self.isDue(&self.m_dataPackets)
        {
            let packet: Box<dyn Packet> = self.dequeue(&self.m_dataPackets);
            self.writeQueued(packet.as_ref())?;
            b = true;
        }
        if self.m_delay.fetch_sub(1, Ordering::Relaxed) <= 0 && self.isDue(&self.m_chunkDataPackets)
        {
            let packet: Box<dyn Packet> = self.dequeue(&self.m_chunkDataPackets);
            self.writeQueued(packet.as_ref())?;
            self.m_delay.store(0, Ordering::Relaxed);
            b = true;
        }
        Ok(b)
    }

    fn sendPacket(&self) -> bool
    {
        match catchThrowable(&EXCEPTION, || self.sendQueuedPackets()).and_then(|result| result)
        {
            Ok(b) => b,
            Err(exception) =>
            {
                if !self.m_isTerminating.load(Ordering::Relaxed)
                {
                    self.onNetworkError(&exception);
                }
                false
            }
        }
    }

    fn readOnePacket(&self) -> IoResult<bool>
    {
        let packet: Option<Box<dyn Packet>> = {
            let mut guard: std::sync::MutexGuard<'_, Option<PacketInput>> = lock(&self.m_socketInputStream);
            let input: &mut PacketInput = guard.as_mut().ok_or_else(npeError)?;
            packet::readPacket(input, self.m_isServerHandler)?
        };
        match packet
        {
            Some(packet) =>
            {
                TOTAL_READ_SIZE[packet.getPacketId() as usize].fetch_add(packet.getPacketSize() + 1, Ordering::Relaxed);
                lock(&self.m_readPackets).push(packet);
                Ok(true)
            }
            None =>
            {
                self.networkShutdown("disconnect.endOfStream", Vec::new());
                Ok(false)
            }
        }
    }

    fn readPacket(&self) -> bool
    {
        match catchThrowable(&EXCEPTION, || self.readOnePacket()).and_then(|result| result)
        {
            Ok(b) => b,
            Err(exception) =>
            {
                if !self.m_isTerminating.load(Ordering::Relaxed)
                {
                    self.onNetworkError(&exception);
                }
                false
            }
        }
    }

    fn onNetworkError(&self, exception: &Throwable)
    {
        exception.printStackTrace();
        self.networkShutdown(
            "disconnect.genericReason",
            vec![JString::from(format!("Internal exception: {exception}").as_str())],
        );
    }

    fn wakeThreads(&self)
    {
        if let Some(thread) = self.m_readThread.get()
        {
            thread.thread().unpark();
        }
        if let Some(thread) = self.m_writeThread.get()
        {
            thread.thread().unpark();
        }
    }

    fn networkShutdown(&self, reason: &str, args: Vec<JString>)
    {
        if !self.m_isRunning.load(Ordering::Relaxed)
        {
            return;
        }
        self.m_isTerminating.store(true, Ordering::Relaxed);
        *lock(&self.m_terminationReason) = JString::from(reason);
        *lock(&self.m_disconnectReasonArgs) = args;
        // rust has no Thread.stop so the killer thread only consumes its name
        jvm::thread::nextName();
        self.m_isRunning.store(false, Ordering::Relaxed);
        if let Some(socket) = lock(&self.m_networkSocket).as_ref()
            && socket.shutdown(Shutdown::Both).is_ok()
        {
            lock(&self.m_socketInputStream).take();
        }
        let mut output: std::sync::MutexGuard<'_, Option<PacketOutput>> = lock(&self.m_socketOutputStream);
        if output.as_mut().is_some_and(|stream| stream.close().is_ok())
        {
            output.take();
        }
        drop(output);
        lock(&self.m_networkSocket).take();
    }
}

pub struct NetworkManager
{
    m_shared: Arc<Shared>,
    m_remoteSocketAddress: Option<std::net::SocketAddr>,
    m_netHandler: RefCell<Weak<dyn NetHandler>>,
    m_timeSinceLastRead: Cell<i32>,
}

impl NetworkManager
{
    pub fn new(
        socket: TcpStream,
        name: &str,
        listener: Weak<dyn NetHandler>,
        isServerHandler: bool,
    ) -> IoResult<Rc<NetworkManager>>
    {
        if let Err(ex) = socket.set_read_timeout(Some(Duration::from_millis(30000)))
        {
            jvm::system::errPrintln(&ex.to_string());
        }
        let remoteSocketAddress: Option<std::net::SocketAddr> = socket.peer_addr().ok();
        let input: Box<dyn InputStream + Send> =
            Box::new(SocketInputStream { m_socket: socket.try_clone().map_err(|error| fromStd(&error))? });
        let output: Box<dyn OutputStream + Send> = Box::new(BufferedOutputStream::new(
            SocketOutputStream { m_socket: socket.try_clone().map_err(|error| fromStd(&error))? },
            5120,
        ));
        let shared: Arc<Shared> = Arc::new(Shared {
            m_sendQueueLock: Mutex::new(()),
            m_networkSocket: Mutex::new(Some(socket)),
            m_socketInputStream: Mutex::new(Some(DataInputStream::new(input))),
            m_socketOutputStream: Mutex::new(Some(DataOutputStream::new(output))),
            m_isRunning: AtomicBool::new(true),
            m_readPackets: Mutex::new(Vec::new()),
            m_dataPackets: Mutex::new(Vec::new()),
            m_chunkDataPackets: Mutex::new(Vec::new()),
            m_isServerHandler: isServerHandler,
            m_isServerTerminating: AtomicBool::new(false),
            m_writeThread: OnceLock::new(),
            m_readThread: OnceLock::new(),
            m_isTerminating: AtomicBool::new(false),
            m_terminationReason: Mutex::new(JString::from("")),
            m_disconnectReasonArgs: Mutex::new(Vec::new()),
            m_sendQueueByteLength: AtomicI32::new(0),
            m_chunkDataSendCounter: AtomicI32::new(0),
            m_delay: AtomicI32::new(50),
        });
        let reader: Arc<Shared> = Arc::clone(&shared);
        let readThread: JoinHandle<()> =
            jvm::thread::start(JString::from(format!("{name} read thread").as_str()), move || reader.readLoop())
                .map_err(|error| fromStd(&error))?;
        let writer: Arc<Shared> = Arc::clone(&shared);
        let writeThread: JoinHandle<()> =
            jvm::thread::start(JString::from(format!("{name} write thread").as_str()), move || writer.writeLoop())
                .map_err(|error| fromStd(&error))?;
        if shared.m_readThread.set(readThread).is_err() || shared.m_writeThread.set(writeThread).is_err()
        {
            unreachable!("the threads are stored once");
        }
        Ok(Rc::new(NetworkManager {
            m_shared: shared,
            m_remoteSocketAddress: remoteSocketAddress,
            m_netHandler: RefCell::new(listener),
            m_timeSinceLastRead: Cell::new(0),
        }))
    }

    fn netHandler(&self) -> Rc<dyn NetHandler>
    {
        self.m_netHandler.borrow().upgrade().expect("the net handler outlives its network manager")
    }

    pub fn setChunkDataSendCounter(&self, value: i32)
    {
        self.m_shared.m_chunkDataSendCounter.store(value, Ordering::Relaxed);
    }

    pub fn setNetHandler(&self, handler: Weak<dyn NetHandler>) { *self.m_netHandler.borrow_mut() = handler; }

    pub fn addToSendQueue(&self, packet: Box<dyn Packet>)
    {
        let shared: &Shared = &self.m_shared;
        if shared.m_isServerTerminating.load(Ordering::Relaxed)
        {
            return;
        }
        let _guard: std::sync::MutexGuard<'_, ()> = lock(&shared.m_sendQueueLock);
        shared.m_sendQueueByteLength.fetch_add(packet.getPacketSize() + 1, Ordering::Relaxed);
        if packet.base().m_isChunkDataPacket
        {
            lock(&shared.m_chunkDataPackets).push(packet);
        }
        else
        {
            lock(&shared.m_dataPackets).push(packet);
        }
    }

    pub fn wakeThreads(&self) { self.m_shared.wakeThreads(); }

    pub fn networkShutdown(&self, reason: &str, args: Vec<JString>) { self.m_shared.networkShutdown(reason, args); }

    pub fn processReadPackets(&self)
    {
        let shared: &Shared = &self.m_shared;
        if shared.m_sendQueueByteLength.load(Ordering::Relaxed) > 0x100000
        {
            self.networkShutdown("disconnect.overflow", Vec::new());
        }
        if lock(&shared.m_readPackets).is_empty()
        {
            let timeSinceLastRead: i32 = self.m_timeSinceLastRead.get();
            self.m_timeSinceLastRead.set(timeSinceLastRead + 1);
            if timeSinceLastRead == 1200
            {
                self.networkShutdown("disconnect.timeout", Vec::new());
            }
        }
        else
        {
            self.m_timeSinceLastRead.set(0);
        }
        let mut n: i32 = 100;
        loop
        {
            if lock(&shared.m_readPackets).is_empty()
            {
                break;
            }
            let current: i32 = n;
            n -= 1;
            if current < 0
            {
                break;
            }
            let packet: Box<dyn Packet> = lock(&shared.m_readPackets).remove(0);
            packet.processPacket(self.netHandler().as_ref());
        }
        self.wakeThreads();
        if shared.m_isTerminating.load(Ordering::Relaxed) && lock(&shared.m_readPackets).is_empty()
        {
            let reason: JString = lock(&shared.m_terminationReason).clone();
            let args: Vec<JString> = lock(&shared.m_disconnectReasonArgs).clone();
            self.netHandler().handleErrorMessage(&reason, &args);
        }
    }

    pub fn getRemoteAddress(&self) -> String
    {
        match &self.m_remoteSocketAddress
        {
            Some(address) => jvm::net::socketAddressToString(address),
            None => String::from("null"),
        }
    }

    pub fn close(&self)
    {
        self.wakeThreads();
        let shared: &Arc<Shared> = &self.m_shared;
        shared.m_isServerTerminating.store(true, Ordering::Relaxed);
        if let Some(thread) = shared.m_readThread.get()
        {
            thread.thread().unpark();
        }
        let closer: Arc<Shared> = Arc::clone(shared);
        let started: std::io::Result<JoinHandle<()>> = jvm::thread::start(jvm::thread::nextName(), move || {
            std::thread::sleep(Duration::from_millis(2000));
            if closer.m_isRunning.load(Ordering::Relaxed)
            {
                if let Some(thread) = closer.m_writeThread.get()
                {
                    thread.thread().unpark();
                }
                closer.networkShutdown("disconnect.closed", Vec::new());
            }
        });
        if let Err(error) = started
        {
            fromStd(&error).printStackTrace();
        }
    }

    pub fn getNumChunkDataPackets(&self) -> i32 { lock(&self.m_shared.m_chunkDataPackets).len() as i32 }
}
