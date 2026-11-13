use crate::host::context::{self, entityInfo};
use crate::host::{ok, unit};
use crate::network::{self, Channel};
use crate::state::{self, ModRuntime, Phase};
use crate::{commands, dimensions, events, keybinds, persist, screens, services, structures};
use game::entity::Entity;
use mod_protocol::Identifier;
use mod_protocol::abi::
{
    EntityRequest, GuestCall, HostCall, HostReply, InventoryRequest, MetaRequest, NetRequest, PlayerRequest,
    ScreenRequest, SharedRequest, StoredRequest, Target, ViewerRequest, WorldgenRequest,
};
use mod_protocol::handshake::ChannelDirection;
use std::rc::Rc;

fn unloaded(world: i32) -> String { format!("world {world} is not loaded") }

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
    Rc::new(move |player: &Rc<Entity>, data: &[u8]|
    {
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
            if let Some(world) = context::world(dimension)
            {
                let center: [f64; 3] = [pos.x as f64, pos.y as f64, pos.z as f64];
                network::sendNear(channel, &world, center, network::Reach::Column, &data);
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
        HostCall::Meta(MetaRequest::Subscribe { kind, custom, priority, receive_canceled, handler }) =>
        {
            unit(events::subscribe(index, kind, custom, priority, receive_canceled, handler))
        }

        HostCall::Meta(MetaRequest::DefineEvent { id }) => unit(runtime.own(&id.to_string()).map(events::define)),

        HostCall::Meta(MetaRequest::PostEvent { id, data }) =>
        {
            ok(events::postCustom(id, data), |envelope| HostReply::Bool(envelope.canceled))
        }

        HostCall::Meta(MetaRequest::RegisterCommand { name, permission, usage, handler }) =>
        {
            unit(commands::register(index, &name, permission, usage, handler))
        }

        HostCall::Screen(ScreenRequest::RegisterKeybind { name, key, handler }) =>
        {
            unit(runtime.own(&name).map(|id| keybinds::register(index, id.to_string(), key, handler)))
        }

        HostCall::Net(NetRequest::RegisterChannel { name, version, direction, handler }) =>
        {
            unit(beforeRunning().and_then(|()| runtime.own(&name)).map(|id|
            {
                let onServer: bool = matches!(direction, ChannelDirection::ToServer | ChannelDirection::Both);
                let onClient: bool = matches!(direction, ChannelDirection::ToClient | ChannelDirection::Both);

                network::registerChannel(Channel
                {
                    m_id: id,
                    m_version: version,
                    m_direction: direction,
                    m_onServer: onServer.then(|| channelHandler(index, handler)),
                    m_onClient: onClient.then(|| channelHandler(index, handler)),
                });
            }))
        }

        HostCall::Net(NetRequest::Send { channel, target, data }) => unit(send(&channel, target, data)),
        HostCall::Net(NetRequest::CanSend { player, channel }) =>
        {
            HostReply::Bool(crate::netplay::canSend(player, &channel))
        }

        HostCall::Screen(ScreenRequest::RegisterScreen { name, spec, handler }) =>
        {
            unit(runtime.own(&name).map(|id| screens::register(index, id, spec, handler)))
        }

        HostCall::Screen(ScreenRequest::RegisterPlayerLayer { handler }) =>
        {
            screens::registerPlayerLayer(index, handler);
            HostReply::Unit
        }

        HostCall::Screen(ScreenRequest::RegisterOverlay { target, handler }) =>
        {
            screens::registerOverlay(index, target, handler);
            HostReply::Unit
        }

        HostCall::Screen(ScreenRequest::RegisterTooltip { handler }) =>
        {
            screens::registerTooltip(index, handler);
            HostReply::Unit
        }

        HostCall::Screen(ScreenRequest::RegisterWorldRender { handler }) =>
        {
            screens::registerWorldRender(index, handler);
            HostReply::Unit
        }

        HostCall::Screen(ScreenRequest::RegisterHud { handler }) =>
        {
            screens::registerHud(index, handler);
            HostReply::Unit
        }

        HostCall::Screen(ScreenRequest::OpenScreen { player, screen, pos, data }) =>
        {
            unit(crate::netplay::openScreen(player, &screen, pos, data))
        }

        HostCall::Screen(ScreenRequest::CloseScreen { player }) =>
        {
            unit(crate::host::worldPlayer(player).map(|player| player.closeScreen()))
        }
        HostCall::Shared(SharedRequest::ProvideService { name, version, handler }) =>
        {
            unit(services::provide(index, name, version, handler))
        }

        HostCall::Shared(SharedRequest::CallService { name, version, method, args }) =>
        {
            ok(services::call(&name, version, method, args), HostReply::Bytes)
        }

        HostCall::Shared(SharedRequest::HasService { name, version }) => HostReply::Bool(services::has(&name, version)),

        HostCall::Shared(SharedRequest::RegisterLookup { api, block, handler }) =>
        {
            crate::lifecycle::addLookup(index, api, block, handler);
            HostReply::Unit
        }

        HostCall::Shared(SharedRequest::CallLookup { api, world, pos, side, method, args }) =>
        {
            ok(services::callLookup(&api, world, pos, side, method, args), |reply| match reply
            {
                Some(bytes) => HostReply::Bytes(bytes),
                None => HostReply::Unit,
            })
        }

        HostCall::Shared(SharedRequest::SendInterMod { to, kind, data }) =>
        {
            services::sendInterMod(index, to, kind, data);
            HostReply::Unit
        }

        HostCall::Worldgen(WorldgenRequest::RegisterBiome { name, spec }) =>
        {
            unit(runtime.own(&name).and_then(|id| dimensions::registerBiome(index, id, spec)))
        }

        HostCall::Worldgen(WorldgenRequest::RegisterDimension { name, spec }) =>
        {
            unit(runtime.own(&name).and_then(|id| dimensions::registerDimension(index, id, spec)))
        }

        HostCall::Worldgen(WorldgenRequest::RegisterGenerator { dimension, handler }) =>
        {
            dimensions::registerGenerator(index, dimension, handler);
            HostReply::Unit
        }

        HostCall::Worldgen(WorldgenRequest::RegisterTerrain { dimension, handler }) =>
        {
            unit(dimensions::registerTerrain(index, dimension, handler))
        }

        HostCall::Worldgen(WorldgenRequest::DimensionHeight { dimension, height, build_limit }) =>
        {
            unit(dimensions::registerHeight(index, dimension, height, build_limit))
        }

        HostCall::Worldgen(WorldgenRequest::RegisterBiomeSource { dimension, biomes, handler }) =>
        {
            unit(dimensions::registerBiomeSource(index, dimension, biomes, handler))
        }

        HostCall::Worldgen(WorldgenRequest::RegisterSpawns { dimension, handler }) =>
        {
            unit(dimensions::registerSpawns(index, dimension, handler))
        }
        HostCall::Worldgen(WorldgenRequest::RegisterStructure { name, spec, finder }) => unit(
            beforeRunning()
                .and_then(|()| runtime.own(&name))
                .and_then(|id| structures::register(index, id, spec, finder)),
        ),
        HostCall::Worldgen(WorldgenRequest::AddStructure { world, start }) => unit(
            context::world(world)
                .ok_or_else(|| unloaded(world))
                .and_then(|world| structures::addModded(index, &world, start)),
        ),
        HostCall::Worldgen(WorldgenRequest::FindStructures { world, min, max, structure }) =>
        {
            match context::world(world)
            {
                Some(world) => HostReply::Starts(structures::find(&world, min, max, structure.as_ref())),
                None => HostReply::Error(unloaded(world)),
            }
        }
        HostCall::Worldgen(WorldgenRequest::LocateStructure { world, structure, from, radius }) => ok(
            context::world(world)
                .ok_or_else(|| unloaded(world))
                .and_then(|world| structures::locate(&world, &structure, from, radius)),
            HostReply::Located,
        ),
        HostCall::Worldgen(WorldgenRequest::DimensionSeed { dimension, seed }) =>
        {
            unit(dimensions::registerSeed(index, dimension, seed))
        }

        HostCall::Screen(ScreenRequest::TerrainTiles { texture, columns, rows }) =>
        {
            unit(crate::atlas::requestTiles(crate::atlas::Atlas::Terrain, &texture, columns, rows))
        }

        HostCall::Screen(ScreenRequest::ItemSlot { texture }) =>
        {
            crate::atlas::request(crate::atlas::Atlas::Items, &texture);
            HostReply::Int(crate::content::textureSlot(&texture, crate::atlas::Atlas::Items).unwrap_or(-1))
        }

        HostCall::Screen(ScreenRequest::TerrainSlot { texture }) =>
        {
            crate::atlas::request(crate::atlas::Atlas::Terrain, &texture);
            HostReply::Int(crate::content::textureSlot(&texture, crate::atlas::Atlas::Terrain).unwrap_or(-1))
        }

        HostCall::Worldgen(WorldgenRequest::WorldSeed { world }) => match context::world(world)
        {
            Some(world) => HostReply::Long(world.getRandomSeed()),
            None => HostReply::Error(unloaded(world)),
        },

        HostCall::Worldgen(WorldgenRequest::Colorizer { foliage, temperature, rainfall }) =>
        {
            let (temperature, rainfall): (f64, f64) = (temperature.clamp(0.0, 1.0), rainfall.clamp(0.0, 1.0));

            HostReply::Int(match foliage
            {
                true => game::worldgen::ColorizerFoliage::getFoliageColor(temperature, rainfall),
                false => game::worldgen::ColorizerGrass::getGrassColor(temperature, rainfall),
            })
        }

        // a multiplayer client has no save, so it holds none of this
        HostCall::Stored(StoredRequest::WorldData { key }) =>
        {
            HostReply::Bytes(match crate::host::context::mirroring()
            {
                true => Vec::new(),
                false => persist::worldData(runtime.id(), &key),
            })
        }

        HostCall::Stored(StoredRequest::SetWorldData { key, data }) =>
        {
            persist::setWorldData(runtime.id(), &key, data);
            HostReply::Unit
        }

        HostCall::Stored(StoredRequest::PlayerData { player, key }) => ok(crate::host::worldPlayer(player), |player|
        {
            HostReply::Bytes(crate::entities::data(&player, &persist::playerKey(runtime.id(), &key)))
        }),

        HostCall::Stored(StoredRequest::SetPlayerData { player, key, data }) =>
        {
            unit(crate::host::worldPlayer(player).map(|player|
            {
                crate::entities::setData(&player, &persist::playerKey(runtime.id(), &key), data);
            }))
        }
        HostCall::Player(PlayerRequest::Teleport { player, dimension, pos }) =>
        {
            unit(crate::netplay::teleport(player, dimension, pos))
        }
        HostCall::Player(PlayerRequest::LocalPlayer) =>
        {
            HostReply::Entity(network::localPlayer().map(|player| entityInfo(&player)))
        }
        HostCall::Screen(ScreenRequest::ScreenSize) => match crate::netplay::screenSize()
        {
            Some((width, height)) => HostReply::Size(width, height),
            None => HostReply::Error("screen size exists only on the client".to_owned()),
        },
        HostCall::Viewer(ViewerRequest::ItemCatalog) => HostReply::Stacks(crate::viewer::itemCatalog()),
        HostCall::Entity(EntityRequest::MobList) => HostReply::Ids(crate::viewer::mobs()),
        HostCall::Viewer(ViewerRequest::RecipeList) => HostReply::Recipes(crate::viewer::recipeList()),
        HostCall::Viewer(ViewerRequest::LootList) => HostReply::Loot(crate::viewer::loot::list()),
        HostCall::Viewer(ViewerRequest::MachineList) => HostReply::Machines(crate::viewer::machines::list()),
        HostCall::Inventory(InventoryRequest::RollLoot { table, seed }) =>
        {
            ok(crate::content::loot::rollChest(&table, seed), HostReply::Stacks)
        }
        HostCall::Viewer(ViewerRequest::ContentGeneration) => HostReply::Int(crate::content::recipes::generation()),
        other => HostReply::Error(format!("{other:?} reached no handler")),
    }
}
