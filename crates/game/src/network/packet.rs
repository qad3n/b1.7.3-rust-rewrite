use crate::network::net_handler::NetHandler;
use crate::network::packets::*;
// java monitors are mutexes that ignore poisoning in port
use crate::port::network::lock;
use jvm::io::{DataInputStream, DataOutputStream, InputStream, IoResult, OutputStream, ioError};
use jvm::string::JString;
use jvm::throwable::{EOF, IO};
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::{LazyLock, Mutex};

pub type PacketInput = DataInputStream<Box<dyn InputStream + Send>>;
pub type PacketOutput = DataOutputStream<Box<dyn OutputStream + Send>>;

#[derive(Clone)]
pub struct PacketBase
{
    pub m_creationTimeMillis: i64,
    pub m_isChunkDataPacket: bool,
}

impl PacketBase
{
    pub fn chunkData() -> PacketBase { PacketBase { m_isChunkDataPacket: true, ..PacketBase::default() } }
}

impl Default for PacketBase
{
    fn default() -> Self
    {
        PacketBase { m_creationTimeMillis: jvm::system::currentTimeMillis(), m_isChunkDataPacket: false }
    }
}

pub trait PacketClone
{
    fn clonePacket(&self) -> Box<dyn Packet>;
}

impl<T: Packet + Clone> PacketClone for T
{
    fn clonePacket(&self) -> Box<dyn Packet> { Box::new(self.clone()) }
}

pub trait Packet: Any + Send + PacketClone
{
    fn base(&self) -> &PacketBase;

    fn readPacketData(&mut self, input: &mut PacketInput) -> IoResult<()>;

    fn writePacketData(&self, output: &mut PacketOutput) -> IoResult<()>;

    fn processPacket(&self, handler: &dyn NetHandler);

    fn getPacketSize(&self) -> i32;

    fn typeName(&self) -> &'static str { std::any::type_name::<Self>() }

    fn getPacketId(&self) -> i32
    {
        let class: TypeId = self.type_id();
        PACKETS.iter().find(|row| (row.m_class)() == class).map(|row| row.m_id).unwrap_or_else(|| jvm::throwable::npe())
    }
}

struct PacketRow
{
    m_id: i32,
    m_clientbound: bool,
    m_serverbound: bool,
    m_class: fn() -> TypeId,
    m_create: fn() -> Box<dyn Packet>,
}

fn newInstance<T: Packet + Default>() -> Box<dyn Packet> { Box::new(T::default()) }

const fn mapping<T: Packet + Default>(id: i32, clientbound: bool, serverbound: bool) -> PacketRow
{
    PacketRow {
        m_id: id,
        m_clientbound: clientbound,
        m_serverbound: serverbound,
        m_class: TypeId::of::<T>,
        m_create: newInstance::<T>,
    }
}

const PACKETS: [PacketRow; 57] = [
    mapping::<Packet0KeepAlive>(0, true, true),
    mapping::<Packet1Login>(1, true, true),
    mapping::<Packet2Handshake>(2, true, true),
    mapping::<Packet3Chat>(3, true, true),
    mapping::<Packet4UpdateTime>(4, true, false),
    mapping::<Packet5PlayerInventory>(5, true, false),
    mapping::<Packet6SpawnPosition>(6, true, false),
    mapping::<Packet7UseEntity>(7, false, true),
    mapping::<Packet8UpdateHealth>(8, true, false),
    mapping::<Packet9Respawn>(9, true, true),
    mapping::<Packet10Flying>(10, true, true),
    mapping::<Packet11PlayerPosition>(11, true, true),
    mapping::<Packet12PlayerLook>(12, true, true),
    mapping::<Packet13PlayerLookMove>(13, true, true),
    mapping::<Packet14BlockDig>(14, false, true),
    mapping::<Packet15Place>(15, false, true),
    mapping::<Packet16BlockItemSwitch>(16, false, true),
    mapping::<Packet17Sleep>(17, true, false),
    mapping::<Packet18Animation>(18, true, true),
    mapping::<Packet19EntityAction>(19, false, true),
    mapping::<Packet20NamedEntitySpawn>(20, true, false),
    mapping::<Packet21PickupSpawn>(21, true, false),
    mapping::<Packet22Collect>(22, true, false),
    mapping::<Packet23VehicleSpawn>(23, true, false),
    mapping::<Packet24MobSpawn>(24, true, false),
    mapping::<Packet25EntityPainting>(25, true, false),
    mapping::<Packet27Position>(27, false, true),
    mapping::<Packet28EntityVelocity>(28, true, false),
    mapping::<Packet29DestroyEntity>(29, true, false),
    mapping::<Packet30Entity>(30, true, false),
    mapping::<Packet31RelEntityMove>(31, true, false),
    mapping::<Packet32EntityLook>(32, true, false),
    mapping::<Packet33RelEntityMoveLook>(33, true, false),
    mapping::<Packet34EntityTeleport>(34, true, false),
    mapping::<Packet38EntityStatus>(38, true, false),
    mapping::<Packet39AttachEntity>(39, true, false),
    mapping::<Packet40EntityMetadata>(40, true, false),
    mapping::<Packet50PreChunk>(50, true, false),
    mapping::<Packet51MapChunk>(51, true, false),
    mapping::<Packet52MultiBlockChange>(52, true, false),
    mapping::<Packet53BlockChange>(53, true, false),
    mapping::<Packet54PlayNoteBlock>(54, true, false),
    mapping::<Packet60Explosion>(60, true, false),
    mapping::<Packet61DoorChange>(61, true, false),
    mapping::<Packet70Bed>(70, true, false),
    mapping::<Packet71Weather>(71, true, false),
    mapping::<Packet100OpenWindow>(100, true, false),
    mapping::<Packet101CloseWindow>(101, true, true),
    mapping::<Packet102WindowClick>(102, false, true),
    mapping::<Packet103SetSlot>(103, true, false),
    mapping::<Packet104WindowItems>(104, true, false),
    mapping::<Packet105UpdateProgressbar>(105, true, false),
    mapping::<Packet106Transaction>(106, true, true),
    mapping::<Packet130UpdateSign>(130, true, true),
    mapping::<Packet131MapData>(131, true, false),
    mapping::<Packet200Statistic>(200, true, false),
    mapping::<Packet255KickDisconnect>(255, true, true),
];

pub struct PacketCounter
{
    m_totalPackets: i32,
    m_totalBytes: i64,
}

impl PacketCounter
{
    fn addPacket(&mut self, size: i32)
    {
        self.m_totalPackets = self.m_totalPackets.wrapping_add(1);
        self.m_totalBytes = self.m_totalBytes.wrapping_add(size as i64);
    }
}

static PACKET_STATS: LazyLock<Mutex<HashMap<i32, PacketCounter>>> = LazyLock::new(|| Mutex::new(HashMap::new()));
static TOTAL_PACKETS_COUNT: AtomicI32 = AtomicI32::new(0);

pub fn getNewPacket(id: i32) -> Option<Box<dyn Packet>>
{
    PACKETS.iter().find(|row| row.m_id == id).map(|row| (row.m_create)())
}

pub fn read(input: &mut PacketInput) -> IoResult<i32> { input.m_in.read() }

pub fn readPacket(input: &mut PacketInput, server: bool) -> IoResult<Option<Box<dyn Packet>>>
{
    let mut i: i32 = 0;
    let mut body = |i: &mut i32| -> IoResult<Option<Box<dyn Packet>>> {
        *i = read(input)?;
        if *i == -1
        {
            return Ok(None);
        }
        let row: Option<&PacketRow> = PACKETS.iter().find(|row| row.m_id == *i);
        if server && !row.is_some_and(|row| row.m_serverbound) || !server && !row.is_some_and(|row| row.m_clientbound)
        {
            return Err(ioError(&IO, &format!("Bad packet id {i}")));
        }
        let Some(mut packet) = getNewPacket(*i)
        else
        {
            return Err(ioError(&IO, &format!("Bad packet id {i}")));
        };
        packet.readPacketData(input)?;
        Ok(Some(packet))
    };
    let packet: Box<dyn Packet> = match body(&mut i)
    {
        Ok(Some(packet)) => packet,
        Ok(None) => return Ok(None),
        Err(throwable) if throwable.instanceOf(&EOF) =>
        {
            jvm::system::println("Reached end of stream");
            return Ok(None);
        }
        Err(throwable) => return Err(throwable),
    };
    lock(&PACKET_STATS)
        .entry(i)
        .or_insert(PacketCounter { m_totalPackets: 0, m_totalBytes: 0 })
        .addPacket(packet.getPacketSize());
    TOTAL_PACKETS_COUNT.fetch_add(1, Ordering::Relaxed);
    Ok(Some(packet))
}

pub fn writePacket(packet: &dyn Packet, output: &mut PacketOutput) -> IoResult<()>
{
    output.write(packet.getPacketId())?;
    packet.writePacketData(output)
}

pub fn writeString<S: OutputStream>(s: &JString, output: &mut DataOutputStream<S>) -> IoResult<()>
{
    if s.length() > i16::MAX as i32
    {
        return Err(ioError(&IO, "String too big"));
    }
    output.writeShort(s.length())?;
    output.writeChars(s)
}

pub fn readString<S: InputStream>(input: &mut DataInputStream<S>, maxLength: i32) -> IoResult<JString>
{
    let n: i32 = input.readShort()? as i32;
    if n > maxLength
    {
        return Err(ioError(&IO, &format!("Received string length longer than maximum allowed ({n} > {maxLength})")));
    }
    if n < 0
    {
        return Err(ioError(&IO, "Received string length is less than zero! Weird string!"));
    }
    let mut chars: Vec<u16> = Vec::with_capacity(n as usize);
    for _ in 0..n
    {
        chars.push(input.readChar()?);
    }
    Ok(JString::fromUtf16(chars))
}

pub fn className(packet: &dyn Packet) -> String
{
    let name: &str = packet.typeName().rsplit("::").next().unwrap_or_default();
    match name
    {
        "Packet11PlayerPosition" | "Packet12PlayerLook" | "Packet13PlayerLookMove" => format!("Packet10Flying${name}"),
        "Packet31RelEntityMove" | "Packet32EntityLook" | "Packet33RelEntityMoveLook" =>
        {
            format!("Packet30Entity${name}")
        }
        _ => name.to_string(),
    }
}
