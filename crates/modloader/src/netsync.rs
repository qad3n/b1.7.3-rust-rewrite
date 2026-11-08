use crate::state::{self, ModRuntime};
use crate::{network, registry};
use mod_protocol::Identifier;
pub use mod_protocol::handshake::LOADER_MAGIC;
use mod_protocol::handshake::{
    ChannelEntry, ConfigSync, Control, DataSync, Hello, ModEntry, ModSide, RegistrySnapshot, acks,
};
use mod_protocol::spec::RecipeSpec;
use mod_protocol::wire::{self, CONTROL_CHANNEL, Frame, limits};
use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, HashMap};
use std::rc::Rc;

// a client must have a shared mod whose ids the server sends, a mod with channels unless its manifest makes
// the client side optional, and any mod whose manifest asks for it, client and server only mods never
pub fn isRequired(runtime: &ModRuntime) -> bool
{
    if runtime.m_manifest.side.wire() != ModSide::Both
    {
        return false;
    }
    let id: &str = runtime.id();
    if crate::remap::ownsSyncedRows(id)
    {
        return true;
    }
    let hasChannels: bool = network::channels().iter().any(|(channel, _, _)| channel.namespace() == id);
    runtime.m_manifest.network.required.unwrap_or(hasChannels)
}

// one side mods run without the other side, so ids they add would never reach or match the other side
pub fn checkSides()
{
    for runtime in state::mods().iter().filter(|runtime| runtime.isActive())
    {
        let side: ModSide = runtime.m_manifest.side.wire();
        if side != ModSide::Both && crate::remap::ownsSyncedRows(runtime.id())
        {
            state::problem(
                runtime.id(),
                &runtime.m_file,
                "adds blocks, items, entities, tile entities, dimensions or biomes but is not side = \"both\"",
            );
        }
    }
}

pub fn localHello(includeServerMods: bool) -> Hello
{
    let mods: Vec<ModEntry> = state::mods()
        .iter()
        .filter(|runtime| runtime.isActive())
        .filter(|runtime| includeServerMods || runtime.m_manifest.side.wire() != ModSide::Server)
        .map(|runtime| ModEntry {
            id: runtime.id().to_owned(),
            version: runtime.m_manifest.version.clone(),
            side: runtime.m_manifest.side.wire(),
            network_version: runtime.m_manifest.network.version.clone(),
            required: isRequired(runtime),
            sha256: runtime.m_sha256.clone(),
        })
        .collect();
    let channels: Vec<ChannelEntry> = network::channels()
        .into_iter()
        .map(|(id, version, direction)| ChannelEntry { id, version, direction, required: false })
        .collect();
    Hello { loader_version: crate::LOADER_VERSION.to_owned(), mods, channels }
}

thread_local! {
    static STRICT_MODS: Cell<bool> = const { Cell::new(false) };
}

// a strict server wants the client list to equal its own, a normal one only its required shared mods
pub fn setStrictMods(strict: bool) { STRICT_MODS.with(|cell| cell.set(strict)); }

fn strictMods() -> bool { STRICT_MODS.with(Cell::get) }

fn checkMode() -> &'static str
{
    match strictMods()
    {
        true => "strict",
        false => "required mods",
    }
}

// what the server refuses, each entry names the mod and why
pub fn serverCheck(server: &Hello, client: &Hello) -> Vec<String>
{
    match strictMods()
    {
        true => strictCheck(server, client),
        false => requiredCheck(server, client),
    }
}

// client only and extra mods pass, shared mods that carry content or channels must match
fn requiredCheck(server: &Hello, client: &Hello) -> Vec<String>
{
    let mut problems: Vec<String> = Vec::new();
    for needed in server.mods.iter().filter(|entry| entry.required)
    {
        match client.mods.iter().find(|entry| entry.id == needed.id)
        {
            None => problems.push(format!("missing {} {}", needed.id, needed.version)),
            Some(found) if found.network_version != needed.network_version => problems.push(format!(
                "{} network version {} but the server has {} ({})",
                needed.id, found.network_version, needed.network_version, needed.version
            )),
            Some(_) =>
            {}
        }
    }
    problems
}

// every mod the server sends must be present as the same file and the client may have nothing else
fn strictCheck(server: &Hello, client: &Hello) -> Vec<String>
{
    let mut problems: Vec<String> = Vec::new();
    for needed in &server.mods
    {
        match client.mods.iter().find(|entry| entry.id == needed.id)
        {
            None => problems.push(format!("missing {} {}", needed.id, needed.version)),
            Some(found) if found.version != needed.version =>
            {
                problems.push(format!("{} {} but the server has {}", found.id, found.version, needed.version))
            }
            Some(found) if found.sha256 != needed.sha256 =>
            {
                problems.push(format!("{} {} is not the server's file", found.id, found.version))
            }
            Some(_) =>
            {}
        }
    }
    for extra in client.mods.iter().filter(|entry| !server.mods.iter().any(|found| found.id == entry.id))
    {
        problems.push(format!("extra {} {}", extra.id, extra.version));
    }
    problems
}

// what the client refuses, its own required mods the server lacks
pub fn clientCheck(server: &Hello, client: &Hello) -> Vec<String>
{
    client
        .mods
        .iter()
        .filter(|entry| entry.required)
        .filter(|entry| !server.mods.iter().any(|found| found.id == entry.id))
        .map(|entry| format!("the server lacks {} {}, which this client requires", entry.id, entry.version))
        .collect()
}

pub fn vanillaClientProblems() -> Vec<String>
{
    let vanilla: Hello = Hello { loader_version: String::new(), mods: Vec::new(), channels: Vec::new() };
    serverCheck(&localHello(false), &vanilla)
}

fn sideLabel(entry: &ModEntry) -> &'static str
{
    match (entry.side, entry.required)
    {
        (ModSide::Both, true) => "required",
        (ModSide::Both, false) => "optional",
        (ModSide::Client, _) => "client only",
        (ModSide::Server, _) => "server only",
    }
}

pub fn logServerMods()
{
    let mods: Vec<ModEntry> = localHello(true).mods;
    if mods.is_empty()
    {
        crate::log::info(crate::log::LOADER, "no mods loaded");
        return;
    }
    let entries: Vec<String> =
        mods.iter().map(|entry| format!("{} {} ({})", entry.id, entry.version, sideLabel(entry))).collect();
    crate::log::info(
        crate::log::LOADER,
        &format!("loaded mods ({}), client check {}: {}", mods.len(), checkMode(), entries.join(", ")),
    );
}

pub fn vanillaServerProblems() -> Vec<String>
{
    localHello(false)
        .mods
        .into_iter()
        .filter(|entry| entry.required)
        .map(|entry| format!("{} {} needs a modded server", entry.id, entry.version))
        .collect()
}

fn modList(mods: &[ModEntry]) -> String
{
    let entries: Vec<String> = mods.iter().map(|entry| format!("{} {}", entry.id, entry.version)).collect();
    entries.join(", ")
}

pub fn kickMessage(problems: &[String]) -> String { format!("Mod mismatch: {}", problems.join(", ")) }

fn recipeTable() -> Vec<(Identifier, RecipeSpec)>
{
    registry::with(|registries| {
        registries
            .m_recipes
            .modded()
            .chain(registries.m_smelting.modded())
            .filter_map(|row| row.m_value.m_spec.clone().map(|spec| (row.m_id.clone(), spec)))
            .collect()
    })
}

type DataTable = (Vec<(Identifier, RecipeSpec)>, String);

fn dataHash() -> String
{
    let bytes: Vec<u8> = wire::encode(&recipeTable());
    let digest = <sha2::Sha256 as sha2::Digest>::digest(&bytes);
    format!("{}{}", crate::discovery::hex(&digest), crate::content::tags::hash())
}

fn configSync() -> ConfigSync
{
    ConfigSync {
        values: state::mods()
            .iter()
            .map(|runtime| {
                (
                    runtime.id().to_owned(),
                    toml::to_string(&runtime.m_config.borrow().serverValues()).unwrap_or_default(),
                )
            })
            .collect(),
    }
}

pub fn send(sender: &dyn Fn(u32, Vec<u8>), control: &Control)
{
    let (channel, body): (u32, Vec<u8>) = wire::compress(CONTROL_CHANNEL, &wire::encode(control));
    sender(channel, body);
}

pub enum Outcome
{
    Continue,
    Kick(String),
    Done,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum ServerStep
{
    Hello,
    Registry,
    Data,
}

// one modded client in the configuration phase, created when its login carried the loader magic
pub struct ServerSession
{
    m_who: String,
    m_step: ServerStep,
    m_peer: Option<Hello>,
    m_channels: Vec<Identifier>,
}

impl ServerSession
{
    pub fn start(sender: &dyn Fn(u32, Vec<u8>), who: String) -> ServerSession
    {
        let channels: Vec<Identifier> = network::channels().into_iter().map(|(id, _, _)| id).collect();
        send(sender, &Control::Hello(localHello(false)));
        send(sender, &Control::Channels(channels.clone()));
        ServerSession { m_who: who, m_step: ServerStep::Hello, m_peer: None, m_channels: channels }
    }

    pub fn peer(&self) -> Option<&Hello> { self.m_peer.as_ref() }

    pub fn channels(&self) -> &[Identifier] { &self.m_channels }

    pub fn receive(&mut self, channel: u32, body: &[u8], sender: &dyn Fn(u32, Vec<u8>)) -> Outcome
    {
        let frame: Frame = match wire::decompress(channel, body, limits::SERVERBOUND_MAX)
        {
            Ok(frame) if frame.m_channel == CONTROL_CHANNEL => frame,
            Ok(_) => return Outcome::Kick("mod channel traffic before the configuration finished".to_owned()),
            Err(error) => return Outcome::Kick(format!("bad loader payload: {error}")),
        };
        let control: Control = match wire::decode(&frame.m_data)
        {
            Ok(control) => control,
            Err(error) => return Outcome::Kick(format!("bad loader payload: {error}")),
        };
        match (self.m_step, control)
        {
            (ServerStep::Hello, Control::HelloReply(peer)) =>
            {
                crate::log::info(crate::log::LOADER, &format!("{} has mods [{}]", self.m_who, modList(&peer.mods)));
                let problems: Vec<String> = serverCheck(&localHello(false), &peer);
                if !problems.is_empty()
                {
                    crate::log::warn(
                        crate::log::LOADER,
                        &format!("{} failed the {} check: {}", self.m_who, checkMode(), problems.join(", ")),
                    );
                    return Outcome::Kick(kickMessage(&problems));
                }
                self.m_peer = Some(peer);
                send(sender, &Control::Registry(crate::remap::snapshot()));
                self.m_step = ServerStep::Registry;
                Outcome::Continue
            }
            (ServerStep::Registry, Control::Ack(ack)) if ack == acks::REGISTRY =>
            {
                send(sender, &Control::Config(configSync()));
                send(sender, &Control::Data(DataSync::Hash(dataHash())));
                self.m_step = ServerStep::Data;
                Outcome::Continue
            }
            (ServerStep::Data, Control::DataRequest) =>
            {
                let table: DataTable = (recipeTable(), String::new());
                send(sender, &Control::Data(DataSync::Full(wire::encode(&table))));
                Outcome::Continue
            }
            (ServerStep::Data, Control::Ack(ack)) if ack == acks::DATA =>
            {
                crate::log::info(
                    crate::log::LOADER,
                    &format!("{} passed the {} check, registry and data synced", self.m_who, checkMode()),
                );
                send(sender, &Control::Done);
                Outcome::Done
            }
            (_, Control::Refused(entries)) => Outcome::Kick(format!("Missing content: {}", entries.join(", "))),
            (step, other) => Outcome::Kick(format!("unexpected loader message {other:?} in step {step:?}")),
        }
    }
}

type RawSender = Rc<dyn Fn(u32, Vec<u8>)>;

#[derive(Default)]
struct ClientState
{
    m_raw: Option<RawSender>,
    m_modded: bool,
    m_channels: Vec<Identifier>,
    m_serverChannels: Vec<(Identifier, String)>,
    m_local: Option<RegistrySnapshot>,
}

thread_local! {
    static CLIENT: RefCell<ClientState> = RefCell::new(ClientState::default());
}

// the id map this client built for itself, restored when it leaves a server
pub fn rememberLocalSnapshot() { CLIENT.with(|client| client.borrow_mut().m_local = Some(crate::remap::snapshot())); }

// the client marks its login so a modded server starts the configuration phase
pub fn loginPacket(mut packet: game::network::packets::Packet1Login) -> game::network::packets::Packet1Login
{
    if state::hasMods()
    {
        packet.m_mapSeed = LOADER_MAGIC;
        packet.m_dimension = mod_protocol::handshake::LOADER_PROTOCOL;
    }
    packet
}

pub fn clientConnected(raw: RawSender)
{
    if !state::hasMods()
    {
        return;
    }
    CLIENT.with(|client| {
        let mut client = client.borrow_mut();
        client.m_raw = Some(Rc::clone(&raw));
        client.m_modded = false;
        client.m_channels.clear();
        client.m_serverChannels.clear();
    });
    network::setServerConnection(Some(Box::new(move |id: &Identifier, data: Vec<u8>| {
        if let Some(index) = clientChannelIndex(id)
        {
            let (channel, body): (u32, Vec<u8>) = wire::compress(index, &data);
            raw(channel, body);
        }
    })));
}

// Some(reason) when this client must leave a server that never spoke the loader protocol
pub fn clientLoggedIn() -> Option<String>
{
    if !state::hasMods() || isModdedServer()
    {
        return None;
    }
    let problems: Vec<String> = vanillaServerProblems();
    if problems.is_empty()
    {
        crate::log::info(crate::log::LOADER, "server has no mod loader, joining without server mods");
        return None;
    }
    let reason: String = problems.join(", ");
    crate::log::warn(crate::log::LOADER, &format!("leaving a server without the mod loader: {reason}"));
    Some(reason)
}

pub fn clientPayload(channel: u32, body: &[u8]) -> Option<String>
{
    let raw: Option<RawSender> = CLIENT.with(|client| client.borrow().m_raw.clone());
    match raw
    {
        Some(raw) => clientReceive(channel, body, &*raw),
        None => Some("mod payload without the mod loader".to_owned()),
    }
}

pub fn clientKicked(reason: &str)
{
    crate::log::warn(crate::log::LOADER, &format!("kicked by the server: {reason}"));
    clientDisconnected();
}

pub fn clientDisconnected()
{
    if !state::hasMods()
    {
        return;
    }
    network::setServerConnection(None);
    CLIENT.with(|client| client.borrow_mut().m_raw = None);
    let local: Option<RegistrySnapshot> = CLIENT.with(|client| {
        let mut client = client.borrow_mut();
        client.m_modded = false;
        client.m_channels.clear();
        client.m_local.clone()
    });
    for runtime in state::mods()
    {
        runtime.m_config.borrow_mut().clearSynced();
    }
    if let Some(local) = local
    {
        crate::remap::apply(&local);
    }
}

pub fn isModdedServer() -> bool { CLIENT.with(|client| client.borrow().m_modded) }

fn applyData(bytes: &[u8]) -> Result<(), String>
{
    let (recipes, _): DataTable = wire::decode(bytes).map_err(|error| error.to_string())?;
    crate::content::recipes::removeModded();
    for (id, spec) in recipes
    {
        crate::content::recipes::apply(&spec).map_err(|reason| format!("recipe {id}: {reason}"))?;
    }
    crate::content::recipes::finish();
    Ok(())
}

// Some(reason) disconnects the client with that reason
fn clientReceive(channel: u32, body: &[u8], sender: &dyn Fn(u32, Vec<u8>)) -> Option<String>
{
    let frame: Frame = match wire::decompress(channel, body, limits::CLIENTBOUND_MAX)
    {
        Ok(frame) => frame,
        Err(error) => return Some(format!("bad loader payload: {error}")),
    };
    if frame.m_channel != CONTROL_CHANNEL
    {
        let id: Option<Identifier> =
            CLIENT.with(|client| client.borrow().m_channels.get(frame.m_channel as usize - 1).cloned());
        match (id, network::localPlayer())
        {
            (Some(id), Some(player)) =>
            {
                network::deliverToClient(&id, &player, &frame.m_data);
            }
            (None, _) =>
            {
                crate::log::warn(crate::log::LOADER, &format!("payload on unknown channel {}", frame.m_channel))
            }
            _ =>
            {}
        }
        return None;
    }
    let control: Control = match wire::decode(&frame.m_data)
    {
        Ok(control) => control,
        Err(error) => return Some(format!("bad loader payload: {error}")),
    };
    match control
    {
        Control::Hello(server) =>
        {
            let local: Hello = localHello(false);
            crate::log::info(crate::log::LOADER, &format!("server mods [{}]", modList(&server.mods)));
            let problems: Vec<String> = clientCheck(&server, &local);
            if !problems.is_empty()
            {
                return Some(problems.join(", "));
            }
            CLIENT.with(|client| {
                client.borrow_mut().m_serverChannels =
                    server.channels.iter().map(|entry| (entry.id.clone(), entry.version.clone())).collect()
            });
            send(sender, &Control::HelloReply(local));
        }
        Control::Channels(channels) => CLIENT.with(|client| client.borrow_mut().m_channels = channels),
        Control::Registry(snapshot) =>
        {
            let missing: Vec<String> = crate::remap::apply(&snapshot);
            let mut required: Vec<String> = missing
                .into_iter()
                .filter(|entry| {
                    entry
                        .split_whitespace()
                        .nth(1)
                        .and_then(|id| id.split(':').next())
                        .is_some_and(|namespace| state::byId(namespace).is_none())
                })
                .collect();
            required.extend(crate::states::mismatches(snapshot.registries.get(crate::states::SNAPSHOT_KEY)));
            let ids: usize = snapshot.registries.values().map(BTreeMap::len).sum();
            crate::log::info(crate::log::LOADER, &format!("applied server registry, {ids} ids"));
            if required.is_empty()
            {
                send(sender, &Control::Ack(acks::REGISTRY.to_owned()));
            }
            else
            {
                send(sender, &Control::Refused(required));
            }
        }
        Control::Config(sync) =>
        {
            for (modId, text) in sync.values
            {
                if let Some(runtime) = state::byId(&modId)
                {
                    let values: BTreeMap<String, String> = toml::from_str(&text).unwrap_or_default();
                    runtime.m_config.borrow_mut().applySynced(values);
                }
            }
        }
        Control::Data(DataSync::Hash(hash)) =>
        {
            let reply: Control = if hash == dataHash()
            {
                Control::Ack(acks::DATA.to_owned())
            }
            else
            {
                Control::DataRequest
            };
            send(sender, &reply);
        }
        Control::Data(DataSync::Full(bytes)) =>
        {
            if let Err(reason) = applyData(&bytes)
            {
                return Some(reason);
            }
            send(sender, &Control::Ack(acks::DATA.to_owned()));
        }
        Control::Done =>
        {
            crate::log::info(crate::log::LOADER, "server handshake done, playing with server mods");
            CLIENT.with(|client| client.borrow_mut().m_modded = true);
        }
        other => return Some(format!("unexpected loader message {other:?}")),
    }
    None
}

// play traffic to the server goes out under the index the server assigned to the channel
pub fn clientChannelIndex(id: &Identifier) -> Option<u32>
{
    CLIENT.with(|client| {
        client.borrow().m_channels.iter().position(|channel| channel == id).map(|index| index as u32 + 1)
    })
}

pub fn serverHasChannel(id: &Identifier) -> bool
{
    CLIENT.with(|client| client.borrow().m_serverChannels.iter().any(|(channel, _)| channel == id))
}

struct Connection
{
    m_channels: Vec<Identifier>,
    m_peerChannels: Vec<Identifier>,
    m_budget: usize,
}

thread_local! {
    static CONNECTIONS: RefCell<HashMap<i32, Connection>> = RefCell::new(HashMap::new());
}

// a modded player entered play, mod traffic now flows over its connection
pub fn serverPlayerJoined(player: i32, session: &ServerSession, raw: Rc<dyn Fn(u32, Vec<u8>)>)
{
    let channels: Vec<Identifier> = session.channels().to_vec();
    let peerChannels: Vec<Identifier> =
        session.peer().map(|peer| peer.channels.iter().map(|entry| entry.id.clone()).collect()).unwrap_or_default();
    CONNECTIONS.with(|connections| {
        connections
            .borrow_mut()
            .insert(player, Connection { m_channels: channels.clone(), m_peerChannels: peerChannels, m_budget: 0 })
    });
    network::setPlayerConnection(
        player,
        Some(Box::new(move |id: &Identifier, data: Vec<u8>| {
            if let Some(index) = channels.iter().position(|channel| channel == id)
            {
                let (channel, body): (u32, Vec<u8>) = wire::compress(index as u32 + 1, &data);
                raw(channel, body);
            }
        })),
    );
}

pub fn serverPlayerLeft(player: i32)
{
    CONNECTIONS.with(|connections| connections.borrow_mut().remove(&player));
    network::setPlayerConnection(player, None);
}

pub fn peerHasChannel(player: i32, id: &Identifier) -> bool
{
    if crate::side::isClient()
    {
        return serverHasChannel(id) || network::isLoopbackClient();
    }
    CONNECTIONS.with(|connections| {
        connections.borrow().get(&player).is_some_and(|connection| connection.m_peerChannels.contains(id))
    })
}

pub fn resetBudgets()
{
    CONNECTIONS.with(|connections| {
        for connection in connections.borrow_mut().values_mut()
        {
            connection.m_budget = 0;
        }
    });
}

// Err kicks the player, oversize payloads, unknown channels and a blown per tick budget all end the connection
pub fn serverReceive(player: &Rc<game::entity::Entity>, channel: u32, body: &[u8]) -> Result<(), String>
{
    let id: i32 = player.m_entityId.get();
    let frame: Frame = wire::decompress(channel, body, limits::SERVERBOUND_MAX).map_err(|error| error.to_string())?;
    let target: Identifier = CONNECTIONS.with(|connections| {
        let mut connections = connections.borrow_mut();
        let connection: &mut Connection = connections.get_mut(&id).ok_or("mod payload from a vanilla connection")?;
        connection.m_budget += body.len();
        if connection.m_budget > limits::SERVERBOUND_BUDGET_PER_TICK
        {
            return Err("too much mod traffic".to_owned());
        }
        connection
            .m_channels
            .get((frame.m_channel as usize).wrapping_sub(1))
            .cloned()
            .ok_or_else(|| format!("unknown mod channel {}", frame.m_channel))
    })?;
    if !network::deliverToServer(&target, player, &frame.m_data)
    {
        return Err(format!("channel {target} takes no serverbound messages"));
    }
    Ok(())
}
