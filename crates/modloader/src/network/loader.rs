use super::{Channel, Handler};
use crate::host::context;
use game::block::Block;
use game::entity::Entity;
use mod_protocol::abi::BlockPos;
use mod_protocol::handshake::ChannelDirection;
use mod_protocol::{Identifier, wire};
use serde::{Deserialize, Serialize};
use std::rc::Rc;

pub const SPAWN_ENTITY: &str = "loader:spawn_entity";
pub const OPEN_SCREEN: &str = "loader:open_screen";
pub const SOUND: &str = "loader:sound";
pub const PARTICLE: &str = "loader:particle";
pub const BLOCK_EVENT: &str = "loader:block_event";

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

fn spawnEntity(message: SpawnEntity)
{
    let Some(world) = crate::network::localPlayer().and_then(|player| player.m_worldObj.get())
    else
    {
        return;
    };
    let Some(entity) = crate::entities::createByNumeric(message.kind, &world)
    else
    {
        crate::log::warn(crate::log::LOADER, &format!("server spawned unknown mod entity {}", message.kind));
        return;
    };
    entity.m_entityId.set(message.id);
    entity.m_serverPosX.set((message.pos[0] * 32.0) as i32);
    entity.m_serverPosY.set((message.pos[1] * 32.0) as i32);
    entity.m_serverPosZ.set((message.pos[2] * 32.0) as i32);
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
    if let Err(reason) = crate::netplay::addClientEntity(message.id, entity)
    {
        crate::log::warn(crate::log::LOADER, &reason);
    }
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

fn openScreen(message: OpenScreen)
{
    match crate::screens::byNumeric(message.screen)
    {
        Some(screen) =>
        {
            let request = crate::screens::OpenRequest {
                m_screen: screen,
                m_pos: message.pos,
                m_data: message.data,
                m_window: message.window,
            };
            crate::screens::openLocal(request);
        }
        None => crate::log::warn(crate::log::LOADER, &format!("server opened unknown screen {}", message.screen)),
    }
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

// channels the loader itself owns, registered before mods so every modded peer has them
pub fn register()
{
    let channels: [(&str, Handler); 6] = [
        (SPAWN_ENTITY, clientHandler(spawnEntity)),
        (super::sync::TILE_DATA, clientHandler(tileData)),
        (OPEN_SCREEN, clientHandler(openScreen)),
        (SOUND, clientHandler(sound)),
        (PARTICLE, clientHandler(particle)),
        (BLOCK_EVENT, clientHandler(blockEvent)),
    ];
    for (name, handler) in channels
    {
        super::registerChannel(Channel {
            m_id: id(name),
            m_version: "1".to_owned(),
            m_direction: ChannelDirection::ToClient,
            m_onServer: None,
            m_onClient: Some(handler),
        });
    }
}

// sends a loader message to every player near a point of a world, used for sounds and particles from mods
pub fn broadcastNear<T: Serialize>(channel: &str, world: &Rc<game::world::World>, pos: [f64; 3], message: &T)
{
    const RANGE_SQUARED: f64 = 64.0 * 64.0;
    if !crate::side::isServer()
    {
        return;
    }
    let bytes: Vec<u8> = wire::encode(message);
    for player in world.m_playerEntities.toVec()
    {
        let dx: f64 = player.m_posX.get() - pos[0];
        let dz: f64 = player.m_posZ.get() - pos[2];
        if dx * dx + dz * dz <= RANGE_SQUARED
        {
            super::sendToPlayer(&id(channel), &player, bytes.clone());
        }
    }
}

pub fn send<T: Serialize>(channel: &str, player: &Rc<Entity>, message: &T) -> bool
{
    super::sendToPlayer(&id(channel), player, wire::encode(message))
}
