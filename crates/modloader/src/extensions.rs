use crate::host::context::{self, entityInfo};
use crate::host::{ok, unit};
use crate::network::{self, Channel};
use crate::state::{self, ModRuntime, Phase};
use crate::{commands, dimensions, events, keybinds, persist, screens, services};
use game::entity::Entity;
use mod_protocol::Identifier;
use mod_protocol::abi::{GuestCall, HostCall, HostReply, Target};
use mod_protocol::handshake::ChannelDirection;
use std::rc::Rc;

fn own(runtime: &ModRuntime, name: &str) -> Result<Identifier, String>
{
    Identifier::parseIn(name, runtime.id()).map_err(|error| error.to_string()).and_then(|id| {
        if id.namespace() == runtime.id()
        {
            Ok(id)
        }
        else
        {
            Err(format!("{id} is outside the namespace {}", runtime.id()))
        }
    })
}

fn beforeRunning() -> Result<(), String>
{
    if state::phase() == Phase::Running
    {
        Err("this registration closes once loading finishes".to_owned())
    }
    else
    {
        Ok(())
    }
}

fn channelHandler(modIndex: usize, handler: u32) -> network::Handler
{
    Rc::new(move |player: &Rc<Entity>, data: &[u8]| {
        if let Some(runtime) = state::modAt(modIndex)
        {
            let call: GuestCall = GuestCall::Channel { handler, player: player.m_entityId.get(), data: data.to_vec() };
            match player.m_worldObj.get()
            {
                Some(world) => context::withWorld(&world, || runtime.call(&call)),
                None => runtime.call(&call),
            };
        }
    })
}

fn send(channel: &Identifier, target: Target, data: Vec<u8>) -> Result<(), String>
{
    let size: usize = data.len();
    let limit: usize = match target
    {
        Target::Server => mod_protocol::wire::limits::SERVERBOUND_MAX,
        _ => mod_protocol::wire::limits::CLIENTBOUND_MAX,
    };
    if size > limit
    {
        return Err(format!("payload of {size} bytes exceeds {limit}"));
    }
    match target
    {
        Target::Server =>
        {
            network::sendToServer(channel, data);
        }
        Target::Player(id) =>
        {
            let player: Rc<Entity> = crate::host::worldPlayer(id)?;
            network::sendToPlayer(channel, &player, data);
        }
        Target::All =>
        {
            for player in context::worlds().iter().flat_map(|world| world.m_playerEntities.toVec())
            {
                network::sendToPlayer(channel, &player, data.clone());
            }
        }
        Target::Dimension(dimension) =>
        {
            for player in context::world(dimension).map(|world| world.m_playerEntities.toVec()).unwrap_or_default()
            {
                network::sendToPlayer(channel, &player, data.clone());
            }
        }
        Target::Tracking(dimension, pos) =>
        {
            const RANGE_SQUARED: f64 = 64.0 * 64.0;
            for player in context::world(dimension).map(|world| world.m_playerEntities.toVec()).unwrap_or_default()
            {
                let dx: f64 = player.m_posX.get() - pos.x as f64;
                let dz: f64 = player.m_posZ.get() - pos.z as f64;
                if dx * dx + dz * dz <= RANGE_SQUARED
                {
                    network::sendToPlayer(channel, &player, data.clone());
                }
            }
        }
    }
    Ok(())
}

pub fn handle(runtime: &Rc<ModRuntime>, call: HostCall) -> HostReply
{
    let index: usize = runtime.m_index;
    match call
    {
        HostCall::Subscribe { kind, custom, priority, receive_canceled, handler } =>
        {
            unit(events::subscribe(index, kind, custom, priority, receive_canceled, handler))
        }
        HostCall::DefineEvent { id } => unit(own(runtime, &id.to_string()).map(events::define)),
        HostCall::PostEvent { id, data } =>
        {
            ok(events::postCustom(id, data), |envelope| HostReply::Bool(envelope.canceled))
        }
        HostCall::RegisterCommand { name, permission, usage, handler } =>
        {
            unit(commands::register(index, &name, permission, usage, handler))
        }
        HostCall::RegisterKeybind { name, key, handler } =>
        {
            unit(own(runtime, &name).map(|id| keybinds::register(index, id.to_string(), key, handler)))
        }
        HostCall::RegisterChannel { name, version, direction, handler } =>
        {
            unit(beforeRunning().and_then(|()| own(runtime, &name)).map(|id| {
                let onServer: bool = matches!(direction, ChannelDirection::ToServer | ChannelDirection::Both);
                let onClient: bool = matches!(direction, ChannelDirection::ToClient | ChannelDirection::Both);
                network::registerChannel(Channel {
                    m_id: id,
                    m_version: version,
                    m_direction: direction,
                    m_onServer: onServer.then(|| channelHandler(index, handler)),
                    m_onClient: onClient.then(|| channelHandler(index, handler)),
                });
            }))
        }
        HostCall::Send { channel, target, data } => unit(send(&channel, target, data)),
        HostCall::CanSend { player, channel } => HostReply::Bool(crate::netplay::canSend(player, &channel)),
        HostCall::RegisterScreen { name, spec, handler } =>
        {
            unit(own(runtime, &name).map(|id| screens::register(index, id, spec, handler)))
        }
        HostCall::RegisterPlayerLayer { handler } =>
        {
            screens::registerPlayerLayer(index, handler);
            HostReply::Unit
        }
        HostCall::RegisterOverlay { target, handler } =>
        {
            screens::registerOverlay(index, target, handler);
            HostReply::Unit
        }
        HostCall::RegisterHud { handler } =>
        {
            screens::registerHud(index, handler);
            HostReply::Unit
        }
        HostCall::OpenScreen { player, screen, pos, data } =>
        {
            unit(crate::netplay::openScreen(player, &screen, pos, data))
        }
        HostCall::CloseScreen { player } => unit(crate::host::worldPlayer(player).map(|player| player.closeScreen())),
        HostCall::ProvideService { name, version, handler } => unit(services::provide(index, name, version, handler)),
        HostCall::CallService { name, version, method, args } =>
        {
            ok(services::call(&name, version, method, args), HostReply::Bytes)
        }
        HostCall::HasService { name, version } => HostReply::Bool(services::has(&name, version)),
        HostCall::RegisterLookup { api, block, handler } =>
        {
            let numeric: Option<i32> = crate::registry::with(|registries| registries.blockId(&block));
            match numeric
            {
                Some(numeric) =>
                {
                    services::registerLookup(index, api, numeric, handler);
                    HostReply::Unit
                }
                None =>
                {
                    crate::lifecycle::queueLookup(index, api, block, handler);
                    HostReply::Unit
                }
            }
        }
        HostCall::CallLookup { api, world, pos, side, method, args } =>
        {
            ok(services::callLookup(&api, world, pos, side, method, args), |reply| match reply
            {
                Some(bytes) => HostReply::Bytes(bytes),
                None => HostReply::Unit,
            })
        }
        HostCall::SendInterMod { to, kind, data } =>
        {
            services::sendInterMod(index, to, kind, data);
            HostReply::Unit
        }
        HostCall::RegisterBiome { name, spec } =>
        {
            unit(own(runtime, &name).and_then(|id| dimensions::registerBiome(index, id, spec)))
        }
        HostCall::RegisterDimension { name, spec } =>
        {
            unit(own(runtime, &name).and_then(|id| dimensions::registerDimension(index, id, spec)))
        }
        HostCall::RegisterGenerator { dimension, handler } =>
        {
            dimensions::registerGenerator(index, dimension, handler);
            HostReply::Unit
        }
        HostCall::RegisterTerrain { dimension, handler } =>
        {
            dimensions::registerTerrain(index, dimension, handler);
            HostReply::Unit
        }
        HostCall::WorldData { key } => HostReply::Bytes(persist::worldData(runtime.id(), &key)),
        HostCall::SetWorldData { key, data } =>
        {
            persist::setWorldData(runtime.id(), &key, data);
            HostReply::Unit
        }
        HostCall::PlayerData { player, key } => ok(crate::host::worldPlayer(player), |player| {
            HostReply::Bytes(crate::entities::data(&player, &persist::playerKey(runtime.id(), &key)))
        }),
        HostCall::SetPlayerData { player, key, data } => unit(crate::host::worldPlayer(player).map(|player| {
            crate::entities::setData(&player, &persist::playerKey(runtime.id(), &key), data);
        })),
        HostCall::Teleport { player, dimension, pos } => unit(crate::netplay::teleport(player, dimension, pos)),
        HostCall::LocalPlayer => HostReply::Entity(network::localPlayer().map(|player| entityInfo(&player))),
        HostCall::ScreenSize => match crate::netplay::screenSize()
        {
            Some((width, height)) => HostReply::Size(width, height),
            None => HostReply::Error("screen size exists only on the client".to_owned()),
        },
        HostCall::ItemCatalog => HostReply::Stacks(crate::viewer::itemCatalog()),
        HostCall::RecipeList => HostReply::Recipes(crate::viewer::recipeList()),
        HostCall::ContentGeneration => HostReply::Int(crate::content::recipes::generation()),
        other => HostReply::Error(format!("{other:?} reached no handler")),
    }
}
