use game::entity::Entity;
use mod_protocol::Identifier;
use mod_protocol::handshake::ChannelDirection;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub type Handler = Rc<dyn Fn(&Rc<Entity>, &[u8])>;
pub type WireSender = Box<dyn Fn(&Identifier, Vec<u8>)>;

pub struct Channel
{
    pub m_id: Identifier,
    pub m_version: String,
    pub m_direction: ChannelDirection,
    pub m_onServer: Option<Handler>,
    pub m_onClient: Option<Handler>,
}

#[derive(Default)]
struct Router
{
    m_channels: HashMap<Identifier, Channel>,
    // set by the client while it holds a multiplayer connection, absent means single player loopback
    m_toServer: Option<WireSender>,
    m_localPlayer: Option<Rc<Entity>>,
    // dedicated server connections by player entity id
    m_toPlayers: HashMap<i32, WireSender>,
}

thread_local!
{
    static ROUTER: RefCell<Router> = RefCell::new(Router::default());
}

pub fn registerChannel(channel: Channel)
{
    ROUTER.with(|router| router.borrow_mut().m_channels.insert(channel.m_id.clone(), channel));
}

pub fn channels() -> Vec<(Identifier, String, ChannelDirection)>
{
    ROUTER.with(|router|
    {
        let mut list: Vec<(Identifier, String, ChannelDirection)> = router
            .borrow()
            .m_channels
            .values()
            .map(|channel| (channel.m_id.clone(), channel.m_version.clone(), channel.m_direction))
            .collect();
        list.sort_by(|left, right| left.0.cmp(&right.0));
        list
    })
}

pub fn setServerConnection(sender: Option<WireSender>)
{
    ROUTER.with(|router| router.borrow_mut().m_toServer = sender);
}

pub fn setLocalPlayer(player: Option<Rc<Entity>>) { ROUTER.with(|router| router.borrow_mut().m_localPlayer = player); }

fn handler(channel: &Identifier, onServer: bool) -> Option<Handler>
{
    ROUTER.with(|router|
    {
        let router: std::cell::Ref<Router> = router.borrow();
        let channel: &Channel = router.m_channels.get(channel)?;

        if onServer
        {
            channel.m_onServer.clone()
        }
        else
        {
            channel.m_onClient.clone()
        }
    })
}

// handlers run outside the router borrow so they may send again
pub fn deliverToServer(channel: &Identifier, sender: &Rc<Entity>, data: &[u8]) -> bool
{
    match handler(channel, true)
    {
        Some(handler) =>
        {
            handler(sender, data);
            true
        }

        None => false,
    }
}

pub fn deliverToClient(channel: &Identifier, player: &Rc<Entity>, data: &[u8]) -> bool
{
    match handler(channel, false)
    {
        Some(handler) =>
        {
            handler(player, data);
            true
        }

        None => false,
    }
}

// single player has no server connection, so the server side handler runs in place this tick
pub fn sendToServer(channel: &Identifier, data: Vec<u8>) -> bool
{
    let sent: bool = ROUTER.with(|router| match &router.borrow().m_toServer
    {
        Some(sender) =>
        {
            sender(channel, data.clone());
            true
        }

        None => false,
    });
    if sent
    {
        return true;
    }

    match ROUTER.with(|router| router.borrow().m_localPlayer.clone())
    {
        Some(player) => deliverToServer(channel, &player, &data),
        None => false,
    }
}

pub fn isLoopbackClient() -> bool
{
    crate::side::isClient() && ROUTER.with(|router| router.borrow().m_toServer.is_none())
}

pub fn setPlayerConnection(player: i32, sender: Option<WireSender>)
{
    ROUTER.with(|router| match sender
    {
        Some(sender) => router.borrow_mut().m_toPlayers.insert(player, sender),
        None => router.borrow_mut().m_toPlayers.remove(&player),
    });
}

pub fn hasPlayerConnection(player: i32) -> bool
{
    ROUTER.with(|router| router.borrow().m_toPlayers.contains_key(&player))
}

// a dedicated server writes to the connection, single player hands the bytes to the local client handler
pub fn sendToPlayer(channel: &Identifier, player: &Rc<Entity>, data: Vec<u8>) -> bool
{
    if sendToConnection(player.m_entityId.get(), channel, &data)
    {
        return true;
    }
    if isLoopbackClient()
    {
        return deliverToClient(channel, player, &data);
    }

    false
}

pub fn sendToConnection(player: i32, channel: &Identifier, data: &[u8]) -> bool
{
    ROUTER.with(|router| match router.borrow().m_toPlayers.get(&player)
    {
        Some(sender) =>
        {
            sender(channel, data.to_vec());
            true
        }

        None => false,
    })
}

thread_local!
{
    static BEHIND_CHUNKS: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

// chunk data waits in a queue of its own that the other packets overtake, so what belongs to a block or a chunk is
// sent in that same queue: it then reaches a client after the chunk it is about and in the order it was sent
pub fn behindChunks<T>(send: impl FnOnce() -> T) -> T
{
    let outer: bool = BEHIND_CHUNKS.replace(true);
    let result: T = send();
    BEHIND_CHUNKS.set(outer);
    result
}

pub fn queuedBehindChunks() -> bool { BEHIND_CHUNKS.get() }

// how far vanilla's sendPacketToPlayersAroundPoint reaches
const NEAR_RANGE: f64 = 64.0;

pub enum Reach
{
    // a point in space, as vanilla sounds and effects measure it
    Sphere,
    // every player watching the chunk column, whatever their height
    Column,
}

pub fn sendNear(channel: &Identifier, world: &game::world::World, pos: [f64; 3], reach: Reach, data: &[u8])
{
    for player in world.m_playerEntities.toVec()
    {
        let dx: f64 = player.m_posX.get() - pos[0];
        let dz: f64 = player.m_posZ.get() - pos[2];

        let dy: f64 = match reach
        {
            Reach::Sphere => player.m_posY.get() - pos[1],
            Reach::Column => 0.0,
        };

        if dx * dx + dy * dy + dz * dz <= NEAR_RANGE * NEAR_RANGE
        {
            sendToPlayer(channel, &player, data.to_vec());
        }
    }
}

// every player of every world, for data a whole save shares
pub fn broadcast(channel: &Identifier, data: &[u8])
{
    for world in crate::host::context::worlds()
    {
        for player in world.m_playerEntities.toVec()
        {
            sendToPlayer(channel, &player, data.to_vec());
        }
    }
}

pub fn localPlayer() -> Option<Rc<Entity>> { ROUTER.with(|router| router.borrow().m_localPlayer.clone()) }

pub fn channelInfo(id: &Identifier) -> Option<(String, ChannelDirection)>
{
    ROUTER.with(|router|
    {
        router.borrow().m_channels.get(id).map(|channel| (channel.m_version.clone(), channel.m_direction))
    })
}

pub mod attachments;
pub mod blocks;
pub mod loader;
pub mod stacks;
pub mod sync;
