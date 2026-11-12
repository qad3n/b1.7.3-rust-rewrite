use super::{Channel, Handler};
use crate::host::context;
use crate::screens::{OpenRequest, ScreenType};
use game::block::Block;
use game::entity::Entity;
use game::entity::entity_falling_sand::FallingSandFields;
use mod_protocol::abi::BlockPos;
use mod_protocol::handshake::ChannelDirection;
use mod_protocol::{Identifier, wire};
use serde::{Deserialize, Serialize};
use std::rc::Rc;

pub const SPAWN_ENTITY: &str = "loader:spawn_entity";
pub const OPEN_SCREEN: &str = "loader:open_screen";
pub const OPEN_NAMED_SCREEN: &str = "loader:open_named_screen";
pub const SOUND: &str = "loader:sound";
pub const PARTICLE: &str = "loader:particle";
pub const BLOCK_EVENT: &str = "loader:block_event";
pub const ENTITY_SYNCED: &str = "loader:entity_attachment";
pub const TICK: &str = "loader:tick";
pub const MOVING_BLOCK: &str = "loader:moving_block";
pub const BIOME: &str = "loader:biome";
pub const STOP_USING: &str = "loader:stop_using";
pub const ATTRIBUTES: &str = "loader:attributes";
pub const EFFECTS: &str = "loader:effects";
pub const PROGRESS: &str = "loader:progress";

#[derive(Serialize, Deserialize)]
pub struct EntitySynced
{
    pub id: i32,
    pub owner: String,
    pub data: Vec<u8>,
}

// a player's attribute modifiers, sent to that player's own client whenever they change and on login
#[derive(Serialize, Deserialize)]
pub struct AttributeSync
{
    pub id: i32,
    pub modifiers: Vec<SyncedModifier>,
}

#[derive(Serialize, Deserialize)]
pub struct SyncedModifier
{
    pub attribute: String,
    pub id: String,
    pub amount: f64,
    pub operation: String,
}

// a player's status effects with their levels and ticks left, sent to that player's own client
#[derive(Serialize, Deserialize)]
pub struct EffectSync
{
    pub id: i32,
    pub effects: Vec<SyncedEffect>,
}

#[derive(Serialize, Deserialize)]
pub struct SyncedEffect
{
    pub name: String,
    pub level: i32,
    pub ticks: i32,
}

// a player's mod stats and earned mod achievements, sent to that player's own client
#[derive(Serialize, Deserialize)]
pub struct ProgressSync
{
    pub id: i32,
    pub stats: Vec<(String, i32)>,
    pub achievements: Vec<String>,
}

// leads every server tick's entity packets so clients can place samples on the server clock
#[derive(Serialize, Deserialize)]
pub struct Tick
{
    pub tick: i32,
}

// one column a mod turned to another biome, sent as it turns and to every client that joins
#[derive(Serialize, Deserialize)]
pub struct BiomeColumn
{
    pub dimension: String,
    pub x: i32,
    pub z: i32,
    pub biome: String,
}

// the block, metadata and fuse a mod's moving or primed block draws, sent after its spawn and on a server write
#[derive(Serialize, Deserialize)]
pub struct MovingBlock
{
    pub id: i32,
    pub block: i32,
    pub meta: i32,
    pub fuse: i32,
}

#[derive(Serialize, Deserialize)]
pub struct SpawnEntity
{
    pub kind: i32,
    pub id: i32,
    pub pos: [f64; 3],
    pub rotation: [f32; 2],
    pub motion: [f64; 3],
    pub data: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
pub struct OpenScreen
{
    pub screen: i32,
    pub window: i32,
    pub pos: Option<BlockPos>,
    pub data: Vec<u8>,
}

// screens by name, numbers differ when either side has screens from a mod the other lacks
#[derive(Serialize, Deserialize)]
pub struct OpenNamedScreen
{
    pub screen: String,
    pub window: i32,
    pub pos: Option<BlockPos>,
    pub data: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
pub struct Sound
{
    pub world: i32,
    pub pos: [f64; 3],
    pub sound: String,
    pub volume: f32,
    pub pitch: f32,
}

#[derive(Serialize, Deserialize)]
pub struct Particle
{
    pub world: i32,
    pub name: String,
    pub pos: [f64; 3],
    pub motion: [f64; 3],
}

#[derive(Serialize, Deserialize)]
pub struct BlockEvent
{
    pub world: i32,
    pub pos: BlockPos,
    pub kind: i32,
    pub data: i32,
}

pub fn id(name: &str) -> Identifier { Identifier::parse(name).expect("loader channel names are valid") }

fn clientHandler<T: for<'de> Deserialize<'de> + 'static>(apply: fn(T)) -> Handler
{
    Rc::new(move |_: &Rc<Entity>, bytes: &[u8]| match wire::decode::<T>(bytes)
    {
        Ok(message) => apply(message),
        Err(error) => crate::log::warn(crate::log::LOADER, &format!("bad loader message: {error}")),
    })
}

thread_local!
{
    static SERVER_TICK: std::cell::Cell<i32> = const { std::cell::Cell::new(0) };
    // spawns that reached a client still joining its world, replayed once the world is there
    static PENDING_SPAWNS: std::cell::RefCell<Vec<SpawnEntity>> = const { std::cell::RefCell::new(Vec::new()) };
}

pub fn flushPendingSpawns()
{
    if crate::network::localPlayer().and_then(|player| player.m_worldObj.get()).is_none()
    {
        return;
    }

    for message in PENDING_SPAWNS.with(|pending| std::mem::take(&mut *pending.borrow_mut()))
    {
        spawnEntity(message);
    }
}

fn spawnEntity(message: SpawnEntity)
{
    let Some(world) = crate::network::localPlayer().and_then(|player| player.m_worldObj.get())
    else
    {
        PENDING_SPAWNS.with(|pending| pending.borrow_mut().push(message));
        return;
    };
    let Some(entity) = crate::entities::createByNumeric(message.kind, &world)
    else
    {
        crate::log::warn(crate::log::LOADER, &format!("server spawned unknown mod entity {}", message.kind));
        return;
    };
    entity.m_entityId.set(message.id);
    // relative moves build on the tracker's encoded position so both sides must encode alike
    entity.m_serverPosX.set(game::math::math_helper::encodePos(message.pos[0]));
    entity.m_serverPosY.set(game::math::math_helper::encodePos(message.pos[1]));
    entity.m_serverPosZ.set(game::math::math_helper::encodePos(message.pos[2]));
    entity.setPositionAndRotation(
        message.pos[0],
        message.pos[1],
        message.pos[2],
        message.rotation[0],
        message.rotation[1],
    );
    entity.setVelocity(message.motion[0], message.motion[1], message.motion[2]);

    if entity.instanceOf("EntityLiving")
    {
        game::entity::entity_living::fields(&entity).m_isMultiplayerEntity.set(true);
    }

    if !message.data.is_empty()
    {
        crate::entities::setData(&entity, &crate::entities::kindNamespace(&entity), message.data);
    }

    crate::entities::claimStashedSynced(&entity);

    if let Err(reason) = crate::netplay::addClientEntity(message.id, entity)
    {
        crate::log::warn(crate::log::LOADER, &reason);
    }
}

// a synced entity update can beat the spawn of a fresh entity, the spawn handler then applies the stored copy
fn entitySynced(message: EntitySynced)
{
    let entity: Option<Rc<game::entity::Entity>> = crate::network::localPlayer()
        .and_then(|player| player.m_worldObj.get())
        .and_then(|world| context::entityById(&world, message.id));

    match entity
    {
        Some(entity) => crate::entities::setSynced(&entity, &message.owner, message.data),
        None => crate::entities::stashSynced(message.id, message.owner, message.data),
    }
}

fn biomeColumn(message: BiomeColumn)
{
    let (Ok(dimension), Ok(biome)) = (Identifier::parse(&message.dimension), Identifier::parse(&message.biome))
    else
    {
        return;
    };

    crate::biomes::turn(dimension, message.x, message.z, biome);
}

fn movingBlock(message: MovingBlock)
{
    let entity: Option<Rc<game::entity::Entity>> = crate::network::localPlayer()
        .and_then(|player| player.m_worldObj.get())
        .and_then(|world| context::entityById(&world, message.id))
        .filter(|entity| entity.instanceOf("EntityFallingSand"));

    let Some(entity) = entity
    else
    {
        return;
    };

    let fields: &FallingSandFields = game::entity::entity_falling_sand::fields(&entity);
    fields.m_blockID.set(message.block);
    fields.m_metadata.set(message.meta);
    crate::entities::setFuse(&entity, message.fuse);
}

fn tileData(message: super::sync::TileData)
{
    let Some(world) = context::world(message.world)
    else
    {
        return;
    };

    if let Some(tile) = world.getBlockTileEntity(message.pos.x, message.pos.y, message.pos.z)
        && let Some(ext) = tile.tryExt::<crate::tiles::ModTileExt>()
    {
        *ext.m_data.borrow_mut() = message.data;
        world.markBlockNeedsUpdate(message.pos.x, message.pos.y, message.pos.z);
    }
}

fn show(screen: Option<Rc<ScreenType>>, name: &str, window: i32, pos: Option<BlockPos>, data: Vec<u8>)
{
    let Some(screen) = screen
    else
    {
        crate::log::warn(crate::log::LOADER, &format!("server opened unknown screen {name}"));
        return;
    };

    crate::screens::openLocal(OpenRequest { m_screen: screen, m_pos: pos, m_data: data, m_window: window });
}

fn openScreen(message: OpenScreen)
{
    let screen: Option<Rc<ScreenType>> = crate::screens::byNumeric(message.screen);
    show(screen, &message.screen.to_string(), message.window, message.pos, message.data);
}

fn openNamedScreen(message: OpenNamedScreen)
{
    let screen: Option<Rc<ScreenType>> =
        Identifier::parse(&message.screen).ok().and_then(|id| crate::screens::byId(&id));
    show(screen, &message.screen, message.window, message.pos, message.data);
}

// a client that knows screens by name gets the name, an older one the shared sorted index
pub fn sendScreen(player: &Rc<Entity>, request: OpenRequest, window: i32) -> Result<(), String>
{
    let named: Identifier = id(OPEN_NAMED_SCREEN);

    if crate::netsync::peerHasChannel(player.m_entityId.get(), &named)
    {
        let message: OpenNamedScreen = OpenNamedScreen
        {
            screen: request.m_screen.m_id.to_string(),
            window,
            pos: request.m_pos,
            data: request.m_data,
        };
        send(OPEN_NAMED_SCREEN, player, &message);
        return Ok(());
    }

    let screen: i32 = crate::screens::numeric(&request.m_screen.m_id).ok_or("unknown screen")?;
    send(OPEN_SCREEN, player, &OpenScreen { screen, window, pos: request.m_pos, data: request.m_data });
    Ok(())
}

fn sound(message: Sound)
{
    if let Some(world) = context::world(message.world)
    {
        world.playSoundEffect(
            message.pos[0],
            message.pos[1],
            message.pos[2],
            &message.sound,
            message.volume,
            message.pitch,
        );
    }
}

fn particle(message: Particle)
{
    if let Some(world) = context::world(message.world)
    {
        world.spawnParticle(
            &message.name,
            message.pos[0],
            message.pos[1],
            message.pos[2],
            message.motion[0],
            message.motion[1],
            message.motion[2],
        );
    }
}

fn blockEvent(message: BlockEvent)
{
    if let Some(world) = context::world(message.world)
        && let Some(block) = Block::byId(world.getBlockId(message.pos.x, message.pos.y, message.pos.z))
    {
        block.playBlock(&world, message.pos.x, message.pos.y, message.pos.z, message.kind, message.data);
    }
}

fn tick(message: Tick) { game::entity::remote::setServerTick(Some(message.tick)); }

pub fn sendTick()
{
    let tick: i32 = SERVER_TICK.with(|cell|
    {
        cell.set(cell.get().wrapping_add(1));
        cell.get()
    });

    let channel: Identifier = id(TICK);
    let bytes: Vec<u8> = wire::encode(&Tick { tick });

    for player in crate::netsync::peersWith(&channel)
    {
        super::sendToConnection(player, &channel, &bytes);
    }
}

// channels the loader itself owns, registered before mods so every modded peer has them
pub fn register()
{
    let channels: [(&str, Handler); 18] = [
        (SPAWN_ENTITY, clientHandler(spawnEntity)),
        (super::sync::TILE_DATA, clientHandler(tileData)),
        (OPEN_SCREEN, clientHandler(openScreen)),
        (OPEN_NAMED_SCREEN, clientHandler(openNamedScreen)),
        (SOUND, clientHandler(sound)),
        (PARTICLE, clientHandler(particle)),
        (BLOCK_EVENT, clientHandler(blockEvent)),
        (ENTITY_SYNCED, clientHandler(entitySynced)),
        (TICK, clientHandler(tick)),
        (MOVING_BLOCK, clientHandler(movingBlock)),
        (BIOME, clientHandler(biomeColumn)),
        (ATTRIBUTES, clientHandler(crate::attributes::received)),
        (EFFECTS, clientHandler(crate::content::effects::received)),
        (PROGRESS, clientHandler(crate::content::progress::received)),
        (super::stacks::STACK_COMPONENTS, clientHandler(super::stacks::received)),
        (super::attachments::ATTACHMENTS, clientHandler(super::attachments::received)),
        (crate::gamerules::GAMERULES, clientHandler(crate::gamerules::received)),
        // the content message travels in the handshake, the channel only tells the server this client takes it
        (crate::content::served::CONTENT, Rc::new(|_: &Rc<Entity>, _: &[u8]| ())),
    ];

    for (name, handler) in channels
    {
        super::registerChannel(Channel
        {
            m_id: id(name),
            m_version: "1".to_owned(),
            m_direction: ChannelDirection::ToClient,
            m_onServer: None,
            m_onClient: Some(handler),
        });
    }

    super::registerChannel(Channel
    {
        m_id: id(STOP_USING),
        m_version: "1".to_owned(),
        m_direction: ChannelDirection::ToServer,
        m_onServer: Some(Rc::new(|player: &Rc<Entity>, _: &[u8]| game::entity::entity_player::stopUsingItem(player))),
        m_onClient: None,
    });
}

// sends a loader message to every player near a point of a world, used for sounds and particles from mods
pub fn broadcastNear<T: Serialize>(channel: &str, world: &Rc<game::world::World>, pos: [f64; 3], message: &T)
{
    if !crate::side::isServer()
    {
        return;
    }

    super::sendNear(&id(channel), world, pos, super::Reach::Sphere, &wire::encode(message));
}

// the client let go of the use key, the server's copy of the player lets go too
pub fn stopUsing(player: &Rc<Entity>)
{
    if player.world().m_multiplayerWorld.get()
    {
        super::sendToServer(&id(STOP_USING), Vec::new());
    }

    game::entity::entity_player::stopUsingItem(player);
}

pub fn send<T: Serialize>(channel: &str, player: &Rc<Entity>, message: &T) -> bool
{
    super::sendToPlayer(&id(channel), player, wire::encode(message))
}
